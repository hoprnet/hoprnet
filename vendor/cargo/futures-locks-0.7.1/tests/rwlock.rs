//vim: tw=80

use futures::{
    FutureExt,
    StreamExt,
    stream
};
#[cfg(feature = "tokio")]
use futures::future::ready;
use std::sync::Arc;
#[cfg(feature = "tokio")]
use std::rc::Rc;
use tokio::{self, sync::Barrier};
#[cfg(feature = "tokio")]
use tokio::runtime;
use tokio_test::{
    assert_pending,
    assert_ready,
    task::spawn
};
use futures_locks::*;


// When an exclusively owned RwLock future is dropped after gaining ownership
// but before being polled, it should relinquish ownership.  If not, deadlocks
// may result.
#[test]
fn drop_exclusive_before_poll_returns_ready() {
    let rwlock = RwLock::<u32>::new(42);

    let mut fut1 = spawn(rwlock.read());
    let guard1 = assert_ready!(fut1.poll()); // fut1 immediately gets ownership
    let mut fut2 = spawn(rwlock.write());
    assert_pending!(fut2.poll());            // fut2 is blocked
    drop(guard1);                            // ownership transfers to fut2
    drop(fut1);
    drop(fut2);                              // relinquish ownership
    let mut fut3 = spawn(rwlock.read());     // fut3 immediately gets ownership
    assert_ready!(fut3.poll());
}

// When a pending exclusive RwLock gets dropped after being polled() but before
// gaining ownership, ownership should pass on to the next waiter.
#[test]
fn drop_exclusive_before_ready() {
    let rwlock = RwLock::<u32>::new(42);

    let mut fut1 = spawn(rwlock.read());
    let guard1 = assert_ready!(fut1.poll()); // fut1 immediately gets ownership
    let mut fut2 = spawn(rwlock.write());
    assert_pending!(fut2.poll());            // fut2 is blocked
    let mut fut3 = spawn(rwlock.write());
    assert_pending!(fut3.poll());            // fut3 is also blocked
    drop(fut2);                  // drop before gaining ownership
    drop(guard1);                // ownership transfers to fut3
    drop(fut1);
    assert!(fut3.is_woken());
    assert_ready!(fut3.poll());
}

// Like drop_exclusive_before_ready, but the rwlock is already locked in
// exclusive mode.
#[test]
fn drop_exclusive_before_ready_2() {
    let rwlock = RwLock::<u32>::new(42);

    let mut fut1 = spawn(rwlock.write());
    let guard1 = assert_ready!(fut1.poll()); // fut1 immediately gets ownership
    let mut fut2 = spawn(rwlock.write());
    assert_pending!(fut2.poll());            // fut2 is blocked
    let mut fut3 = spawn(rwlock.write());
    assert_pending!(fut3.poll());            // fut3 is also blocked
    drop(fut2);                  // drop before gaining ownership
    drop(guard1);                // ownership transfers to fut3
    drop(fut1);
    assert!(fut3.is_woken());
    assert_ready!(fut3.poll());
}

// When a nonexclusively owned RwLock future is dropped after gaining ownership
// but before begin polled, it should relinquish ownership.  If not, deadlocks
// may result.
#[test]
fn drop_shared_before_poll_returns_ready() {
    let rwlock = RwLock::<u32>::new(42);

    let mut fut1 = spawn(rwlock.write());
    let guard1 = assert_ready!(fut1.poll()); // fut1 immediately gets ownership
    let mut fut2 = spawn(rwlock.read());
    assert_pending!(fut2.poll());            // fut2 is blocked
    drop(guard1);                            // ownership transfers to fut2
    drop(fut2);                              // relinquish ownership
    let mut fut3 = spawn(rwlock.write());    // fut3 immediately gets ownership
    assert_ready!(fut3.poll());
}

