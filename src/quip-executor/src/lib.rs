//! Quip Executor is NUMA-aware SMP based Fault-tolerant Executor
//!
//! Quip Executor is a highly-available, fault-tolerant, async communication
//! oriented executor. Quip's main idea is supplying a fully async runtime
//! with fault-tolerance to work on heavy loads.
//!
//! Main differences between other executors are:
//! * Uses SMP based execution scheme to exploit cache affinity on multiple cores and execution is
//! equally distributed over the system resources, which means utilizing the all system.
//! * Uses NUMA-aware allocation for scheduler's queues and exploit locality on server workloads.
//! * Tailored for creating middleware and working with actor model like concurrency and distributed communication.
//!
//! **NOTE:** Quip Executor is independent of it's framework implementation.
//! It uses [tinyproc] to encapsulate and provide fault-tolerance to your future based workloads.
//! You can use your futures with [tinyproc] to run your workloads on Quip Executor without the need to have framework.
//!
//! [tinyproc]: https://docs.rs/lightproc
//!

// Discarded lints
#![allow(clippy::if_same_then_else)]
// Force missing implementations
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod blocking;
pub mod load_balancer;
pub mod placement;
pub mod pool;
pub mod run;
pub mod run_queue;
pub mod sleepers;
mod thread_manager;
pub mod worker;

/// Prelude of Quip Executor
pub mod prelude {
    // pub use crate::blocking::*;
    pub use crate::pool::*;
    pub use crate::run::*;
}
