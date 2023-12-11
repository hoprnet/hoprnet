#![feature(test)]

extern crate test;

use std::sync::Arc;
use std::thread;

use parking_lot::Mutex;
use test::Bencher;

#[bench]
fn create(b: &mut Bencher) {
    b.iter(|| Mutex::new(()));
}

#[bench]
fn contention(b: &mut Bencher) {
    b.iter(|| run(10, 1000));
}

#[bench]
fn no_contention(b: &mut Bencher) {
    b.iter(|| run(1, 10000));
}

fn run(num_threads: usize, iter: usize) {
    let m = Arc::new(Mutex::new(0i32));
    let mut threads = Vec::new();

    for _ in 0..num_threads {
        let m = m.clone();
        threads.push(thread::spawn(move || {
            for _ in 0..iter {
                *m.lock() += 1;
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}
