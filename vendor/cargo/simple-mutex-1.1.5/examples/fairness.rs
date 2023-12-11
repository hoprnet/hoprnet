/// Demonstrates fairness properties of the mutex.
///
/// A number of threads run a loop in which they hold the lock for a little bit and re-acquire it
/// immediately after. In the end we print the number of times each thread acquired the lock.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use simple_mutex::Mutex;

fn main() {
    let num_threads = 30;
    let mut threads = Vec::new();
    let hits = Arc::new(Mutex::new(vec![0; num_threads]));

    for i in 0..num_threads {
        let hits = hits.clone();
        threads.push(thread::spawn(move || {
            let start = Instant::now();

            while start.elapsed() < Duration::from_secs(1) {
                let mut hits = hits.lock();
                hits[i] += 1;
                thread::sleep(Duration::from_micros(5000));
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    dbg!(hits);
}
