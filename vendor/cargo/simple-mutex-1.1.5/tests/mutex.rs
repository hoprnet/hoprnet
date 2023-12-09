use std::sync::{mpsc, Arc};
use std::thread;

use simple_mutex::Mutex;

#[test]
fn smoke() {
    let m = Mutex::new(());
    drop(m.lock());
    drop(m.lock());
}

#[test]
fn try_lock() {
    let m = Mutex::new(());
    *m.try_lock().unwrap() = ();
}

#[test]
fn into_inner() {
    let m = Mutex::new(10i32);
    assert_eq!(m.into_inner(), 10);
}

#[test]
fn get_mut() {
    let mut m = Mutex::new(10i32);
    *m.get_mut() = 20;
    assert_eq!(m.into_inner(), 20);
}

#[test]
fn contention() {
    let (tx, rx) = mpsc::channel();
    let mutex = Arc::new(Mutex::new(0i32));
    let num_threads = 100;

    for _ in 0..num_threads {
        let tx = tx.clone();
        let mutex = mutex.clone();

        thread::spawn(move || {
            let mut lock = mutex.lock();
            *lock += 1;
            tx.send(()).unwrap();
            drop(lock);
        });
    }

    for _ in 0..num_threads {
        rx.recv().unwrap();
    }

    let lock = mutex.lock();
    assert_eq!(num_threads, *lock);
}
