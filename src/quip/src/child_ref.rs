// FIXME: remove and add custom errors
#![allow(clippy::result_unit_err)]
//!
//! Allows users to communicate with Child through the mailboxes.
use crate::context::QuipId;
use crate::envelope::{Envelope, RefAddr};
use crate::message::{Answer, Message, QuipMessage};
use crate::path::QuipPath;
use crate::{broadcast::Sender, prelude::SendError};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::{debug, trace};

#[derive(Debug, Clone)]
/// A "reference" to an element of a children group, allowing to
/// communicate with it.
pub struct ChildRef {
    id: QuipId,
    sender: Sender,
    name: String,
    path: Arc<QuipPath>,
    // True if the ChildRef references a child that will receive user defined messages.
    // use `ChildRef::new_internal` to set it to false, for internal use children,
    // such as the heartbeat children for example
    is_public: bool,
}

impl ChildRef {
    pub(crate) fn new_internal(
        id: QuipId,
        sender: Sender,
        name: String,
        path: Arc<QuipPath>,
    ) -> ChildRef {
        ChildRef {
            id,
            sender,
            name,
            path,
            is_public: false,
        }
    }

    pub(crate) fn new(id: QuipId, sender: Sender, name: String, path: Arc<QuipPath>) -> ChildRef {
        ChildRef {
            id,
            sender,
            name,
            path,
            is_public: true,
        }
    }

