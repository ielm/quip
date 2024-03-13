//!
//! A path represents a message sender's semantics and
//! later will be used to route messages to them

use crate::context::{QuipId, NIL_ID};
use std::fmt;
use std::result::Result;

#[derive(Clone)]
/// Represents a Path for a System, Supervisor, Children or Child.
///
/// QuipPath can be used to identify message senders.
/// Later it will be used to route messages to a path.
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
///
///     # Quip::children(|children| {
///         # children.with_exec(|ctx: QuipContext| {
///             # async move {
/// ctx.tell(&ctx.signature(), "Hello to myself").expect("Couldn't send a message");
/// msg! { ctx.recv().await?,
///     ref msg: &'static str => {
///         let path: &QuipPath = signature!().path();
///         assert_eq!(path.elem(), ctx.signature().path().elem());
///     };
///     // We are only sending a `&'static str` in this
///     // example, so we know that this won't happen...
///     _: _ => ();
/// }
///                 # Quip::stop();
///                 # Ok(())
///             # }
///         # })
///     # }).unwrap();
///     #
///     # Quip::start();
///     # Quip::block_until_stopped();
/// # }
/// ```
pub struct QuipPath {
    // TODO: possibly more effective collection depending on how we'll use it in routing
    parent_chain: Vec<QuipId>,
    this: Option<QuipPathElement>,
}

impl QuipPath {
    // SYSTEM or a sender out of Quip scope
    pub(crate) fn root() -> QuipPath {
        QuipPath {
            parent_chain: vec![],
            this: None,
        }
    }

    /// iterates over path elements
    pub(crate) fn iter(&self) -> impl Iterator<Item = &QuipId> {
        let parent_iter = self.parent_chain.iter();
        parent_iter.chain(self.this.iter().map(|e| e.id()))
    }

    /// Returns the last element's id.
    /// If it's root or a dead_letters then &NIL_ID is returned.
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
    ///         assert_eq!(path.id(), &NIL_ID);
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
    pub fn id(&self) -> &QuipId {
        self.this.as_ref().map(|e| e.id()).unwrap_or(&NIL_ID)
    }

    /// Returns a path element. If the path is root then None is returned.
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
    ///         assert!(path.elem().as_ref().unwrap().is_children());
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
    pub fn elem(&self) -> &Option<QuipPathElement> {
        &self.this
    }

    /// Checks whether `QuipPath` is a dead-letters path.
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
    pub fn is_dead_letters(&self) -> bool {
        self.parent_chain.len() == 2 && self.this.as_ref().map(|e| e.is_child()).unwrap_or(false)
    }
}

impl fmt::Display for QuipPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "/{}",
            self.iter()
                .map(|id| format!("{}", id))
                .collect::<Vec<String>>()
                .join("/")
        )
    }
}

impl fmt::Debug for QuipPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.this {
            Some(this @ QuipPathElement::Supervisor(_))
            | Some(this @ QuipPathElement::Children(_)) => write!(
                f,
                "/{}",
                self.parent_chain
                    .iter()
                    .map(|id| QuipPathElement::Supervisor(id.clone()))
                    .chain(vec![this.clone()])
                    .map(|el| format!("{:?}", el))
                    .collect::<Vec<String>>()
                    .join("/")
            ),
            Some(this @ QuipPathElement::Child(_)) => {
                let parent_len = self.parent_chain.len();

                write!(
                    f,
                    "/{}",
                    self.parent_chain
                        .iter()
                        .enumerate()
                        .map(|(i, id)| {
                            if i == parent_len - 1 {
                                QuipPathElement::Children(id.clone())
                            } else {
                                QuipPathElement::Supervisor(id.clone())
                            }
                        })
                        .chain(vec![this.clone()])
                        .map(|el| format!("{:?}", el))
                        .collect::<Vec<String>>()
                        .join("/")
                )
            }
            None => write!(f, "/"),
        }
    }
}

#[derive(Clone, PartialEq)]
/// Represents QuipPath element
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
///
/// Quip::children(|children| {
///     children.with_exec(|ctx: QuipContext| {
///         async move {
///             ctx.tell(&ctx.signature(), "Hello to myself");
///             
///             let msg: SignedMessage = ctx.recv().await?;
///             let elem: &Option<QuipPathElement> = msg.signature().path().elem();
///             assert!(elem.is_some());
///             assert_eq!(elem, ctx.signature().path().elem());
///
///             # Quip::stop();
///             Ok(())
///         }
///     })
/// }).expect("Couldn't create the children group.");
/// #
/// # Quip::start();
/// # Quip::block_until_stopped();
/// # }
/// ```
pub enum QuipPathElement {
    #[doc(hidden)]
    /// Supervisor element
    Supervisor(QuipId),
    #[doc(hidden)]
    /// Children element
    Children(QuipId),
    #[doc(hidden)]
    /// Child element
    Child(QuipId),
}

