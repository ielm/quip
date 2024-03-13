use crate::broadcast::{Broadcast, Parent};
use crate::children::Children;
use crate::children_ref::ChildrenRef;
use crate::config::Config;
use crate::context::{QuipContext, QuipId};
use crate::envelope::Envelope;
use crate::message::{Message, QuipMessage};
use crate::path::QuipPathElement;
use crate::supervisor::{Supervisor, SupervisorRef};
use crate::system::SYSTEM;

use core::future::Future;
use tracing::{debug, trace};

use std::fmt::{self, Debug, Formatter};

distributed_api! {
    use std::sync::Arc;
    use crate::distributed::*;
    use artillery_core::cluster::ap::*;
}

/// A `struct` allowing to access the system's API to initialize it,
/// start, stop and kill it and to create new supervisors and top-level
/// children groups.
///
/// # Example
///
/// ```rust
/// use quip::prelude::*;
///
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
/// fn run() {
///     /// Creating the system's configuration...
///     let config = Config::new().hide_backtraces();
///     // ...and initializing the system with it (this is required)...
///     Quip::init_with(config);
///
///     // Note that `Quip::init();` would work too and initialize
///     // the system with the default config.
///
///     // Starting the system...
///     Quip::start();
///
///     // Creating a new supervisor...
///     let supervisor = Quip::supervisor(|sp| {
///         sp
///         // ...with a specific supervision strategy...
///             .with_strategy(SupervisionStrategy::OneForAll)
///         // ...and some supervised children groups...
///             .children(|children| {
///                 // ...
///                 # children
///             })
///             .children(|children| {
///                 // ...
///                 # children
///             })
///         // ...or even supervised supervisors...
///             .supervisor(|sp| {
///                 // ...
///                 # sp
///             })
///     }).expect("Couldn't create the supervisor.");
///
///     // ...which can start supervising new children groups
///     // later on...
///     supervisor.children(|children| {
///         // ...
///         # children
///     }).expect("Couldn't create the supervised children group.");
///
///     // ...or broadcast messages to all its supervised children
///     // and supervisors...
///     supervisor.broadcast("A message containing data.").expect("Couldn't broadcast the message.");
///
///     // ...and then can even be stopped or killed...
///     supervisor.stop().expect("Couldn't stop the supervisor");
///     // supervisor.kill().expect("Couldn't kill the supervisor");
///
///     // Creating a new top-level children group...
///     let children = Quip::children(|children| {
///         children
///         // ...containing a defined number of elements...
///             .with_redundancy(4)
///         // ...all executing a similar future...
///             .with_exec(|ctx: QuipContext| {
///                 async move {
///                     // ...receiving and matching messages...
///                     msg! { ctx.recv().await?,
///                         ref msg: &'static str => {
///                             // ...
///                         };
///                         msg: &'static str => {
///                             // ...
///                         };
///                         msg: &'static str =!> {
///                             // ...
///                         };
///                         // ...
///                         _: _ => ();
///                     }
///
///                     // ...
///
///                     Ok(())
///                 }
///             })
///     }).expect("Couldn't create the children group.");
///
///     // ...which can broadcast messages to all its elements...
///     children.broadcast("A message containing data.").expect("Couldn't broadcast the message.");
///
///     // ...and then can even be stopped or killed...
///     children.stop().expect("Couldn't stop the children group.");
///     // children.kill().expect("Couldn't kill the children group.");
///
///     // Create a new top-level children group and getting a list
///     // of reference to its elements...
///     let children = Quip::children(|children| {
///         // ...
///         # children.with_exec(|ctx: QuipContext| {
///         #   async move {
///         #       msg! { ctx.recv().await?,
///         #            _: _ => ();
///         #        }
///         #        Ok(())
///         #    }
///         # })
///     }).expect("Couldn't create the children group.");
///     let elems: &[ChildRef] = children.elems();
///
///     // ...to then get one of its elements' reference...
///     let child = &elems[0];
///
///     // ...to then "tell" it messages...
///     child.tell_anonymously("A message containing data.").expect("Couldn't send the message.");
///
///     // ...or "ask" it messages...
///     let answer: Answer = child.ask_anonymously("A message containing data.").expect("Couldn't send the message.");
///     # async {
///     // ...until the child eventually answers back...
///     let answer: Result<SignedMessage, ()> = run!(answer);
///     # };
///
///     // ...and then even stop or kill it...
///     child.stop().expect("Couldn't stop the child.");
///     // child.kill().expect("Couldn't kill the child.");
///
///     // Broadcasting a message to all the system's children...
///     Quip::broadcast("A message containing data.").expect("Couldn't send the message.");
///
///     // Stopping or killing the system...
///     Quip::stop();
///     // Quip::kill();
///
///     // Blocking until the system has stopped (or got killed)...
///     Quip::block_until_stopped();
/// }
/// ```
pub struct Quip {
    _priv: (),
}

