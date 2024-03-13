#[derive(Default, Debug, Clone)]
/// The configuration that should be used to initialize the
/// system using [`Quip::init_with`].
///
/// The default behaviors are the following:
/// - All backtraces are shown (see [`Config::show_backtraces`]).
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
/// let config = Config::new().show_backtraces();
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
///
/// [`Quip::init_with`]: crate::quip::init_with
pub struct Config {
    backtraces: Backtraces,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum Backtraces {
    /// Shows all backtraces, like an application without
    /// Quip would.
    Show,
    // TODO: Catch,
    /// Hides all backtraces.
    Hide,
}

impl Config {
    /// Creates a new configuration with the following default
    /// behaviors:
    /// - All backtraces are shown (see [`Config::show_backtraces`]).
    pub fn new() -> Self {
        Config::default()
    }

    /// Makes Quip show all backtraces, like an application
    /// without it would. This can be useful when trying to
    /// debug children panicking.
    ///
    /// Note that this is the default behavior.
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
    /// let config = Config::new().show_backtraces();
    ///
    /// Quip::init_with(config);
    ///
    /// // You can now use quip and it will show you the
    /// // backtraces of panics...
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn show_backtraces(mut self) -> Self {
        self.backtraces = Backtraces::show();
        self
    }

    /// Makes Quip hide all backtraces.
    ///
    /// Note that the default behavior is to show all backtraces
    /// (see [`Config::show_backtraces`]).
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
    /// let config = Config::new().hide_backtraces();
    ///
    /// Quip::init_with(config);
    ///
    /// // You can now use quip and no panic backtraces
    /// // will be shown...
    /// #
    /// # Quip::start();
    /// # Quip::stop();
    /// # Quip::block_until_stopped();
    /// # }
    /// ```
    pub fn hide_backtraces(mut self) -> Self {
        self.backtraces = Backtraces::hide();
        self
    }

    pub(crate) fn backtraces(&self) -> &Backtraces {
        &self.backtraces
    }
}

impl Backtraces {
    fn show() -> Self {
        Backtraces::Show
    }

    fn hide() -> Self {
        Backtraces::Hide
    }

    pub(crate) fn is_hide(&self) -> bool {
        self == &Backtraces::Hide
    }
}

impl Default for Backtraces {
    fn default() -> Self {
        Backtraces::Show
    }
}
