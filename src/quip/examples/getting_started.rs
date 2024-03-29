use quip::prelude::*;

fn main() {
    env_logger::init();

    // Creating the system's configuration...
    let config = Config::new().hide_backtraces();
    // ...and initializing the system with it (this is required)...
    Quip::init_with(config);

    // Note that `Quip::init();` would work too and initialize
    // the system with the default config.

    // Starting the system...
    Quip::start();

    // Creating a new supervisor...
    let supervisor = Quip::supervisor(|sp| {
        sp
            // ...with a specific supervision strategy...
            .with_strategy(SupervisionStrategy::OneForAll)
            // ...and some supervised children groups...
            .children(|children| {
                // ...
                children
            })
            .children(|children| {
                // ...
                children
            })
            // ...or even supervised supervisors...
            .supervisor(|sp| {
                // ...
                sp
            })
    })
    .expect("Couldn't create the supervisor.");

    // ...which can start supervising new children groups
    // later on...
    supervisor
        .children(|children| {
            // ...
            children
        })
        .expect("Couldn't create the supervised children group.");

    // ...or broadcast messages to all its supervised children
    // and supervisors...
    supervisor
        .broadcast("A message containing data.")
        .expect("Couldn't broadcast the message.");

    // ...and then can even be stopped or killed...
    supervisor.stop().expect("Couldn't stop the supervisor");
    // supervisor.kill().expect("Couldn't kill the supervisor");

    // Creating a new top-level children group...
    let children = Quip::children(|children| {
        children
            // ...containing a defined number of elements...
            .with_redundancy(4)
            // ...all executing a similar future...
            .with_exec(|ctx: QuipContext| {
                async move {
                    // ...receiving and matching messages...
                    msg! { ctx.recv().await?,
                        // ref <name> are broadcasts.
                        ref _msg: &'static str => {
                            // ...
                        };
                        // <name> (without the ref keyword) are messages that have a unique recipient.
                        _msg: &'static str => {
                            // ...
                        };
                        // =!> refer to messages that can be replied to.
                        _msg: &'static str =!> {
                            // ...
                        };
                        // <name> that have the `_` type are catch alls
                        _: _ => ();
                    }

                    // ...

                    Ok(())
                }
            })
    })
    .expect("Couldn't create the children group.");

    // ...which can broadcast messages to all its elements...
    children
        .broadcast("A message containing data.")
        .expect("Couldn't broadcast the message.");

    // ...and then can even be stopped or killed...
    children.stop().expect("Couldn't stop the children group.");
    // children.kill().expect("Couldn't kill the children group.");

    // Create a new top-level children group and getting a list
    // of reference to its elements...
    let children = Quip::children(|children| {
        // ...
        children
    })
    .expect("Couldn't create the children group.");
    let elems: &[ChildRef] = children.elems();

    // ...to then get one of its elements' reference...
    let child = &elems[0];

    // ...to then "tell" it messages...
    child
        .tell_anonymously("A message containing data.")
        .expect("Couldn't send the message.");

    // ...or "ask" it messages...
    let answer: Answer = child
        .ask_anonymously("A message containing data.")
        .expect("Couldn't send the message.");
    let _ = async {
        // ...until the child eventually answers back...
        let _answer: Result<SignedMessage, ()> = answer.await;
    };

    // ...and then even stop or kill it...
    child.stop().expect("Couldn't stop the child.");
    // child.kill().expect("Couldn't kill the child.");

    // Broadcasting a message to all the system's children...
    Quip::broadcast("A message containing data.").expect("Couldn't send the message.");

    // Stopping or killing the system...
    // Quip::stop();
    // Quip::kill();

    // Blocking until the system has stopped (or got killed)...
    Quip::block_until_stopped();
}
