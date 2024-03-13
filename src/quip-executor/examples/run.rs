use quip_executor::prelude::*;
use tinyproc::proc_stack::ProcStack;
use tinyproc::proc_state::EmptyProcState;

fn main() {
    run(
        async {
            println!("Example execution");
            panic!("fault");
        },
        ProcStack::default().with_after_panic(|_s: &mut EmptyProcState| {
            println!("after panic");
        }),
    );
}
