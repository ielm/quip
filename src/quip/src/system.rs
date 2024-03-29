use crate::broadcast::{Broadcast, Parent, Sender};
use crate::children_ref::ChildrenRef;
use crate::context::{QuipContext, QuipId, NIL_ID};
use crate::dispatcher::GlobalDispatcher;
use crate::envelope::Envelope;
use crate::message::{Deployment, QuipMessage};
use crate::path::{QuipPath, QuipPathElement};
use crate::supervisor::{Supervisor, SupervisorRef};
use async_mutex::Mutex as AsyncMutex;
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use futures::{pending, poll};
use fxhash::{FxHashMap, FxHashSet};
use lasso::ThreadedRodeo;
use once_cell::sync::Lazy;
use quip_executor::pool;
use std::sync::{Arc, Condvar, Mutex};
use std::task::Poll;
use tinyproc::prelude::*;
use tracing::{debug, error, info, trace, warn};

pub(crate) static STRING_INTERNER: Lazy<Arc<ThreadedRodeo>> =
    Lazy::new(|| Arc::new(Default::default()));

pub(crate) static SYSTEM: Lazy<GlobalSystem> = Lazy::new(System::init);

pub(crate) struct GlobalSystem {
    sender: Sender,
    supervisor: SupervisorRef,
    dead_letters: ChildrenRef,
    path: Arc<QuipPath>,
    handle: Arc<AsyncMutex<Option<RecoverableHandle<()>>>>,
    running: Mutex<bool>,
    stopping_cvar: Condvar,
    dispatcher: GlobalDispatcher,
}

#[derive(Debug)]
struct System {
    bcast: Broadcast,
    launched: FxHashMap<QuipId, RecoverableHandle<Supervisor>>,
    // TODO: set limit
    restart: FxHashSet<QuipId>,
    waiting: FuturesUnordered<RecoverableHandle<Supervisor>>,
    pre_start_msgs: Vec<Envelope>,
    started: bool,
}

#[allow(clippy::mutex_atomic)]
impl GlobalSystem {
    fn new(
        sender: Sender,
        supervisor: SupervisorRef,
        dead_letters: ChildrenRef,
        handle: RecoverableHandle<()>,
    ) -> Self {
        let handle = Some(handle);
        let handle = Arc::new(AsyncMutex::new(handle));
        let path = Arc::new(QuipPath::root());
        let running = Mutex::new(true);
        let stopping_cvar = Condvar::new();
        let dispatcher = GlobalDispatcher::new();

        GlobalSystem {
            sender,
            supervisor,
            dead_letters,
            path,
            handle,
            running,
            stopping_cvar,
            dispatcher,
        }
    }

    pub(crate) fn sender(&self) -> &Sender {
        &self.sender
    }

    pub(crate) fn supervisor(&self) -> &SupervisorRef {
        &self.supervisor
    }

    pub(crate) fn dead_letters(&self) -> &ChildrenRef {
        &self.dead_letters
    }

    pub(crate) fn handle(&self) -> Arc<AsyncMutex<Option<RecoverableHandle<()>>>> {
        self.handle.clone()
    }

    pub(crate) fn path(&self) -> &Arc<QuipPath> {
        &self.path
    }

    pub(crate) fn dispatcher(&self) -> &GlobalDispatcher {
        &self.dispatcher
    }

    pub(crate) fn notify_stopped(&self) {
        // FIXME: panics
        *self.running.lock().unwrap() = false;
        self.stopping_cvar.notify_all();
    }

    pub(crate) fn wait_until_stopped(&self) {
        // FIXME: panics
        let mut running = self.running.lock().unwrap();
        while *running {
            running = self.stopping_cvar.wait(running).unwrap();
        }
    }
}

impl System {
    fn init() -> GlobalSystem {
        info!("System: Initializing.");
        let parent = Parent::none();
        let bcast = Broadcast::new_root(parent);
        let launched = FxHashMap::default();
        let restart = FxHashSet::default();
        let waiting = FuturesUnordered::new();
        let pre_start_msgs = Vec::new();
        let started = false;

        let sender = bcast.sender().clone();

        let system = System {
            bcast,
            launched,
            restart,
            waiting,
            pre_start_msgs,
            started,
        };

        debug!("System: Creating the system supervisor.");
        let parent = Parent::system();
        let bcast = Broadcast::new(parent, QuipPathElement::Supervisor(NIL_ID));

        let supervisor = Supervisor::system(bcast);
        let supervisor_ref = supervisor.as_ref();

        let msg = QuipMessage::deploy_supervisor(supervisor);
        let env = Envelope::new(
            msg,
            system.bcast.path().clone(),
            system.bcast.sender().clone(),
        );
        system.bcast.send_self(env);

        debug!("System: Launching.");
        let stack = system.stack();
        let handle = pool::spawn(system.run(), stack);

        let dead_letters_ref =
            Self::spawn_dead_letters(&supervisor_ref).expect("Can't spawn dead letters");

        GlobalSystem::new(sender, supervisor_ref, dead_letters_ref, handle)
    }