impl fmt::Debug for QuipPathElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuipPathElement::Supervisor(id) => write!(f, "supervisor#{}", id),
            QuipPathElement::Children(id) => write!(f, "children#{}", id),
            QuipPathElement::Child(id) => write!(f, "child#{}", id),
        }
    }
}

impl QuipPathElement {
    pub(crate) fn id(&self) -> &QuipId {
        match self {
            QuipPathElement::Supervisor(id) => id,
            QuipPathElement::Children(id) => id,
            QuipPathElement::Child(id) => id,
        }
    }

    #[doc(hidden)]
    /// Checks whether the QuipPath identifies a supervisor.
    pub fn is_supervisor(&self) -> bool {
        matches!(self, QuipPathElement::Supervisor(_))
    }

    #[doc(hidden)]
    /// Checks whether the QuipPath identifies children.
    pub fn is_children(&self) -> bool {
        matches!(self, QuipPathElement::Children(_))
    }

    /// Checks whether the QuipPath identifies a child.
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
    ///
    /// Quip::children(|children| {
    ///     children.with_exec(|ctx: QuipContext| {
    ///         async move {
    ///             ctx.tell(&ctx.signature(), "Hello to myself");
    ///             
    ///             let msg: SignedMessage = ctx.recv().await?;
    ///             let elem: &Option<QuipPathElement> = msg.signature().path().elem();
    ///             assert!(elem.is_some());
    ///             assert_eq!(elem, ctx.signature().path().elem());
    ///
    ///             # Quip::stop();
    ///             Ok(())
    ///         }
    ///     })
    /// }).expect("Couldn't create the children group.");
    /// #
    /// # Quip::start();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn is_child(&self) -> bool {
        matches!(self, QuipPathElement::Child(_))
    }
}

#[derive(Clone)]
pub(crate) struct AppendError {
    path: QuipPath,
    element: QuipPathElement,
}

impl fmt::Display for AppendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.element {
            QuipPathElement::Supervisor(..) => match self.path.this {
                None => unreachable!(),
                Some(QuipPathElement::Supervisor(..)) => unreachable!(),
                Some(QuipPathElement::Children(..)) => {
                    write!(f, "Supervisor is not appendable to children")
                }
                Some(QuipPathElement::Child(..)) => {
                    write!(f, "Supervisor is not appendable to a child")
                }
            },
            QuipPathElement::Children(..) => match self.path.this {
                None => write!(f, "Children is not appendable to root"),
                Some(QuipPathElement::Supervisor(..)) => unreachable!(),
                Some(QuipPathElement::Children(..)) => {
                    write!(f, "Children is not appendable to children")
                }
                Some(QuipPathElement::Child(..)) => {
                    write!(f, "Children is not appendable to a child")
                }
            },
            QuipPathElement::Child(..) => match self.path.this {
                None => write!(f, "Child is not appendable to root"),
                Some(QuipPathElement::Supervisor(..)) => {
                    write!(f, "Child is not appendable to a supervisor")
                }
                Some(QuipPathElement::Children(..)) => unreachable!(),
                Some(QuipPathElement::Child(..)) => {
                    write!(f, "Child is not appendable to a child")
                }
            },
        }
    }
}

impl fmt::Debug for AppendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Can't append {:?} to {:?}", self.element, self.path)
    }
}

