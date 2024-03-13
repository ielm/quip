use quip::prelude::*;
use std::fmt::Debug;
use tracing::error;

// This example shows that it is possible to use the MessageHandler to match
// over different types of message.

async fn child_task(ctx: QuipContext) -> Result<(), ()> {
    loop {
        MessageHandler::new(ctx.recv().await?)
            .on_question(|n: i32, sender| {
                if n == 42 {
                    sender.reply(101).expect("Failed to reply to sender");
                } else {
                    error!("Expected number `42`, found `{}`", n);
                }
            })
            .on_question(|s: &str, sender| {
                if s == "marco" {
                    sender.reply("polo").expect("Failed to reply to sender");
                } else {
                    panic!("Expected string `marco`, found `{}`", s);
                }
            })
            .on_fallback(|v, addr| panic!("Wrong message from {:?}: got {:?}", addr, v))
    }
}

async fn request<T: 'static + Debug + Send + Sync>(
    child: &ChildRef,
    body: T,
) -> std::io::Result<()> {
    let answer = child
        .ask_anonymously(body)
        .expect("Couldn't perform request")
        .await
        .expect("Couldn't receive answer");

    MessageHandler::new(answer)
        .on_tell(|n: i32, _| assert_eq!(n, 101))
        .on_tell(|s: &str, _| assert_eq!(s, "polo"))
        .on_fallback(|_, _| panic!("Unknown message"));

    Ok(())
}

fn main() {
    env_logger::init();

    Quip::init();
    Quip::start();

    let children =
        Quip::children(|c| c.with_exec(child_task)).expect("Failed to spawn children");

    let child = &children.elems()[0];

    run!(request(child, 42)).unwrap();
    run!(request(child, "marco")).unwrap();

    // run!(request(child, "foo")).unwrap();
}