    /// Returns the identifier of the children group element this
    /// `ChildRef` is referencing.
    ///
    /// Note that the children group element's identifier is reset
    /// when it is restarted.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use quip::prelude::*;
    /// #
    /// # #[cfg(feature = "tokio-runtime")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # #[cfg(not(feature = "tokio-runtime"))]
    /// # fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # fn run() {
    /// # Quip::init();
    /// #
    /// Quip::children(|children| {
    ///     children.with_exec(|ctx| {
    ///         async move {
    ///             let child_id: &QuipId = ctx.current().id();
    ///             // ...
    ///             # Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn id(&self) -> &QuipId {
        &self.id
    }

    /// Returns true if the child this `ChildRef` is referencing is public,
    /// Which means it can receive messages. private `ChildRef`s
    /// reference quip internal children, such as the heartbeat child for example.
    /// This function comes in handy when implementing your own dispatchers.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// # use quip::prelude::*;
    /// #
    /// # #[cfg(feature = "tokio-runtime")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # #[cfg(not(feature = "tokio-runtime"))]
    /// # fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # fn run() {
    /// # Quip::init();
    /// #
    /// Quip::children(|children| {
    ///     children.with_exec(|ctx| {
    ///         async move {
    ///             if ctx.current().is_public() {
    ///                 // ...
    ///             }
    ///             # Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn is_public(&self) -> bool {
        self.is_public
    }

    /// Sends a message to the child this `ChildRef` is referencing.
    /// This message is intended to be used outside of Quip context when
    /// there is no way for receiver to identify message sender
    ///
    /// This method returns `()` if it succeeded, or `Err(msg)`
    /// otherwise.
    ///
    /// # Argument
    ///
    /// * `msg` - The message to send.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use quip::prelude::*;
    /// #
    /// # #[cfg(feature = "tokio-runtime")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # #[cfg(not(feature = "tokio-runtime"))]
    /// # fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # fn run() {
    ///     # Quip::init();
    /// // The message that will be "told"...
    /// const TELL_MSG: &'static str = "A message containing data (tell).";
    ///
    ///     # let children_ref =
    /// // Create a new child...
    /// Quip::children(|children| {
    ///     children.with_exec(|ctx: QuipContext| {
    ///         async move {
    ///             // ...which will receive the message "told"...
    ///             msg! { ctx.recv().await?,
    ///                 msg: &'static str => {
    ///                     assert_eq!(msg, TELL_MSG);
    ///                     // Handle the message...
    ///                 };
    ///                 // This won't happen because this example
    ///                 // only "tells" a `&'static str`...
    ///                 _: _ => ();
    ///             }
    ///
    ///             Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    ///
    ///     # let child_ref = &children_ref.elems()[0];
    /// // Later, the message is "told" to the child...
    /// child_ref.tell_anonymously(TELL_MSG).expect("Couldn't send the message.");
    ///     #
    ///     # Quip::start();
    ///     # Quip::stop();
    ///     # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn tell_anonymously<M: Message>(&self, msg: M) -> Result<(), M> {
        debug!("ChildRef({}): Telling message: {:?}", self.id(), msg);
        let msg = QuipMessage::tell(msg);
        let env = Envelope::from_dead_letters(msg);
        // FIXME: panics?
        self.send(env).map_err(|env| env.into_msg().unwrap())
    }

    /// Try to send a message to the child this `ChildRef` is referencing.
    /// This message is intended to be used outside of Quip context when
    /// there is no way for receiver to identify message sender
    ///
    /// This method returns `()` if it succeeded, or a `SendError`(../child_ref/enum.SendError.html)
    /// otherwise.
    ///
    /// # Argument
    ///
    /// * `msg` - The message to send.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use quip::prelude::*;
    /// #
    /// # #[cfg(feature = "tokio-runtime")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # #[cfg(not(feature = "tokio-runtime"))]
    /// # fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # fn run() {
    ///     # Quip::init();
    /// // The message that will be "told"...
    /// const TELL_MSG: &'static str = "A message containing data (tell).";
    ///
    ///     # let children_ref =
    /// // Create a new child...
    /// Quip::children(|children| {
    ///     children.with_exec(|ctx: QuipContext| {
    ///         async move {
    ///             // ...which will receive the message "told"...
    ///             msg! { ctx.recv().await?,
    ///                 msg: &'static str => {
    ///                     assert_eq!(msg, TELL_MSG);
    ///                     // Handle the message...
    ///                 };
    ///                 // This won't happen because this example
    ///                 // only "tells" a `&'static str`...
    ///                 _: _ => ();
    ///             }
    ///
    ///             Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    ///
    ///     # let child_ref = &children_ref.elems()[0];
    /// // Later, the message is "told" to the child...
    /// child_ref.try_tell_anonymously(TELL_MSG).expect("Couldn't send the message.");
    ///     #
    ///     # Quip::start();
    ///     # Quip::stop();
    ///     # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn try_tell_anonymously<M: Message>(&self, msg: M) -> Result<(), SendError> {
        debug!("ChildRef({}): Try Telling message: {:?}", self.id(), msg);
        let msg = QuipMessage::tell(msg);
        let env = Envelope::from_dead_letters(msg);
        self.try_send(env).map_err(Into::into)
    }

    /// Sends a message to the child this `ChildRef` is referencing,
    /// allowing it to answer.
    /// This message is intended to be used outside of Quip context when
    /// there is no way for receiver to identify message sender
    ///
    /// This method returns [`Answer`](../message/struct.Answer.html) if it succeeded, or `Err(msg)`
    /// otherwise.
    ///
    /// # Argument
    ///
    /// * `msg` - The message to send.
    ///
    /// # Example
    ///
    /// ```
    /// # use quip::prelude::*;
    /// #
    /// # #[cfg(feature = "tokio-runtime")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # #[cfg(not(feature = "tokio-runtime"))]
    /// # fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # fn run() {
    ///     # Quip::init();
    /// // The message that will be "asked"...
    /// const ASK_MSG: &'static str = "A message containing data (ask).";
    /// // The message the will be "answered"...
    /// const ANSWER_MSG: &'static str = "A message containing data (answer).";
    ///
    ///     # let children_ref =
    /// // Create a new child...
    /// Quip::children(|children| {
    ///     children.with_exec(|ctx: QuipContext| {
    ///         async move {
    ///             // ...which will receive the message asked...
    ///             msg! { ctx.recv().await?,
    ///                 msg: &'static str =!> {
    ///                     assert_eq!(msg, ASK_MSG);
    ///                     // Handle the message...
    ///
    ///                     // ...and eventually answer to it...
    ///                     answer!(ctx, ANSWER_MSG);
    ///                 };
    ///                 // This won't happen because this example
    ///                 // only "asks" a `&'static str`...
    ///                 _: _ => ();
    ///             }
    ///
    ///             Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    ///
    ///     # Quip::children(|children| {
    ///         # children.with_exec(move |ctx: QuipContext| {
    ///             # let child_ref = children_ref.elems()[0].clone();
    ///             # async move {
    /// // Later, the message is "asked" to the child...
    /// let answer: Answer = child_ref.ask_anonymously(ASK_MSG).expect("Couldn't send the message.");
    ///
    /// // ...and the child's answer is received...
    /// msg! { answer.await.expect("Couldn't receive the answer."),
    ///     msg: &'static str => {
    ///         assert_eq!(msg, ANSWER_MSG);
    ///         // Handle the answer...
    ///     };
    ///     // This won't happen because this example
    ///     // only answers a `&'static str`...
    ///     _: _ => ();
    /// }
    ///                 #
    ///                 # Ok(())
    ///             # }
    ///         # })
    ///     # }).unwrap();
    ///     #
    ///     # Quip::start();
    ///     # Quip::stop();
    ///     # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn ask_anonymously<M: Message>(&self, msg: M) -> Result<Answer, M> {
        debug!("ChildRef({}): Asking message: {:?}", self.id(), msg);
        let (msg, answer) = QuipMessage::ask(msg, self.addr());
        let env = Envelope::from_dead_letters(msg);
        // FIXME: panics?
        self.send(env).map_err(|env| env.into_msg().unwrap())?;

        Ok(answer)
    }

    /// Try to send a message to the child this `ChildRef` is referencing,
    /// allowing it to answer.
    /// This message is intended to be used outside of Quip context when
    /// there is no way for receiver to identify message sender
    ///
    /// This method returns [`Answer`](../message/struct.Answer.html) if it succeeded, or a `SendError`(../child_ref/enum.SendError.html)
    /// otherwise.
    ///
    /// # Argument
    ///
    /// * `msg` - The message to send.
    ///
    /// # Example
    ///
    /// ```
    /// # use quip::prelude::*;
    /// #
    /// # #[cfg(feature = "tokio-runtime")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # #[cfg(not(feature = "tokio-runtime"))]
    /// # fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # fn run() {
    ///     # Quip::init();
    /// // The message that will be "asked"...
    /// const ASK_MSG: &'static str = "A message containing data (ask).";
    /// // The message the will be "answered"...
    /// const ANSWER_MSG: &'static str = "A message containing data (answer).";
    ///
    ///     # let children_ref =
    /// // Create a new child...
    /// Quip::children(|children| {
    ///     children.with_exec(|ctx: QuipContext| {
    ///         async move {
    ///             // ...which will receive the message asked...
    ///             msg! { ctx.recv().await?,
    ///                 msg: &'static str =!> {
    ///                     assert_eq!(msg, ASK_MSG);
    ///                     // Handle the message...
    ///
    ///                     // ...and eventually answer to it...
    ///                     answer!(ctx, ANSWER_MSG);
    ///                 };
    ///                 // This won't happen because this example
    ///                 // only "asks" a `&'static str`...
    ///                 _: _ => ();
    ///             }
    ///
    ///             Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    ///
    ///     # Quip::children(|children| {
    ///         # children.with_exec(move |ctx: QuipContext| {
    ///             # let child_ref = children_ref.elems()[0].clone();
    ///             # async move {
    /// // Later, the message is "asked" to the child...
    /// let answer: Answer = child_ref.try_ask_anonymously(ASK_MSG).expect("Couldn't send the message.");
    ///
    /// // ...and the child's answer is received...
    /// msg! { answer.await.expect("Couldn't receive the answer."),
    ///     msg: &'static str => {
    ///         assert_eq!(msg, ANSWER_MSG);
    ///         // Handle the answer...
    ///     };
    ///     // This won't happen because this example
    ///     // only answers a `&'static str`...
    ///     _: _ => ();
    /// }
    ///                 #
    ///                 # Ok(())
    ///             # }
    ///         # })
    ///     # }).unwrap();
    ///     #
    ///     # Quip::start();
    ///     # Quip::stop();
    ///     # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn try_ask_anonymously<M: Message>(&self, msg: M) -> Result<Answer, SendError> {
        debug!("ChildRef({}): Try Asking message: {:?}", self.id(), msg);
        let (msg, answer) = QuipMessage::ask(msg, self.addr());
        let env = Envelope::from_dead_letters(msg);
        self.try_send(env).map(|_| answer)
    }

    /// Sends a message to the child this `ChildRef` is referencing
    /// to tell it to stop its execution.
    ///
    /// This method returns `()` if it succeeded, or `Err(())`
    /// otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # use quip::prelude::*;
    /// #
    /// # #[cfg(feature = "tokio-runtime")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # #[cfg(not(feature = "tokio-runtime"))]
    /// # fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # fn run() {
    /// # Quip::init();
    /// # let children_ref =
    /// # Quip::children(|children| {
    ///     children.with_exec(|ctx: QuipContext| {
    ///         async move {
    ///             // ...which will receive the message asked...
    ///             msg! { ctx.recv().await?,
    ///                 msg: &'static str =!> {
    ///                     // Handle the message...
    ///
    ///                     // ...and eventually answer to it...
    ///                 };
    ///                 // This won't happen because this example
    ///                 // only "asks" a `&'static str`...
    ///                 _: _ => ();
    ///             }
    ///
    ///             Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    ///     # Quip::start();
    ///     # let child_ref = &children_ref.elems()[0];
    ///     child_ref.stop().expect("Couldn't send the message.");
    ///     #
    ///     # Quip::stop();
    ///     # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn stop(&self) -> Result<(), ()> {
        debug!("ChildRef({}): Stopping.", self.id);
        let msg = QuipMessage::stop();
        let env = Envelope::from_dead_letters(msg);
        self.send(env).map_err(|_| ())
    }

    /// Sends a message to the child this `ChildRef` is referencing
    /// to tell it to suicide.
    ///
    /// This method returns `()` if it succeeded, or `Err(())`
    /// otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # use quip::prelude::*;
    /// #
    /// # #[cfg(feature = "tokio-runtime")]
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # #[cfg(not(feature = "tokio-runtime"))]
    /// # fn main() {
    /// #    run();    
    /// # }
    /// #
    /// # fn run() {
    /// # Quip::init();
    /// #
    /// # let children_ref = Quip::children(|children| children).unwrap();
    /// # let child_ref = &children_ref.elems()[0];
    /// # child_ref.kill().expect("Couldn't send the message.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn kill(&self) -> Result<(), ()> {
        debug!("ChildRef({}): Killing.", self.id());
        let msg = QuipMessage::kill();
        let env = Envelope::from_dead_letters(msg);
        self.send(env).map_err(|_| ())
    }

    /// Returns [`RefAddr`] for the child
    pub fn addr(&self) -> RefAddr {
        RefAddr::new(self.path.clone(), self.sender.clone())
    }

    pub(crate) fn send(&self, env: Envelope) -> Result<(), Envelope> {
        trace!("ChildRef({}): Sending message: {:?}", self.id(), env);
        self.sender
            .unbounded_send(env)
            .map_err(|err| err.into_inner())
    }

    pub(crate) fn try_send(&self, env: Envelope) -> Result<(), SendError> {
        trace!("ChildRef({}): Sending message: {:?}", self.id(), env);
        self.sender.unbounded_send(env).map_err(Into::into)
    }

    pub(crate) fn sender(&self) -> &Sender {
        &self.sender
    }

    /// Returns the [`QuipPath`] of the child
    pub fn path(&self) -> &Arc<QuipPath> {
        &self.path
    }

    /// Return the `name` of the child
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for ChildRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ChildRef {}

impl Hash for ChildRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