impl QuipPath {
    pub(crate) fn append(self, el: QuipPathElement) -> Result<QuipPath, AppendError> {
        match el {
            sv @ QuipPathElement::Supervisor(_) => match self.this {
                None => Ok(QuipPath {
                    parent_chain: self.parent_chain,
                    this: Some(sv),
                }),
                Some(QuipPathElement::Supervisor(id)) => {
                    let mut path = QuipPath {
                        parent_chain: self.parent_chain,
                        this: Some(sv),
                    };
                    path.parent_chain.push(id);
                    Ok(path)
                }
                this => Err(AppendError {
                    path: QuipPath {
                        parent_chain: self.parent_chain,
                        this,
                    },
                    element: sv,
                }),
            },
            children @ QuipPathElement::Children(_) => match self.this {
                Some(QuipPathElement::Supervisor(id)) => {
                    let mut path = QuipPath {
                        parent_chain: self.parent_chain,
                        this: Some(children),
                    };
                    path.parent_chain.push(id);
                    Ok(path)
                }
                this => Err(AppendError {
                    path: QuipPath {
                        parent_chain: self.parent_chain,
                        this,
                    },
                    element: children,
                }),
            },
            child @ QuipPathElement::Child(_) => match self.this {
                Some(QuipPathElement::Children(id)) => {
                    let mut path = QuipPath {
                        parent_chain: self.parent_chain,
                        this: Some(child),
                    };
                    path.parent_chain.push(id);
                    Ok(path)
                }
                this => Err(AppendError {
                    path: QuipPath {
                        parent_chain: self.parent_chain,
                        this,
                    },
                    element: child,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SYSTEM + smth

    #[test]
    fn append_sv_to_system() {
        let sv_id = QuipId::new();
        let path = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id.clone()))
            .unwrap();
        assert_eq!(path.iter().collect::<Vec<&QuipId>>(), vec![&sv_id]);
    }

    #[test]
    fn append_children_to_system() {
        let sv_id = QuipId::new();
        let res = QuipPath::root().append(QuipPathElement::Children(sv_id));
        assert_eq!(
            res.unwrap_err().to_string(),
            "Children is not appendable to root"
        );
    }

    #[test]
    fn append_child_to_system() {
        let sv_id = QuipId::new();
        let res = QuipPath::root().append(QuipPathElement::Child(sv_id));
        assert_eq!(
            res.unwrap_err().to_string(),
            "Child is not appendable to root"
        );
    }

    // Supervisor + smth

    #[test]
    fn append_sv_to_sv() {
        let sv1_id = QuipId::new();
        let sv2_id = QuipId::new();
        let path = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv1_id.clone()))
            .unwrap()
            .append(QuipPathElement::Supervisor(sv2_id.clone()))
            .unwrap();
        assert_eq!(
            path.iter().collect::<Vec<&QuipId>>(),
            vec![&sv1_id, &sv2_id]
        );
    }

    #[test]
    fn append_children_to_sv() {
        let sv_id = QuipId::new();
        let children_id = QuipId::new();
        let path = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id.clone()))
            .unwrap()
            .append(QuipPathElement::Children(children_id.clone()))
            .unwrap();
        assert_eq!(
            path.iter().collect::<Vec<&QuipId>>(),
            vec![&sv_id, &children_id]
        );
    }

    #[test]
    fn append_child_to_sv() {
        let sv_id = QuipId::new();
        let children_id = QuipId::new();
        let res = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id))
            .unwrap()
            .append(QuipPathElement::Child(children_id));
        assert_eq!(
            res.unwrap_err().to_string(),
            "Child is not appendable to a supervisor"
        );
    }

    // children + smth

    #[test]
    fn append_sv_to_children() {
        let sv_id = QuipId::new();
        let children_id = QuipId::new();
        let res = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id))
            .unwrap()
            .append(QuipPathElement::Children(children_id))
            .unwrap()
            .append(QuipPathElement::Supervisor(QuipId::new()));
        assert_eq!(
            res.unwrap_err().to_string(),
            "Supervisor is not appendable to children"
        );
    }

    #[test]
    fn append_children_to_children() {
        let sv_id = QuipId::new();
        let children_id = QuipId::new();
        let res = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id))
            .unwrap()
            .append(QuipPathElement::Children(children_id))
            .unwrap()
            .append(QuipPathElement::Children(QuipId::new()));
        assert_eq!(
            res.unwrap_err().to_string(),
            "Children is not appendable to children"
        );
    }

    #[test]
    fn append_child_to_children() {
        let sv_id = QuipId::new();
        let children_id = QuipId::new();
        let child_id = QuipId::new();
        let path = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id.clone()))
            .unwrap()
            .append(QuipPathElement::Children(children_id.clone()))
            .unwrap()
            .append(QuipPathElement::Child(child_id.clone()))
            .unwrap();
        assert_eq!(
            path.iter().collect::<Vec<&QuipId>>(),
            vec![&sv_id, &children_id, &child_id]
        );
    }

    // child + smth

    #[test]
    fn append_sv_to_child() {
        let sv_id = QuipId::new();
        let children_id = QuipId::new();
        let child_id = QuipId::new();
        let res = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id))
            .unwrap()
            .append(QuipPathElement::Children(children_id))
            .unwrap()
            .append(QuipPathElement::Child(child_id))
            .unwrap()
            .append(QuipPathElement::Supervisor(QuipId::new()));
        assert_eq!(
            res.unwrap_err().to_string(),
            "Supervisor is not appendable to a child"
        );
    }

    #[test]
    fn append_children_to_child() {
        let sv_id = QuipId::new();
        let children_id = QuipId::new();
        let child_id = QuipId::new();
        let res = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id))
            .unwrap()
            .append(QuipPathElement::Children(children_id))
            .unwrap()
            .append(QuipPathElement::Child(child_id))
            .unwrap()
            .append(QuipPathElement::Children(QuipId::new()));
        assert_eq!(
            res.unwrap_err().to_string(),
            "Children is not appendable to a child"
        );
    }

    #[test]
    fn append_child_to_child() {
        let sv_id = QuipId::new();
        let children_id = QuipId::new();
        let child_id = QuipId::new();
        let res = QuipPath::root()
            .append(QuipPathElement::Supervisor(sv_id))
            .unwrap()
            .append(QuipPathElement::Children(children_id))
            .unwrap()
            .append(QuipPathElement::Child(child_id))
            .unwrap()
            .append(QuipPathElement::Child(QuipId::new()));
        assert_eq!(
            res.unwrap_err().to_string(),
            "Child is not appendable to a child"
        );
    }
}
