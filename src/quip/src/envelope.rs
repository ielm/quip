//!
//! A envelope wraps messages and signs them to help user identify message senders
//! and instruct Quip how to send messages back to them

use crate::broadcast::Sender;
use crate::message::{Message, Msg, QuipMessage};
use crate::path::QuipPath;
use crate::system::SYSTEM;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct Envelope {
    pub(crate) msg: QuipMessage,
    pub(crate) sign: RefAddr,
}

#[derive(Debug)]
/// A struct containing a message and its sender signature
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
///     children.with_exec(|ctx: QuipContext| {
///         async move {
///             let msg: SignedMessage = ctx.recv().await?;
///
///             Ok(())
///         }
///     })
/// }).expect("Couldn't create the children group.");
/// #
/// # Quip::start();
/// # Quip::stop();
/// # Quip::block_until_stopped();
/// # }
/// ```
pub struct SignedMessage {
    pub(crate) msg: Msg,
    pub(crate) sign: RefAddr,
}

impl SignedMessage {
    pub(crate) fn new(msg: Msg, sign: RefAddr) -> Self {
        SignedMessage { msg, sign }
    }

    #[doc(hidden)]
    pub fn extract(self) -> (Msg, RefAddr) {
        (self.msg, self.sign)
    }

    /// Returns a message signature to identify the message sender
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
    ///     children.with_exec(|ctx: QuipContext| {
    ///         async move {
    ///             let msg: SignedMessage = ctx.recv().await?;
    ///             println!("received message from {:?}", msg.signature().path());
    ///             Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn signature(&self) -> &RefAddr {
        &self.sign
    }
}

#[derive(Debug, Clone)]
/// Message signature used to identify message sender and send messages to it.
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
///     children.with_exec(|ctx: QuipContext| {
///         async move {
///             // Wait for a message to be received...
///             let msg: SignedMessage = ctx.recv().await?;
///             
///             ctx.tell(msg.signature(), "reply").expect("Unable to reply");
///             Ok(())
///         }
///     })
/// }).expect("Couldn't create the children group.");
/// #
/// # Quip::start();
/// # Quip::stop();
/// # Quip::block_until_stopped();
/// # }
/// ```
pub struct RefAddr {
    path: Arc<QuipPath>,
    sender: Sender,
}

impl RefAddr {
    pub(crate) fn new(path: Arc<QuipPath>, sender: Sender) -> Self {
        RefAddr { path, sender }
    }

    pub(crate) fn dead_letters() -> Self {
        Self::new(
            SYSTEM.dead_letters().path().clone(),
            SYSTEM.dead_letters().sender().clone(),
        )
    }

    /// Checks whether the sender is identified.
    /// Usually anonymous sender means messages sent by
    /// [broadcast][crate::Quip::broadcast()] and it's other methods implied to
    /// be called outside of the Quip context.
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
    ///     #
    ///     # let children_ref = Quip::children(|children| children).unwrap();
    /// let msg = "A message containing data.";
    /// children_ref.broadcast(msg).expect("Couldn't send the message.");
    ///
    ///     # Quip::children(|children| {
    ///         # children.with_exec(|ctx: QuipContext| {
    ///             # async move {
    /// msg! { ctx.recv().await?,
    ///     ref msg: &'static str => {
    ///         assert!(signature!().is_sender_identified());
    ///     };
    ///     // We are only sending a `&'static str` in this
    ///     // example, so we know that this won't happen...
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
    pub fn is_sender_identified(&self) -> bool {
        self.path.is_dead_letters()
    }

    /// Returns `QuipPath` of a sender
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
    ///     #
    ///     # let children_ref = Quip::children(|children| children).unwrap();
    /// let msg = "A message containing data.";
    /// children_ref.broadcast(msg).expect("Couldn't send the message.");
    ///
    ///     # Quip::children(|children| {
    ///         # children.with_exec(|ctx: QuipContext| {
    ///             # async move {
    /// msg! { ctx.recv().await?,
    ///     ref msg: &'static str => {
    ///         let path = signature!().path();
    ///         assert!(path.is_dead_letters());
    ///     };
    ///     // We are only sending a `&'static str` in this
    ///     // example, so we know that this won't happen...
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
    pub fn path(&self) -> &Arc<QuipPath> {
        &self.path
    }

    pub(crate) fn sender(&self) -> &Sender {
        &self.sender
    }
}

impl Envelope {
    pub(crate) fn new(msg: QuipMessage, path: Arc<QuipPath>, sender: Sender) -> Self {
        Envelope {
            msg,
            sign: RefAddr::new(path, sender),
        }
    }

    pub(crate) fn new_with_sign(msg: QuipMessage, sign: RefAddr) -> Self {
        Envelope { msg, sign }
    }

    pub(crate) fn from_dead_letters(msg: QuipMessage) -> Self {
        Envelope {
            msg,
            sign: RefAddr::dead_letters(),
        }
    }

    pub(crate) fn try_clone(&self) -> Option<Self> {
        self.msg.try_clone().map(|msg| Envelope {
            msg,
            sign: self.sign.clone(),
        })
    }

    pub(crate) fn into_msg<M: Message>(self) -> Option<M> {
        self.msg.into_msg()
    }
}