// When a pending shared RwLock gets dropped after being polled() but before
// gaining ownership, ownership should pass on to the next waiter.
#[test]
fn drop_shared_before_ready() {
    let rwlock = RwLock::<u32>::new(42);

    let mut fut1 = spawn(rwlock.write());
    let guard1 = assert_ready!(fut1.poll()); // fut1 immediately gets ownership
    let mut fut2 = spawn(rwlock.read());
    assert_pending!(fut2.poll());            // fut2 is blocked
    let mut fut3 = spawn(rwlock.read());
    assert_pending!(fut3.poll());            // fut3 is also blocked
    drop(fut2);                  // drop before gaining ownership
    drop(guard1);                // ownership transfers to fut3
    drop(fut1);
    assert!(fut3.is_woken());
    assert_ready!(fut3.poll());
}

// Mutably dereference a uniquely owned RwLock
#[test]
fn get_mut() {
    let mut rwlock = RwLock::<u32>::new(42);
    *rwlock.get_mut().unwrap() += 1;
    assert_eq!(*rwlock.get_mut().unwrap(), 43);
}

// Cloned RwLocks cannot be deferenced
#[test]
fn get_mut_cloned() {
    let mut rwlock = RwLock::<u32>::new(42);
    let _clone = rwlock.clone();
    assert!(rwlock.get_mut().is_none());
}

// Acquire an RwLock nonexclusively by two different tasks simultaneously .
#[test]
fn read_shared() {
    let rwlock = RwLock::<u32>::new(42);

    let mut fut1 = spawn(rwlock.read());
    let _guard1 = assert_ready!(fut1.poll()); // fut1 immediately gets ownership
    let mut fut2 = spawn(rwlock.read());
    let _guard2 = assert_ready!(fut2.poll()); // fut2 also gets ownership
}

// Acquire an RwLock nonexclusively by a single task
#[tokio::test]
async fn read_uncontested() {
    let rwlock = RwLock::<u32>::new(42);

    let guard = rwlock.read().await;
    let result = *guard;
    drop(guard);

    assert_eq!(result, 42);
}

// Attempt to acquire an RwLock for reading that already has a writer
#[test]
fn write_read_contested() {
    let rwlock = RwLock::<u32>::new(0);

    let mut fut0 = spawn(rwlock.write());
    let guard0 = assert_ready!(fut0.poll()); // fut0 immediately gets ownership

    let mut fut1 = spawn(rwlock.read());
    assert_pending!(fut1.poll());            // fut1 is blocked

    drop(guard0);                   // Ownership transfers to fut1
    assert!(fut1.is_woken());
    assert_ready!(fut1.poll());
}

// Attempt to acquire an rwlock exclusively when it already has a reader.
#[test]
fn read_write_contested() {
    let rwlock = RwLock::<u32>::new(42);

    let mut fut0 = spawn(rwlock.read());
    let guard0 = assert_ready!(fut0.poll()); // fut0 immediately gets ownership

    let mut fut1 = spawn(rwlock.write());
    assert_pending!(fut1.poll());            // fut1 is blocked

    drop(guard0);                   // Ownership transfers to fut1
    assert!(fut1.is_woken());
    assert_ready!(fut1.poll());
}

// Attempt to acquire an rwlock exclusively when it already has a writer.
#[test]
fn write_contested() {
    let rwlock = RwLock::<u32>::new(42);

    let mut fut0 = spawn(rwlock.write());
    let guard0 = assert_ready!(fut0.poll()); // fut0 immediately gets ownership

    let mut fut1 = spawn(rwlock.write());
    assert_pending!(fut1.poll());            // fut1 is blocked

    drop(guard0);                   // Ownership transfers to fut1
    assert!(fut1.is_woken());
    assert_ready!(fut1.poll());
}

#[test]
fn try_read_uncontested() {
    let rwlock = RwLock::<u32>::new(42);
    assert_eq!(42, *rwlock.try_read().unwrap());
}

