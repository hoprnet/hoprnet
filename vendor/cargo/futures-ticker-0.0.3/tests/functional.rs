use futures::stream::StreamExt;
use futures_ticker::Ticker;
use std::time::{Duration, Instant};

const INTERVAL: Duration = Duration::from_millis(100); // 100ms should be enough to let tests work consistently.
const FIB_TIME: Duration = Duration::from_micros(10); // about this much "play".

#[test]
fn does_wait() {
    let start = Instant::now();
    let mut ticker = Ticker::new(INTERVAL);
    smol::block_on(async {
        for i in 0..3 {
            let start = start + i * INTERVAL;
            if let Some(next) = ticker.next().await {
                println!(
                    "now={:?} next={:?} since_start={:?}",
                    Instant::now(),
                    next,
                    start.elapsed()
                );

                assert!(
                    next.duration_since(start) >= INTERVAL - FIB_TIME,
                    "mismatch at {}: next={:?}, start={:?}",
                    i,
                    next,
                    start
                );
            } else {
                panic!("Received None from ticker, that's impossible");
            }
        }
    });
}
