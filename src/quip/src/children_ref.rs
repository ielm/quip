// FIXME: remove and add custom errors
#![allow(clippy::result_unit_err)]

//!
//! Allows users to communicate with children through the mailboxes.
use crate::broadcast::Sender;
use crate::context::QuipId;
use crate::dispatcher::DispatcherType;
use crate::envelope::Envelope;
use crate::message::{Message, QuipMessage};
use crate::path::QuipPath;
use crate::system::SYSTEM;
use crate::{child_ref::ChildRef, distributor::Distributor};
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{debug, trace};

#[derive(Debug, Clone)]
/// A "reference" to a children group, allowing to communicate
/// with it.
pub struct ChildrenRef {
    id: QuipId,
    sender: Sender,
    path: Arc<QuipPath>,
    children: Vec<ChildRef>,
    dispatchers: Vec<DispatcherType>,
    distributors: Vec<Distributor>,
}

impl ChildrenRef {
    pub(crate) fn new(
        id: QuipId,
        sender: Sender,
        path: Arc<QuipPath>,
        children: Vec<ChildRef>,
        dispatchers: Vec<DispatcherType>,
        distributors: Vec<Distributor>,
    ) -> Self {
        ChildrenRef {
            id,
            sender,
            path,
            children,
            dispatchers,
            distributors,
        }
    }

    /// Returns the identifier of the children group this `ChildrenRef`
    /// is referencing.
    ///
    /// Note that the children group's identifier is reset when it
    /// is restarted.
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
    /// let children_ref = Quip::children(|children| {
    ///     // ...
    /// # children
    /// }).expect("Couldn't create the children group.");
    ///
    /// let children_id: &QuipId = children_ref.id();
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn id(&self) -> &QuipId {
        &self.id
    }

    /// Returns a list of dispatcher names that can be used for
    /// communication with other actors in the same group(s).
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
    /// # let children_ref = Quip::children(|children| children).unwrap();
    /// let dispatchers = children_ref.dispatchers();
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn dispatchers(&self) -> &Vec<DispatcherType> {
        &self.dispatchers
    }

    /// Returns a list of distributors that can be used for
    /// communication with other actors in the same group(s).
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
    /// # let children_ref = Quip::children(|children| children).unwrap();
    /// let distributors = children_ref.distributors();
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn distributors(&self) -> &Vec<Distributor> {
        &self.distributors
    }

    /// Returns a list of [`ChildRef`] referencing the elements
    /// of the children group this `ChildrenRef` is referencing.
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
    /// # let children_ref = Quip::children(|children| children).unwrap();
    /// let elems: &[ChildRef] = children_ref.elems();
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn elems(&self) -> &[ChildRef] {
        &self.children
    }

    /// Sends a message to the children group this `ChildrenRef`
    /// is referencing which will then send it to all of its
    /// elements.
    ///
    /// An alternative would be to use [`elems`] to get all the
    /// elements of the group and then send the message to all
    /// of them.
    ///
    /// This method returns `()` if it succeeded, or `Err(msg)`
    /// otherwise.
    ///
    /// # Arguments
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
    ///     #
    ///     # let children_ref = Quip::children(|children| children).unwrap();
    /// let msg = "A message containing data.";
    /// children_ref.broadcast(msg).expect("Couldn't send the message.");
    ///
    ///     # Quip::children(|children| {
    ///         # children.with_exec(|ctx: QuipContext| {
    ///             # async move {
    /// // And then in every of the children group's elements' futures...
    /// msg! { ctx.recv().await?,
    ///     ref msg: &'static str => {
    ///         assert_eq!(msg, &"A message containing data.");
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
    ///
    /// [`elems`]: Self::elems
    pub fn broadcast<M: Message>(&self, msg: M) -> Result<(), M> {
        debug!(
            "ChildrenRef({}): Broadcasting message: {:?}",
            self.id(),
            msg
        );
        let msg = QuipMessage::broadcast(msg);
        let env = Envelope::from_dead_letters(msg);
        // FIXME: panics?
        self.send(env).map_err(|err| err.into_msg().unwrap())
    }

    /// Sends a message to the children group this `ChildrenRef`
    /// is referencing to tell it to stop all of its running
    /// elements.
    ///
    /// This method returns `()` if it succeeded, or `Err(())`
    /// otherwise.
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
    /// # let children_ref = Quip::children(|children| children).unwrap();
    /// children_ref.stop().expect("Couldn't send the message.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn stop(&self) -> Result<(), ()> {
        debug!("ChildrenRef({}): Stopping.", self.id());
        let msg = QuipMessage::stop();
        let env = Envelope::from_dead_letters(msg);
        self.send(env).map_err(|_| ())
    }

    /// Sends a message to the children group this `ChildrenRef`
    /// is referencing to tell it to kill all of its running
    /// elements.
    ///
    /// This method returns `()` if it succeeded, or `Err(())`
    /// otherwise.
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
    /// # let children_ref = Quip::children(|children| children).unwrap();
    /// children_ref.kill().expect("Couldn't send the message.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn kill(&self) -> Result<(), ()> {
        debug!("ChildrenRef({}): Killing.", self.id());
        let msg = QuipMessage::kill();
        let env = Envelope::from_dead_letters(msg);
        self.send(env).map_err(|_| ())
    }

    pub(crate) fn send(&self, env: Envelope) -> Result<(), Envelope> {
        trace!("ChildrenRef({}): Sending message: {:?}", self.id(), env);
        self.sender.unbounded_send(env).or_else(|err| {
            SYSTEM
                .dead_letters()
                .sender
                .unbounded_send(err.into_inner())
                .map_err(|err| err.into_inner())
        })
    }

    /// Returns the [`QuipPath`] of this ChildrenRef
    pub fn path(&self) -> &Arc<QuipPath> {
        &self.path
    }

    pub(crate) fn sender(&self) -> &Sender {
        &self.sender
    }
}

impl PartialEq for ChildrenRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ChildrenRef {}