#[test]
fn try_read_contested() {
    let rwlock = RwLock::<u32>::new(42);
    let _guard = rwlock.try_write();
    assert!(rwlock.try_read().is_err());
}

#[test]
fn try_unwrap_multiply_referenced() {
    let rwlock = RwLock::<u32>::new(0);
    let _rwlock2 = rwlock.clone();
    assert!(rwlock.try_unwrap().is_err());
}

#[test]
fn try_write_uncontested() {
    let rwlock = RwLock::<u32>::new(0);
    *rwlock.try_write().unwrap() += 5;
    assert_eq!(5, rwlock.try_unwrap().unwrap());
}

#[test]
fn try_write_contested() {
    let rwlock = RwLock::<u32>::new(42);
    let _guard = rwlock.try_read();
    assert!(rwlock.try_write().is_err());
}

// Acquire an uncontested RwLock in exclusive mode.  poll immediately returns
// Ready
#[tokio::test]
async fn write_uncontested() {
    let rwlock = RwLock::<u32>::new(0);

    let mut guard = rwlock.write().await;
    *guard += 5;
    drop(guard);
    assert_eq!(rwlock.try_unwrap().expect("try_unwrap"), 5);
}

// RwLocks should be acquired in the order that their Futures are waited upon.
#[tokio::test]
async fn write_order() {
    let rwlock = RwLock::<Vec<u32>>::new(vec![]);
    let fut2 = rwlock.write().map(|mut guard| guard.push(2));
    let fut1 = rwlock.write().map(|mut guard| guard.push(1));

    fut1.then(|_| fut2).await;
    assert_eq!(rwlock.try_unwrap().unwrap(), vec![1, 2]);
}

// A single RwLock is contested by tasks in multiple threads
#[tokio::test]
async fn multithreaded() {
    let rwlock = RwLock::<u32>::new(0);
    let rwlock_clone0 = rwlock.clone();
    let rwlock_clone1 = rwlock.clone();
    let rwlock_clone2 = rwlock.clone();
    let rwlock_clone3 = rwlock.clone();
    let barrier = Arc::new(Barrier::new(5));
    let b0 = barrier.clone();
    let b1 = barrier.clone();
    let b2 = barrier.clone();
    let b3 = barrier.clone();

    tokio::task::spawn(async move {
        stream::iter(0..1000).for_each(move |_| {
            let rwlock_clone4 = rwlock_clone0.clone();
            rwlock_clone0.write()
            .map(|mut guard| { *guard += 2 })
            .then(move |_| rwlock_clone4.read().map(|_| ()))
        }).await;
        b0.wait().await;
    });
    tokio::task::spawn(async move {
        stream::iter(0..1000).for_each(move |_| {
            let rwlock_clone5 = rwlock_clone1.clone();
            rwlock_clone1.write()
            .map(|mut guard| { *guard += 3 })
            .then(move |_| rwlock_clone5.read().map(|_| ()))
        }).await;
        b1.wait().await;
    });
    tokio::task::spawn(async move {
        stream::iter(0..1000).for_each(move |_| {
            let rwlock_clone6 = rwlock_clone2.clone();
            rwlock_clone2.write()
            .map(|mut guard| { *guard += 5 })
            .then(move |_| rwlock_clone6.read().map(|_| ()))
        }).await;
        b2.wait().await;
    });
    tokio::task::spawn(async move {
        stream::iter(0..1000).for_each(move |_| {
            let rwlock_clone7 = rwlock_clone3.clone();
            rwlock_clone3.write()
            .map(|mut guard| { *guard += 7 })
            .then(move |_| rwlock_clone7.read().map(|_| ()))
        }).await;
        b3.wait().await;
    });

    barrier.wait().await;
    assert_eq!(rwlock.try_unwrap().expect("try_unwrap"), 17_000);
}

