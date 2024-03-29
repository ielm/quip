use proptest::prelude::*;
use quip::prelude::*;
use std::sync::Once;

static START: Once = Once::new();

#[cfg(feature = "tokio-runtime")]
mod tokio_proptests {
    use super::*;
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1_000))]
        #[test]
        fn proptest_bcast_message(message in "\\PC*") {
            tokio_test::block_on(async {
                super::test_with_message(message);
            });
        }
    }
}
#[cfg(not(feature = "tokio-runtime"))]
mod not_tokio_proptests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1_000))]
        #[test]
        fn proptest_bcast_message(message in "\\PC*") {
            super::test_with_message(message);
        }
    }
}

fn test_with_message(message: String) {
    START.call_once(|| {
        Quip::init();
    });
    Quip::start();

    if let Ok(_chrn) = Quip::children(|children: Children| {
        children.with_exec(move |ctx: QuipContext| {
            async move {
                msg! { ctx.recv().await?,
                    ref _msg: &'static str => {};
                    // This won't happen because this example
                    // only "asks" a `&'static str`...
                    _: _ => {};
                }

                Ok(())
            }
        })
    }) {
        let message: &'static str = Box::leak(message.into_boxed_str());
        Quip::broadcast(message).expect("broadcast failed");
    }
}
