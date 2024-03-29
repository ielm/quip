use futures::future::join_all;
use quip_executor::blocking;
use quip_executor::run::run;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use tinyproc::proc_stack::ProcStack;
use tinyproc::recoverable_handle::RecoverableHandle;

// Test for slow joins without task bursts during joins.
#[test]
#[ignore]
fn slow_join() {
    let thread_join_time_max = 11_000;
    let start = Instant::now();

    // Send an initial batch of million bursts.
    let handles = (0..1_000_000)
        .map(|_| {
            blocking::spawn_blocking(
                async {
                    let duration = Duration::from_millis(1);
                    thread::sleep(duration);
                },
                ProcStack::default(),
            )
        })
        .collect::<Vec<RecoverableHandle<()>>>();

    run(join_all(handles), ProcStack::default());

    // Let them join to see how it behaves under different workloads.
    let duration = Duration::from_millis(thread_join_time_max);
    thread::sleep(duration);

    // Spawn yet another batch of work on top of it
    let handles = (0..10_000)
        .map(|_| {
            blocking::spawn_blocking(
                async {
                    let duration = Duration::from_millis(100);
                    thread::sleep(duration);
                },
                ProcStack::default(),
            )
        })
        .collect::<Vec<RecoverableHandle<()>>>();

    run(join_all(handles), ProcStack::default());

    // Slow joins shouldn't cause internal slow down
    let elapsed = start.elapsed().as_millis() - thread_join_time_max as u128;
    println!("Slow task join. Monotonic exec time: {:?} ns", elapsed);

    // Previous implementation is around this threshold.
}

// Test for slow joins with task burst.
#[test]
#[ignore]
fn slow_join_interrupted() {
    let thread_join_time_max = 2_000;
    let start = Instant::now();

    // Send an initial batch of million bursts.
    let handles = (0..1_000_000)
        .map(|_| {
            blocking::spawn_blocking(
                async {
                    let duration = Duration::from_millis(1);
                    thread::sleep(duration);
                },
                ProcStack::default(),
            )
        })
        .collect::<Vec<RecoverableHandle<()>>>();

    run(join_all(handles), ProcStack::default());

    // Let them join to see how it behaves under different workloads.
    // This time join under the time window.
    let duration = Duration::from_millis(thread_join_time_max);
    thread::sleep(duration);

    // Spawn yet another batch of work on top of it
    let handles = (0..10_000)
        .map(|_| {
            blocking::spawn_blocking(
                async {
                    let duration = Duration::from_millis(100);
                    thread::sleep(duration);
                },
                ProcStack::default(),
            )
        })
        .collect::<Vec<RecoverableHandle<()>>>();

    run(join_all(handles), ProcStack::default());

    // Slow joins shouldn't cause internal slow down
    let elapsed = start.elapsed().as_millis() - thread_join_time_max as u128;
    println!("Slow task join. Monotonic exec time: {:?} ns", elapsed);

    // Previous implementation is around this threshold.
}

// This test is expensive but it proves that longhauling tasks are working in adaptive thread pool.
// Thread pool which spawns on-demand will panic with this test.
#[test]
#[ignore]
fn longhauling_task_join() {
    let thread_join_time_max = 11_000;
    let start = Instant::now();

    // First batch of overhauling tasks
    let _ = (0..100_000)
        .map(|_| {
            blocking::spawn_blocking(
                async {
                    let duration = Duration::from_millis(1000);
                    thread::sleep(duration);
                },
                ProcStack::default(),
            )
        })
        .collect::<Vec<RecoverableHandle<()>>>();

    // Let them join to see how it behaves under different workloads.
    let duration = Duration::from_millis(thread_join_time_max);
    thread::sleep(duration);

    // Send yet another medium sized batch to see how it scales.
    let handles = (0..10_000)
        .map(|_| {
            blocking::spawn_blocking(
                async {
                    let duration = Duration::from_millis(100);
                    thread::sleep(duration);
                },
                ProcStack::default(),
            )
        })
        .collect::<Vec<RecoverableHandle<()>>>();

    run(join_all(handles), ProcStack::default());

    // Slow joins shouldn't cause internal slow down
    let elapsed = start.elapsed().as_millis() - thread_join_time_max as u128;
    println!(
        "Long-hauling task join. Monotonic exec time: {:?} ns",
        elapsed
    );

    // Previous implementation will panic when this test is running.
}