// Returning errors is simpler than in futures-locks 0.5: just return a Result
#[cfg(feature = "tokio")]
#[test]
fn with_read_err() {
    let mtx = RwLock::<i32>::new(-5);
    let rt = runtime::Builder::new_current_thread().build().unwrap();

    let r = rt.block_on(async {
        mtx.with_read(|guard| {
            if *guard > 0 {
                ready(Ok(*guard))
            } else {
                ready(Err("Whoops!"))
            }
        }).await
    });
    assert_eq!(r, Err("Whoops!"));
}

#[cfg(feature = "tokio")]
#[test]
fn with_read_ok() {
    let mtx = RwLock::<i32>::new(5);
    let rt = runtime::Builder::new_current_thread().build().unwrap();

    let r = rt.block_on(async {
        mtx.with_read(|guard| {
            ready(*guard)
        }).await
    });
    assert_eq!(r, 5);
}

// RwLock::with_read should work with multithreaded Runtimes as well as
// single-threaded Runtimes.
// https://github.com/asomers/futures-locks/issues/5
#[cfg(feature = "tokio")]
#[test]
fn with_read_threadpool() {
    let mtx = RwLock::<i32>::new(5);
    let rt = runtime::Builder::new_multi_thread().build().unwrap();

    let r = rt.block_on(async {
        mtx.with_read(|guard| {
            ready(*guard)
        }).await
    });
    assert_eq!(r, 5);
}

#[cfg(feature = "tokio")]
#[test]
fn with_read_local_ok() {
    // Note: Rc is not Send
    let rwlock = RwLock::<Rc<i32>>::new(Rc::new(5));
    let rt = runtime::Builder::new_current_thread().build().unwrap();
    let r = rt.block_on(async {
        rwlock.with_read_local(|guard| {
            ready(**guard)
        }).await
    });
    assert_eq!(r, 5);
}

// Returning errors is simpler than in futures-locks 0.5: just return a Result
#[cfg(feature = "tokio")]
#[test]
fn with_write_err() {
    let mtx = RwLock::<i32>::new(-5);
    let rt = runtime::Builder::new_current_thread().build().unwrap();

    let r = rt.block_on(async {
        mtx.with_write(|mut guard| {
            if *guard > 0 {
                *guard -= 1;
                ready(Ok(()))
            } else {
                ready(Err("Whoops!"))
            }
        }).await
    });
    assert_eq!(r, Err("Whoops!"));
}

#[cfg(feature = "tokio")]
#[test]
fn with_write_ok() {
    let mtx = RwLock::<i32>::new(5);
    let rt = runtime::Builder::new_current_thread().build().unwrap();

    rt.block_on(async {
        mtx.with_write(|mut guard| {
            *guard += 1;
            ready(())
        }).await
    });
    assert_eq!(mtx.try_unwrap().unwrap(), 6);
}

// RwLock::with_write should work with multithreaded Runtimes as well as
// single-threaded Runtimes.
// https://github.com/asomers/futures-locks/issues/5
#[cfg(feature = "tokio")]
#[test]
fn with_write_threadpool() {
    let mtx = RwLock::<i32>::new(5);
    let rt = runtime::Builder::new_multi_thread().build().unwrap();

    rt.block_on(async {
        mtx.with_write(|mut guard| {
            *guard += 1;
            ready(())
        }).await
    });
    assert_eq!(mtx.try_unwrap().unwrap(), 6);
}

#[cfg(feature = "tokio")]
#[test]
fn with_write_local_ok() {
    // Note: Rc is not Send
    let rwlock = RwLock::<Rc<i32>>::new(Rc::new(5));
    let rt = runtime::Builder::new_current_thread().build().unwrap();
    rt.block_on(async {
        rwlock.with_write_local(|mut guard| {
            *Rc::get_mut(&mut *guard).unwrap() += 1;
            ready(())
        }).await
    });
    assert_eq!(*rwlock.try_unwrap().unwrap(), 6);
}
