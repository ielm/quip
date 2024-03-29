use tinyproc::proc_stack::ProcStack;
use tinyproc::proc_state::EmptyProcState;

#[test]
fn stack_copy() {
    let stack = ProcStack::default()
        .with_pid(12)
        .with_after_panic(|_s: &mut EmptyProcState| {
            println!("After panic!");
        });

    let stack2 = stack;

    assert_eq!(stack2.get_pid(), 12);
}
