use crossbeam::channel::{unbounded, Sender};
use futures_executor as executor;
use lazy_static::lazy_static;
use std::future::Future;
use std::thread;
use tinyproc::prelude::*;

fn spawn_on_thread<F, R>(future: F) -> RecoverableHandle<R>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    lazy_static! {
        // A channel that holds scheduled procs.
        static ref QUEUE: Sender<TinyProc> = {
            let (sender, receiver) = unbounded::<TinyProc>();

            // Start the executor thread.
            thread::spawn(move || {
                for proc in receiver {
                    proc.run();
                }
            });

            sender
        };
    }

    let schedule = |t| QUEUE.send(t).unwrap();
    let (proc, handle) = TinyProc::recoverable(
        future,
        schedule,
        ProcStack::default()
            .with_pid(1)
            .with_before_start(|_s: &mut EmptyProcState| {
                println!("Before start");
            })
            .with_after_complete(|_s: &mut EmptyProcState| {
                println!("After complete");
            })
            .with_after_panic(|_s: &mut EmptyProcState| {
                println!("After panic");
            }),
    );

    proc.schedule();

    handle
}

fn main() {
    let handle = spawn_on_thread(async {
        panic!("Panic here!");
    });

    executor::block_on(handle);
}