impl Quip {
    /// Initializes the system if it hasn't already been done, using
    /// the default [`Config`].
    ///
    /// **It is required that you call `Quip::init` or
    /// [`Quip::init_with`] at least once before using any of
    /// quip's features.**
    ///
    /// # Example
    ///
    /// ```rust
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
    /// use quip::prelude::*;
    ///
    /// Quip::init();
    ///
    /// // You can now use quip...
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn init() {
        let config = Config::default();
        Quip::init_with(config)
    }

    /// Initializes the system if it hasn't already been done, using
    /// the specified [`Config`].
    ///
    /// **It is required that you call [`Quip::init`] or
    /// `Quip::init_with` at least once before using any of
    /// quip's features.**
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration used to initialize the system.
    ///
    /// # Example
    ///
    /// ```rust
    /// use quip::prelude::*;
    ///
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
    /// let config = Config::new()
    ///     .show_backtraces();
    ///
    /// Quip::init_with(config);
    ///
    /// // You can now use quip...
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn init_with(config: Config) {
        debug!("Quip: Initializing with config: {:?}", config);
        if config.backtraces().is_hide() {
            debug!("Quip: Hiding backtraces.");
            std::panic::set_hook(Box::new(|_| ()));
        }

        let _ = &SYSTEM;
    }

    /// Creates a new [`Supervisor`], passes it through the specified
    /// `init` closure and then sends it to the system for it to
    /// start supervising children.
    ///
    /// This method returns a [`SupervisorRef`] referencing the newly
    /// created supervisor if it succeeded, or `Err(())`
    /// otherwise.
    ///
    /// # Arguments
    ///
    /// * `init` - The closure taking the new [`Supervisor`] as an
    ///     argument and returning it once configured.
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
    /// let sp_ref: SupervisorRef = Quip::supervisor(|sp| {
    ///     // Configure the supervisor...
    ///     sp.with_strategy(SupervisionStrategy::OneForOne)
    ///     // ...and return it.
    /// }).expect("Couldn't create the supervisor.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn supervisor<S>(init: S) -> Result<SupervisorRef, ()>
    where
        S: FnOnce(Supervisor) -> Supervisor,
    {
        debug!("Quip: Creating supervisor.");
        let parent = Parent::system();
        let bcast = Broadcast::new(parent, QuipPathElement::Supervisor(QuipId::new()));

        debug!("Quip: Initializing Supervisor({}).", bcast.id());
        let supervisor = Supervisor::new(bcast);
        let supervisor = init(supervisor);
        debug!("Supervisor({}): Initialized.", supervisor.id());
        let supervisor_ref = supervisor.as_ref();

        debug!("Quip: Deploying Supervisor({}).", supervisor.id());
        let msg = QuipMessage::deploy_supervisor(supervisor);
        let envelope = Envelope::new(msg, SYSTEM.path().clone(), SYSTEM.sender().clone());
        trace!("Quip: Sending envelope: {:?}", envelope);
        SYSTEM.sender().unbounded_send(envelope).map_err(|_| ())?;

        Ok(supervisor_ref)
    }

    /// Creates a new [`Children`], passes it through the specified
    /// `init` closure and then sends it to the system's default
    /// supervisor for it to start supervising it.
    ///
    /// This methods returns a [`ChildrenRef`] referencing the newly
    /// created children group it it succeeded, or `Err(())`
    /// otherwise.
    ///
    /// Note that the "system supervisor" is a supervisor created
    /// by the system at startup.
    ///
    /// # Arguments
    ///
    /// * `init` - The closure taking the new [`Children`] as an
    ///     argument and returning it once configured.
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
    /// let children_ref: ChildrenRef = Quip::children(|children| {
    ///     // Configure the children group...
    ///     children.with_exec(|ctx: QuipContext| {
    ///         async move {
    ///             // Send and receive messages...
    ///             let opt_msg: Option<SignedMessage> = ctx.try_recv().await;
    ///             // ...and return `Ok(())` or `Err(())` when you are done...
    ///             Ok(())
    ///
    ///             // Note that if `Err(())` was returned, the supervisor would
    ///             // restart the children group.
    ///         }
    ///     })
    ///     // ...and return it.
    /// }).expect("Couldn't create the children group.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn children<C>(init: C) -> Result<ChildrenRef, ()>
    where
        C: FnOnce(Children) -> Children,
    {
        debug!("Quip: Creating children group.");
        SYSTEM.supervisor().children(init)
    }

