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
        fn proptest_redundancy(r in std::usize::MIN..32) {
            let _ = tokio_test::task::spawn(async {
                super::test_with_usize(r);
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
        fn proptest_redundancy(r in std::usize::MIN..32) {
            super::test_with_usize(r);
        }
    }
}

fn test_with_usize(r: usize) {
    START.call_once(|| {
        Quip::init();
    });
    Quip::start();

    Quip::children(|children| {
        children
            // shrink over the redundancy
            .with_redundancy(r)
            .with_exec(|_ctx: QuipContext| {
                async move {
                    // It's a proptest,
                    // we just don't want the loop
                    // to be optimized away
                    #[allow(clippy::drop_copy)]
                    loop {
                        std::mem::drop(());
                    }
                }
            })
    })
    .expect("Coudn't spawn children.");
}
