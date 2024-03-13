//! A module that exposes the functions used under the hoods from `quip`s macros: `spawn!`, `run!`
//! and `blocking!`.
use std::future::Future;
pub use tinyproc::proc_stack::ProcStack;
use tinyproc::recoverable_handle::RecoverableHandle;

/// Spawns a blocking task, which will run on the blocking thread pool,
/// and returns the handle.
///
/// # Example
/// ```
/// # use std::{thread, time};
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
/// use quip::executor::blocking;
/// let task = blocking(async move {
///     thread::sleep(time::Duration::from_millis(3000));
/// });
/// # }
/// ```
pub fn blocking<F, R>(future: F) -> RecoverableHandle<R>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    quip_executor::blocking::spawn_blocking(future, tinyproc::proc_stack::ProcStack::default())
}

/// Block the current thread until passed
/// future is resolved with an output (including the panic).
///
/// # Example
/// ```
/// # use quip::prelude::*;
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
/// use quip::executor::run;
/// let future1 = async move {
///     123
/// };
///
/// run(async move {
///     let result = future1.await;
///     assert_eq!(result, 123);
/// });
///
/// let future2 = async move {
///     10 / 2
/// };
///
/// let result = run(future2);
/// assert_eq!(result, 5);
/// # }
/// ```
pub fn run<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    quip_executor::run::run(future, tinyproc::proc_stack::ProcStack::default())
}

/// Spawn a given future onto the executor from the global level.
///
/// # Example
/// ```
/// # use quip::prelude::*;
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
/// use quip::executor::{spawn, run};
/// let handle = spawn(async {
///     panic!("test");
/// });
/// run(handle);
/// # }
/// ```
pub fn spawn<F, T>(future: F) -> RecoverableHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    quip_executor::pool::spawn(future, tinyproc::proc_stack::ProcStack::default())
}
