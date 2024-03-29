use crate::children_ref::ChildrenRef;
use crate::context::QuipId;
use crate::envelope::Envelope;
use crate::message::QuipMessage;
use crate::path::{QuipPath, QuipPathElement};
use crate::supervisor::SupervisorRef;
use crate::system::SYSTEM;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::prelude::*;
use fxhash::FxHashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub(crate) type Sender = UnboundedSender<Envelope>;
pub(crate) type Receiver = UnboundedReceiver<Envelope>;

#[derive(Debug)]
pub(crate) struct Broadcast {
    sender: Sender,
    recver: Receiver,
    path: Arc<QuipPath>, // Arc is needed because we put path to Envelope
    parent: Parent,
    children: FxHashMap<QuipId, Sender>,
}

#[derive(Debug, Clone)]
pub(crate) enum Parent {
    None,
    System,
    Supervisor(SupervisorRef),
    Children(ChildrenRef),
}

impl Parent {
    pub(super) fn is_none(&self) -> bool {
        matches!(self, Parent::None)
    }

    pub(super) fn is_system(&self) -> bool {
        matches!(self, Parent::System)
    }
}

impl Broadcast {
    pub(crate) fn new(parent: Parent, element: QuipPathElement) -> Self {
        let (sender, recver) = mpsc::unbounded();
        let children = FxHashMap::default();

        let parent_path: QuipPath = match &parent {
            Parent::None | Parent::System => QuipPath::root(),
            Parent::Supervisor(sv_ref) => QuipPath::clone(sv_ref.path()),
            Parent::Children(ch_ref) => QuipPath::clone(ch_ref.path()),
        };

        // FIXME: unwrap
        let path = parent_path
            .append(element)
            .expect("Can't append path in Broadcast::new");
        let path = Arc::new(path);

        Broadcast {
            parent,
            sender,
            recver,
            path,
            children,
        }
    }

    pub(crate) fn new_root(parent: Parent) -> Self {
        // FIXME
        assert!(parent.is_none() || parent.is_system());

        let (sender, recver) = mpsc::unbounded();
        let children = FxHashMap::default();
        let path = QuipPath::root();
        let path = Arc::new(path);

        Broadcast {
            parent,
            sender,
            recver,
            path,
            children,
        }
    }

    pub(crate) fn id(&self) -> &QuipId {
        self.path.id()
    }

    pub(crate) fn sender(&self) -> &Sender {
        &self.sender
    }

    pub(crate) fn path(&self) -> &Arc<QuipPath> {
        &self.path
    }

    pub(crate) fn parent(&self) -> &Parent {
        &self.parent
    }

    pub(crate) fn register(&mut self, child: &Self) {
        self.children
            .insert(child.id().clone(), child.sender.clone());
    }

    pub(crate) fn unregister(&mut self, id: &QuipId) {
        self.children.remove(id);
    }

    pub(crate) fn clear_children(&mut self) {
        self.children.clear();
    }

    pub(crate) fn stop_child(&mut self, id: &QuipId) {
        let msg = QuipMessage::stop();
        let env = Envelope::new(msg, self.path.clone(), self.sender.clone());
        self.send_child(id, env);

        self.unregister(id);
    }

    pub(crate) fn stop_children(&mut self) {
        let msg = QuipMessage::stop();
        let env = Envelope::new(msg, self.path.clone(), self.sender.clone());
        self.send_children(env);

        self.clear_children();
    }

    pub(crate) fn kill_child(&mut self, id: &QuipId) {
        let msg = QuipMessage::kill();
        let env = Envelope::new(msg, self.path.clone(), self.sender.clone());
        self.send_child(id, env);

        self.unregister(id);
    }

    pub(crate) fn kill_children(&mut self) {
        let msg = QuipMessage::kill();
        let env = Envelope::new(msg, self.path.clone(), self.sender.clone());
        self.send_children(env);

        self.clear_children();
    }

    pub(crate) fn stopped(&mut self) {
        self.stop_children();

        let msg = QuipMessage::stopped(self.id().clone());
        let env = Envelope::new(msg, self.path.clone(), self.sender.clone());
        // FIXME: Err(msg)
        self.send_parent(env).ok();
    }

