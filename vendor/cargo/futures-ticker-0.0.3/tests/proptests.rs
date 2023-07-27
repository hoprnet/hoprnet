use proptest::prelude::*;

use futures_ticker::Ticker;
use std::time::{Duration, Instant};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50000))]

    #[test]
    fn next_tick_always_in_future(ms_interval in 1u64..50000,
                                  test_at in 0u64..100000) {
        let now = Instant::now();
        let interval = Duration::from_millis(ms_interval);
        let ticker = Ticker::new(interval);
        let test_at = Duration::from_millis(test_at);


        let next = ticker.next_tick_from(now);
        prop_assert!(next > now, "next tick should be after now: next={:?} <= now={:?}", next, now);

        let next = ticker.next_tick_from(now+test_at);
        // println!("interval={:?} next={:?} test_at={:?}", interval, next, test_at);
        prop_assert!(next > now+test_at, "next tick should be after test_at: next={:?} <= test_at={:?}", next, test_at);
    }
}