    /// Creates a new [`Children`] which will have the given closure
    /// as action and then sends it to the system's default supervisor.
    ///
    /// This method returns a [`ChildrenRef`] referencing the newly created children
    /// if the creation was successful, otherwise returns an `Err(())`.
    ///
    /// Internally this method uses the [`Quip::children`] and [`Children::with_exec`] methods
    /// to create a new children.
    ///
    /// # Arguments
    /// * `action` - The closure which gets executed by the child.
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
    /// let children_ref: ChildrenRef = Quip::spawn(|ctx: quipContext| {
    ///     async move {
    ///         // ...
    ///         Ok(())
    ///     }
    /// }).expect("Couldn't create the children group.");
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn spawn<I, F>(action: I) -> Result<ChildrenRef, ()>
    where
        I: Fn(QuipContext) -> F + Send + 'static,
        F: Future<Output = Result<(), ()>> + Send + 'static,
    {
        Quip::children(|ch| ch.with_redundancy(1).with_exec(action))
    }
    distributed_api! {
        // FIXME!
        #[allow(missing_docs)]
        pub fn distributed<I, F>(cluster_config: &'static ArtilleryAPClusterConfig, action: I) -> Result<ChildrenRef, ()>
        where
            I: Fn(Arc<DistributedContext>) -> F + Send + Sync + 'static,
            F: Future<Output = Result<(), ()>> + Send + 'static,
        {
            cluster_actor(cluster_config, action)
        }
    }

    /// Sends a message to the system which will then send it to all
    /// the root-level supervisors and their supervised children and
    /// supervisors, etc.
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
    /// let msg = "A message containing data.";
    /// Quip::broadcast(msg).expect("Couldn't send the message.");
    ///
    ///     # Quip::children(|children| {
    ///         # children.with_exec(|ctx: QuipContext| {
    ///             # async move {
    /// // And then in every children groups's elements' future...
    /// msg! { ctx.recv().await?,
    ///     ref msg: &'static str => {
    ///         assert_eq!(msg, &"A message containing data.");
    ///     };
    ///     // We are only broadcasting a `&'static str` in this
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
    pub fn broadcast<M: Message>(msg: M) -> Result<(), M> {
        debug!("Quip: Broadcasting message: {:?}", msg);
        let msg = QuipMessage::broadcast(msg);
        let envelope = Envelope::from_dead_letters(msg);
        trace!("Quip: Sending envelope: {:?}", envelope);
        // FIXME: panics?
        SYSTEM
            .sender()
            .unbounded_send(envelope)
            .map_err(|err| err.into_inner().into_msg().unwrap())
    }

    /// Sends a message to the system to tell it to start
    /// handling messages and running children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use quip::prelude::*;
    ///
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
    /// Quip::init();
    ///
    /// // Use quip, spawn children and supervisors...
    ///
    /// Quip::start();
    ///
    /// // The system will soon start, messages will
    /// // now be handled...
    /// #
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn start() {
        debug!("Quip: Starting.");
        let msg = QuipMessage::start();
        let envelope = Envelope::from_dead_letters(msg);
        trace!("Quip: Sending envelope: {:?}", envelope);
        // FIXME: Err(Error)
        SYSTEM.sender().unbounded_send(envelope).ok();
    }

    /// Sends a message to the system to tell it to stop
    /// every running children groups and supervisors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use quip::prelude::*;
    ///
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
    ///
    /// Quip::init();
    ///
    /// // Use quip, spawn children and supervisors...
    ///
    /// Quip::start();
    ///
    /// // Send messages to children and/or do some
    /// // work until you decide to stop the system...
    ///
    /// Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn stop() {
        debug!("Quip: Stopping.");
        let msg = QuipMessage::stop();
        let envelope = Envelope::from_dead_letters(msg);
        trace!("Quip: Sending envelope: {:?}", envelope);
        // FIXME: Err(Error)
        SYSTEM.sender().unbounded_send(envelope).ok();
    }

    /// Sends a message to the system to tell it to kill every
    /// running children groups and supervisors
    ///
    /// # Example
    ///
    /// ```rust
    /// use quip::prelude::*;
    ///
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
    /// Quip::init();
    ///
    /// // Use quip, spawn children and supervisors...
    ///
    /// Quip::start();
    /// // Send messages to children and/or do some
    /// // work until you decide to kill the system...
    ///
    /// Quip::kill();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn kill() {
        debug!("Quip: Killing.");
        let msg = QuipMessage::kill();
        let envelope = Envelope::from_dead_letters(msg);
        trace!("Quip: Sending envelope: {:?}", envelope);
        // FIXME: Err(Error)
        SYSTEM.sender().unbounded_send(envelope).ok();

        let handle = SYSTEM.handle();
        let system = crate::executor::run(async { handle.lock().await.take() });
        if let Some(system) = system {
            debug!("Quip: Cancelling system handle.");
            system.cancel();
        }

        SYSTEM.notify_stopped();
    }

    /// Blocks the current thread until the system is stopped
    /// (either by calling [`Quip::stop`] or
    /// [`Quip::kill`]).
    ///
    /// # Example
    ///
    /// ```rust
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
    /// use quip::prelude::*;
    ///
    /// Quip::init();
    ///
    /// // Use quip, spawn children and supervisors...
    ///
    /// Quip::start();
    /// // Send messages to children and/or do some
    /// // work...
    ///
    /// # Quip::stop();
    /// Quip::block_until_stopped();
    /// // The system is now stopped. A child might have
    /// // stopped or killed it...
    /// # }
    /// ```
    pub fn block_until_stopped() {
        debug!("Quip: Blocking until system is stopped.");
        SYSTEM.wait_until_stopped();
    }
}

impl Debug for Quip {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Quip").finish()
    }
}
