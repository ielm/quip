#[cfg(not(target_os = "windows"))]
use std::fs::File;
use std::fs::OpenOptions;
#[cfg(target_os = "windows")]
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(not(target_os = "windows"))]
use futures::*;

use quip::prelude::*;

///
/// Parallel (MapReduce) job which async writes results to a single output file
///
/// Prologue:
/// This example maps a stream of cycling values([0,1,2,3,4,0,1,2,3,4...]) one by one:
/// to 10 workers and every worker compute the double of what they receive and send back.
///
/// Then mapper aggregates the doubled values and write them in their receiving order.
///
/// Try increasing the worker count. Yes!
fn main() {
    env_logger::init();

    Quip::init();

    // Workers that process the work.
    let workers = Quip::children(|children: Children| {
        children
            .with_redundancy(10) // Let's have a pool of ten workers.
            .with_exec(move |ctx: QuipContext| {
                async move {
                    println!("Worker started!");

                    // Start receiving work
                    loop {
                        msg! { ctx.recv().await?,
                            msg: u64 =!> {
                                let data: u64 = msg.wrapping_mul(2);
                                println!("Child doubled the value of {} and gave {}", msg, data); // true
                                let _ = answer!(ctx, data);
                            };
                            _: _ => ();
                        }
                    }
                }
            })
    })
    .expect("Couldn't start a new children group.");

    // Get a shadowed sharable reference of workers.
    let workers = Arc::new(workers);

    // Mapper that generates work.
    Quip::children(|children: Children| {
        children.with_exec(move |ctx: QuipContext| {
            let workers = workers.clone();
            async move {
                println!("Mapper started!");

                let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                path.push("data");
                path.push("distwrite");

                if !path.exists() || !path.is_file() {
                    Quip::stop();
                    panic!("The file could not be opened.");
                }

                let fo = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&path)
                    .unwrap();

                #[cfg(not(target_os = "windows"))]
                let mut file = Handle::<File>::new(fo).unwrap();
                #[cfg(target_os = "windows")]
                let mut file = fo;

                // Distribute your workload to workers
                for id_worker_pair in workers.elems().iter().enumerate() {
                    let data = cycle(id_worker_pair.0 as u64, 5);

                    let computed: Answer = ctx.ask(&id_worker_pair.1.addr(), data).unwrap();
                    msg! { computed.await?,
                        msg: u64 => {
                            // Handle the answer...
                            println!("Source received the computed value: {}", msg);
                            #[cfg(target_os = "windows")]
                            file.write_all(msg.to_string().as_bytes()).unwrap();
                            #[cfg(not(target_os = "windows"))]
                            file.write_all(msg.to_string().as_bytes()).await.unwrap();
                        };
                        _: _ => ();
                    }
                }

                // Send a signal to system that computation is finished.
                Quip::stop();

                Ok(())
            }
        })
    })
    .expect("Couldn't start a new children group.");

    Quip::start();
    Quip::block_until_stopped();
}

fn cycle(x: u64, at_most: u64) -> u64 {
    let mut x = x;
    x += 1;
    x % at_most
}
