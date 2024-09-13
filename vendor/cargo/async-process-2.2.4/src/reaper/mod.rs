//! The underlying system reaper.
//!
//! There are two backends:
//!
//! - signal, which waits for SIGCHLD.
//! - wait, which waits directly on a process handle.
//!
//! "wait" is preferred, but is not available on all supported Linuxes. So we
//! test to see if pidfd is supported first. If it is, we use wait. If not, we use
//! signal.

#![allow(irrefutable_let_patterns)]

/// Enable the waiting reaper.
#[cfg(target_os = "linux")]
macro_rules! cfg_wait {
    ($($tt:tt)*) => {$($tt)*};
}

/// Enable the waiting reaper.
#[cfg(not(target_os = "linux"))]
macro_rules! cfg_wait {
    ($($tt:tt)*) => {};
}

/// Enable signals.
macro_rules! cfg_signal {
    ($($tt:tt)*) => {$($tt)*};
}

cfg_wait! {
    mod wait;
}

cfg_signal! {
    mod signal;
}

use std::io;
use std::sync::Mutex;

/// The underlying system reaper.
pub(crate) enum Reaper {
    #[cfg(target_os = "linux")]
    /// The reaper based on the wait backend.
    Wait(wait::Reaper),

    /// The reaper based on the signal backend.
    Signal(signal::Reaper),
}

/// The wrapper around a child.
pub(crate) enum ChildGuard {
    #[cfg(target_os = "linux")]
    /// The child guard based on the wait backend.
    Wait(wait::ChildGuard),

    /// The child guard based on the signal backend.
    Signal(signal::ChildGuard),
}

/// A lock on the reaper.
pub(crate) enum Lock {
    #[cfg(target_os = "linux")]
    /// The wait-based reaper needs no lock.
    Wait,

    /// The lock for the signal-based reaper.
    Signal(signal::Lock),
}

impl Reaper {
    /// Create a new reaper.
    pub(crate) fn new() -> Self {
        cfg_wait! {
            if wait::available() && !cfg!(async_process_force_signal_backend) {
                return Self::Wait(wait::Reaper::new());
            }
        }

        // Return the signal-based reaper.
        cfg_signal! {
            return Self::Signal(signal::Reaper::new());
        }

        #[allow(unreachable_code)]
        {
            panic!("neither the signal backend nor the waiter backend is available")
        }
    }

    /// Lock the driver thread.
    ///
    /// This makes it so only one thread can reap at once.
    pub(crate) async fn lock(&'static self) -> Lock {
        cfg_wait! {
            if let Self::Wait(_this) = self {
                // No locking needed.
                return Lock::Wait;
            }
        }

        cfg_signal! {
            if let Self::Signal(this) = self {
                // We need to lock.
                return Lock::Signal(this.lock().await);
            }
        }

        unreachable!()
    }

    /// Reap zombie processes forever.
    pub(crate) async fn reap(&'static self, lock: Lock) -> ! {
        cfg_wait! {
            if let (Self::Wait(this), Lock::Wait) = (self, &lock) {
                this.reap().await;
            }
        }

        cfg_signal! {
            if let (Self::Signal(this), Lock::Signal(lock)) = (self, lock) {
                this.reap(lock).await;
            }
        }

        unreachable!()
    }

    /// Register a child into this reaper.
    pub(crate) fn register(&'static self, child: std::process::Child) -> io::Result<ChildGuard> {
        cfg_wait! {
            if let Self::Wait(this) = self {
                return this.register(child).map(ChildGuard::Wait);
            }
        }

        cfg_signal! {
            if let Self::Signal(this) = self {
                return this.register(child).map(ChildGuard::Signal);
            }
        }

        unreachable!()
    }

    /// Wait for the inner child to complete.
    pub(crate) async fn status(
        &'static self,
        child: &Mutex<crate::ChildGuard>,
    ) -> io::Result<std::process::ExitStatus> {
        cfg_wait! {
            if let Self::Wait(this) = self {
                return this.status(child).await;
            }
        }

        cfg_signal! {
            if let Self::Signal(this) = self {
                return this.status(child).await;
            }
        }

        unreachable!()
    }

    /// Do we have any registered zombie processes?
    pub(crate) fn has_zombies(&'static self) -> bool {
        cfg_wait! {
            if let Self::Wait(this) = self {
                return this.has_zombies();
            }
        }

        cfg_signal! {
            if let Self::Signal(this) = self {
                return this.has_zombies();
            }
        }

        unreachable!()
    }
}

impl ChildGuard {
    /// Get a reference to the inner process.
    pub(crate) fn get_mut(&mut self) -> &mut std::process::Child {
        cfg_wait! {
            if let Self::Wait(this) = self {
                return this.get_mut();
            }
        }

        cfg_signal! {
            if let Self::Signal(this) = self {
                return this.get_mut();
            }
        }

        unreachable!()
    }

    /// Start reaping this child process.
    pub(crate) fn reap(&mut self, reaper: &'static Reaper) {
        cfg_wait! {
            if let (Self::Wait(this), Reaper::Wait(reaper)) = (&mut *self, reaper) {
                this.reap(reaper);
                return;
            }
        }

        cfg_signal! {
            if let (Self::Signal(this), Reaper::Signal(reaper)) = (self, reaper) {
                this.reap(reaper);
                return;
            }
        }

        unreachable!()
    }
}
