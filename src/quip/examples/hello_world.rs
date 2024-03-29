use quip::prelude::*;

async fn once_hello_world(ctx: QuipContext) -> Result<(), ()> {
    MessageHandler::new(ctx.recv().await?)
        .on_question(|question: &str, sender| {
            if question == "hi!" {
                sender
                    .reply("hello, world!")
                    .expect("Failed to reply to sender");
            } else {
                panic!("Expected string `hi!`, found `{}`", question);
            }
        })
        .on_fallback(|v, addr| panic!("Wrong message from {:?}: got {:?}", addr, v));
    Ok(())
}

fn main() {
    Quip::init();
    Quip::start();

    Quip::children(|children| {
        children
            .with_distributor(Distributor::named("say_hi"))
            .with_exec(once_hello_world)
    })
    .expect("Couldn't create the children group.");

    let say_hi = Distributor::named("say_hi");

    run!(async {
        let answer: Result<&str, SendError> =
            say_hi.request("hi!").await.expect("Couldn't send request");

        println!("{}", answer.expect("Couldn't receive answer"))
    });

    Quip::stop();
}