    fn stack(&self) -> ProcStack {
        // FIXME: with_id
        ProcStack::default()
    }

    fn spawn_dead_letters(root_sv: &SupervisorRef) -> Result<ChildrenRef, ()> {
        root_sv.children_with_id(NIL_ID, |children| {
            children.with_exec(|ctx: QuipContext| async move {
                loop {
                    let smsg = ctx.recv().await?;
                    debug!("Received dead letter: {:?}", smsg);
                }
            })
        })
    }

    // TODO: set a limit?
    async fn recover(&mut self, mut supervisor: Supervisor) {
        warn!("System: Recovering Supervisor({}).", supervisor.id());
        supervisor.callbacks().before_restart();

        let parent = Parent::system();
        let bcast = if supervisor.id() == &NIL_ID {
            None
        } else {
            Some(Broadcast::new(
                parent,
                QuipPathElement::Supervisor(QuipId::new()),
            ))
        };

        supervisor.reset(bcast).await;
        supervisor.callbacks().after_restart();

        self.bcast.register(supervisor.bcast());

        info!("System: Launching Supervisor({}).", supervisor.id());
        let id = supervisor.id().clone();
        let launched = supervisor.launch();
        self.launched.insert(id, launched);
    }

    async fn stop(&mut self) -> Vec<Supervisor> {
        self.bcast.stop_children();

        for (_, launched) in self.launched.drain() {
            self.waiting.push(launched);
        }

        let mut supervisors = Vec::new();
        loop {
            match poll!(&mut self.waiting.next()) {
                Poll::Ready(Some(Some(supervisor))) => {
                    debug!("System: Supervisor({}) stopped.", supervisor.id());
                    supervisors.push(supervisor);
                }
                Poll::Ready(Some(None)) => {
                    error!("System: Unknown supervisor cancelled instead of stopped.");
                }
                Poll::Ready(None) => return supervisors,
                Poll::Pending => pending!(),
            }
        }
    }

    async fn kill(&mut self) {
        self.bcast.kill_children();

        for launched in self.waiting.iter_mut() {
            launched.cancel();
        }

        for (_, launched) in self.launched.drain() {
            launched.cancel();

            self.waiting.push(launched);
        }

        loop {
            match poll!(&mut self.waiting.next()) {
                Poll::Ready(Some(Some(supervisor))) => {
                    debug!("System: Supervisor({}) killed.", supervisor.id());
                }
                Poll::Ready(Some(None)) => {
                    debug!("System: Unknown Supervisor killed.");
                }
                Poll::Ready(None) => return,
                Poll::Pending => pending!(),
            }
        }
    }

    async fn deploy(&mut self, deployment: Box<Deployment>) {
        match *deployment {
            Deployment::Supervisor(supervisor) => {
                debug!("System: Deploying Supervisor({}).", supervisor.id());
                supervisor.callbacks().before_start();

                self.bcast.register(supervisor.bcast());
                if self.started {
                    let msg = QuipMessage::start();
                    let envelope =
                        Envelope::new(msg, self.bcast.path().clone(), self.bcast.sender().clone());
                    self.bcast.send_child(supervisor.id(), envelope);
                }

                info!("System: Launching Supervisor({}).", supervisor.id());
                let id = supervisor.id().clone();
                let launched = supervisor.launch();
                self.launched.insert(id, launched);
            }
            // FIXME
            Deployment::Children(_) => unimplemented!(),
        }
    }

    async fn prune_supervised_object(&mut self, id: QuipId) {
        // TODO: Err if None?
        if let Some(launched) = self.launched.remove(&id) {
            // TODO: stop or kill?
            self.bcast.kill_child(&id);
            self.waiting.push(launched);
        }
    }

