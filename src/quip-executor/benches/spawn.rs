#![feature(test)]

extern crate test;

use futures_timer::Delay;
use quip_executor::prelude::spawn;
use std::time::Duration;
use test::Bencher;
use tinyproc::proc_stack::ProcStack;

#[cfg(feature = "tokio-runtime")]
mod tokio_benchs {
    use super::*;
    #[bench]
    fn spawn_lot(b: &mut Bencher) {
        tokio_test::block_on(async { _spawn_lot(b) });
    }
    #[bench]
    fn spawn_single(b: &mut Bencher) {
        tokio_test::block_on(async {
            _spawn_single(b);
        });
    }
}

#[cfg(not(feature = "tokio-runtime"))]
mod no_tokio_benchs {
    use super::*;
    #[bench]
    fn spawn_lot(b: &mut Bencher) {
        _spawn_lot(b);
    }
    #[bench]
    fn spawn_single(b: &mut Bencher) {
        _spawn_single(b);
    }
}

// Benchmark for a 10K burst task spawn
fn _spawn_lot(b: &mut Bencher) {
    let proc_stack = ProcStack::default();
    b.iter(|| {
        let _ = (0..10_000)
            .map(|_| {
                spawn(
                    async {
                        let duration = Duration::from_millis(1);
                        Delay::new(duration).await;
                    },
                    proc_stack.clone(),
                )
            })
            .collect::<Vec<_>>();
    });
}

// Benchmark for a single task spawn
fn _spawn_single(b: &mut Bencher) {
    let proc_stack = ProcStack::default();
    b.iter(|| {
        spawn(
            async {
                let duration = Duration::from_millis(1);
                Delay::new(duration).await;
            },
            proc_stack.clone(),
        );
    });
}