    pub(crate) fn faulted(&mut self) {
        self.kill_children();

        let msg = QuipMessage::faulted(self.id().clone());
        let env = Envelope::new(msg, self.path.clone(), self.sender.clone());
        // FIXME: Err(msg)
        self.send_parent(env).ok();
    }

    pub(crate) fn send_parent(&self, envelope: Envelope) -> Result<(), Envelope> {
        self.parent.send(envelope)
    }

    pub(crate) fn send_child(&self, id: &QuipId, envelope: Envelope) {
        // FIXME: Err if None?
        if let Some(child) = self.children.get(id) {
            // FIXME: handle errors
            child.unbounded_send(envelope).ok();
        }
    }

    pub(crate) fn send_children(&self, env: Envelope) {
        for child in self.children.values() {
            // FIXME: Err(Error) if None
            if let Some(env) = env.try_clone() {
                // FIXME: handle errors
                child.unbounded_send(env).ok();
            }
        }
    }

    pub(crate) fn send_self(&self, env: Envelope) {
        // FIXME: handle errors
        self.sender.unbounded_send(env).ok();
    }
}

impl Parent {
    pub(crate) fn none() -> Self {
        Parent::None
    }

    pub(crate) fn system() -> Self {
        Parent::System
    }

    pub(crate) fn supervisor(supervisor: SupervisorRef) -> Self {
        Parent::Supervisor(supervisor)
    }

    pub(crate) fn children(children: ChildrenRef) -> Self {
        Parent::Children(children)
    }

    pub(crate) fn into_supervisor(self) -> Option<SupervisorRef> {
        if let Parent::Supervisor(supervisor) = self {
            Some(supervisor)
        } else {
            None
        }
    }

    pub(crate) fn into_children(self) -> Option<ChildrenRef> {
        if let Parent::Children(children) = self {
            Some(children)
        } else {
            None
        }
    }

    fn send(&self, env: Envelope) -> Result<(), Envelope> {
        match self {
            // FIXME
            Parent::None => unimplemented!(),
            Parent::System => SYSTEM
                .sender()
                .unbounded_send(env)
                .map_err(|err| err.into_inner()),
            Parent::Supervisor(supervisor) => supervisor.send(env),
            Parent::Children(children) => children.send(env),
        }
    }
}

impl Stream for Broadcast {
    type Item = Envelope;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().recver).poll_next(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::{Broadcast, Parent, QuipMessage};
    use crate::context::{QuipId, NIL_ID};
    use crate::envelope::Envelope;
    use crate::path::{QuipPath, QuipPathElement};
    use futures::channel::mpsc;
    use futures::executor;
    use futures::poll;
    use futures::prelude::*;
    use std::sync::Arc;
    use std::task::Poll;

    #[test]
    fn send_children() {
        let mut parent = Broadcast::new_root(Parent::System);

        let mut children = vec![];
        for _ in 0..4 {
            let child = Broadcast::new(Parent::System, QuipPathElement::Supervisor(QuipId::new()));
            parent.register(&child);
            children.push(child);
        }

        let msg = QuipMessage::start();

        // need manual construction because SYSTEM is not running in this test
        let (sender, _) = mpsc::unbounded();
        let env = Envelope::new(
            msg,
            Arc::new(
                QuipPath::root()
                    .append(QuipPathElement::Supervisor(NIL_ID))
                    .unwrap()
                    .append(QuipPathElement::Children(NIL_ID))
                    .unwrap(),
            ),
            sender,
        );

        parent.send_children(env.try_clone().unwrap());
        executor::block_on(async {
            for child in &mut children {
                match poll!(child.next()) {
                    Poll::Ready(Some(Envelope {
                        msg: QuipMessage::Start,
                        ..
                    })) => (),
                    _ => panic!(),
                }
            }
        });

        parent.unregister(children[0].id());
        parent.send_children(env.try_clone().unwrap());
        executor::block_on(async {
            assert!(poll!(children[0].next()).is_pending());

            for child in &mut children[1..] {
                match poll!(child.next()) {
                    Poll::Ready(Some(Envelope {
                        msg: QuipMessage::Start,
                        ..
                    })) => (),
                    _ => panic!(),
                }
            }
        });

        parent.clear_children();
        parent.send_children(env);
        executor::block_on(async {
            for child in &mut children[1..] {
                assert!(poll!(child.next()).is_pending());
            }
        });
    }
}