    fn restart_supervised_object(&mut self, id: QuipId) {
        // TODO: Err if None?
        if let Some(launched) = self.launched.remove(&id) {
            warn!("System: Supervisor({}) faulted.", id);
            self.waiting.push(launched);
            self.restart.insert(id);
        }
    }

    async fn handle(&mut self, env: Envelope) -> Result<(), ()> {
        match env {
            Envelope {
                msg: QuipMessage::Start,
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::Stop,
                ..
            } => {
                info!("System: Stopping.");
                for supervisor in self.stop().await {
                    supervisor.callbacks().after_stop();
                }

                return Err(());
            }
            Envelope {
                msg: QuipMessage::Kill,
                ..
            } => {
                info!("System: Killing.");
                self.kill().await;

                return Err(());
            }
            Envelope {
                msg: QuipMessage::Deploy(deployment),
                ..
            } => self.deploy(deployment).await,
            Envelope {
                msg: QuipMessage::Prune { id },
                ..
            } => self.prune_supervised_object(id).await,
            // FIXME
            Envelope {
                msg: QuipMessage::SuperviseWith(_),
                ..
            } => unimplemented!(),
            Envelope {
                msg: QuipMessage::ApplyCallback { .. },
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::InstantiatedChild { .. },
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::Message(ref message),
                ..
            } => {
                debug!("System: Broadcasting a message: {:?}", message);
                self.bcast.send_children(env);
            }
            Envelope {
                msg: QuipMessage::RestartRequired { .. },
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::RestartSubtree,
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::RestoreChild { .. },
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::FinishedChild { .. },
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::DropChild { .. },
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::SetState { .. },
                ..
            } => unreachable!(),
            Envelope {
                msg: QuipMessage::Stopped { id, .. },
                ..
            } => self.restart_supervised_object(id),
            Envelope {
                msg: QuipMessage::Faulted { id, .. },
                ..
            } => self.restart_supervised_object(id),
            Envelope {
                msg: QuipMessage::Heartbeat,
                ..
            } => unreachable!(),
        }

        Ok(())
    }

    async fn run(mut self) {
        info!("System: Launched.");
        loop {
            match poll!(&mut self.waiting.next()) {
                Poll::Ready(Some(Some(supervisor))) => {
                    let id = supervisor.id();
                    self.bcast.unregister(id);

                    if self.restart.remove(id) {
                        self.recover(supervisor).await;
                    } else {
                        supervisor.callbacks().after_stop();
                    }

                    continue;
                }
                // FIXME
                Poll::Ready(Some(None)) => unimplemented!(),
                Poll::Ready(None) | Poll::Pending => (),
            }

            match poll!(&mut self.bcast.next()) {
                // TODO: Err if started == true?
                Poll::Ready(Some(Envelope {
                    msg: QuipMessage::Start,
                    ..
                })) => {
                    trace!(
                        "System: Received a new message (started=false): {:?}",
                        QuipMessage::Start
                    );
                    info!("System: Starting.");
                    self.started = true;

                    let msg = QuipMessage::start();
                    let env =
                        Envelope::new(msg, self.bcast.path().clone(), self.bcast.sender().clone());
                    self.bcast.send_children(env);

                    let msgs = self.pre_start_msgs.drain(..).collect::<Vec<_>>();
                    self.pre_start_msgs.shrink_to_fit();

                    debug!("System: Replaying messages received before starting.");
                    for msg in msgs {
                        trace!("System: Replaying message: {:?}", msg);
                        // FIXME: Err(Error)?
                        if self.handle(msg).await.is_err() {
                            let handle = SYSTEM.handle();
                            let mut system = handle.lock().await;
                            *system = None;

                            SYSTEM.notify_stopped();

                            return;
                        }
                    }
                }
                Poll::Ready(Some(msg)) if !self.started => {
                    trace!("System: Received a new message (started=false): {:?}", msg);
                    self.pre_start_msgs.push(msg);
                }
                Poll::Ready(Some(msg)) => {
                    trace!("System: Received a new message (started=true): {:?}", msg);
                    if self.handle(msg).await.is_err() {
                        let handle = SYSTEM.handle();
                        let mut system = handle.lock().await;
                        *system = None;

                        SYSTEM.notify_stopped();

                        return;
                    }
                }
                // NOTE: because `Broadcast` always holds both a `Sender` and
                //      `Receiver` of the same channel, this would only be
                //      possible if the channel was closed, which never happens.
                Poll::Ready(None) => unreachable!(),
                Poll::Pending => pending!(),
            }
        }
    }
}
