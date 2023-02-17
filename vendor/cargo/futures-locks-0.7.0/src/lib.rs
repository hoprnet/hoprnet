// vim: tw=80

//!  A library of [`Futures`]-aware locking primitives.  These locks can safely
//!  be used in asynchronous environments like [`Tokio`].  When they block,
//!  they'll only block a single task, not the entire reactor.
//!
//!  These primitives generally work much like their counterparts from the
//!  standard library.  But instead of blocking, they return a `Future` that
//!  completes when the lock has been acquired.
//!
//! # Examples
//!
//! ```
//! # use futures_locks::*;
//! # use futures::executor::block_on;
//! # use futures::{Future, FutureExt};
//! # fn main() {
//! let mtx = Mutex::<u32>::new(0);
//! let fut = mtx.lock().map(|mut guard| { *guard += 5; });
//! block_on(fut);
//! assert_eq!(mtx.try_unwrap().unwrap(), 5);
//! # }
//! ```
//!
//! [`Futures`]: https://github.com/rust-lang-nursery/futures-rs
//! [`Tokio`]: https:/tokio.rs

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod mutex;
mod rwlock;

pub use mutex::{Mutex, MutexFut, MutexGuard, MutexWeak};
pub use rwlock::{RwLock, RwLockReadFut, RwLockWriteFut,
                 RwLockReadGuard, RwLockWriteGuard};

use futures_channel::oneshot;
use std::{error, fmt};

/// Poll state of all Futures in this crate.
enum FutState {
    New,
    Pending(oneshot::Receiver<()>),
    Acquired
}

/// The lock could not be acquired at this time because the operation would
/// otherwise block.
#[derive(Clone, Copy, Debug)]
pub struct TryLockError;

impl fmt::Display for TryLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "try_lock failed because the operation would block")
    }
}

impl error::Error for TryLockError {}
