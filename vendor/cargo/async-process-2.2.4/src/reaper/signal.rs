//! A version of the reaper that waits for a signal to check for process progress.

use async_lock::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard};
use event_listener::Event;
use futures_lite::future;

use std::io;
use std::mem;
use std::sync::Mutex;

pub(crate) type Lock = AsyncMutexGuard<'static, ()>;

/// The zombie process reaper.
pub(crate) struct Reaper {
    /// An event delivered every time the SIGCHLD signal occurs.
    sigchld: Event,

    /// The list of zombie processes.
    zombies: Mutex<Vec<std::process::Child>>,

    /// The pipe that delivers signal notifications.
    pipe: Pipe,

    /// Locking this mutex indicates that we are polling the SIGCHLD event.
    driver_guard: AsyncMutex<()>,
}

impl Reaper {
    /// Create a new reaper.
    pub(crate) fn new() -> Self {
        Reaper {
            sigchld: Event::new(),
            zombies: Mutex::new(Vec::new()),
            pipe: Pipe::new().expect("cannot create SIGCHLD pipe"),
            driver_guard: AsyncMutex::new(()),
        }
    }

    /// Lock the driver thread.
    pub(crate) async fn lock(&self) -> AsyncMutexGuard<'_, ()> {
        self.driver_guard.lock().await
    }

    /// Reap zombie processes forever.
    pub(crate) async fn reap(&'static self, _driver_guard: async_lock::MutexGuard<'_, ()>) -> ! {
        loop {
            // Wait for the next SIGCHLD signal.
            self.pipe.wait().await;

            // Notify all listeners waiting on the SIGCHLD event.
            self.sigchld.notify(usize::MAX);

            // Reap zombie processes, but make sure we don't hold onto the lock for too long!
            let mut zombies = mem::take(&mut *self.zombies.lock().unwrap());
            let mut i = 0;
            'reap_zombies: loop {
                for _ in 0..50 {
                    if i >= zombies.len() {
                        break 'reap_zombies;
                    }

                    if let Ok(None) = zombies[i].try_wait() {
                        i += 1;
                    } else {
                        zombies.swap_remove(i);
                    }
                }

                // Be a good citizen; yield if there are a lot of processes.
                //
                // After we yield, check if there are more zombie processes.
                future::yield_now().await;
                zombies.append(&mut self.zombies.lock().unwrap());
            }

            // Put zombie processes back.
            self.zombies.lock().unwrap().append(&mut zombies);
        }
    }

    /// Register a process with this reaper.
    pub(crate) fn register(&'static self, child: std::process::Child) -> io::Result<ChildGuard> {
        self.pipe.register(&child)?;
        Ok(ChildGuard { inner: Some(child) })
    }

    /// Wait for an event to occur for a child process.
    pub(crate) async fn status(
        &'static self,
        child: &Mutex<crate::ChildGuard>,
    ) -> io::Result<std::process::ExitStatus> {
        loop {
            // Wait on the child process.
            if let Some(status) = child.lock().unwrap().get_mut().try_wait()? {
                return Ok(status);
            }

            // Start listening.
            event_listener::listener!(self.sigchld => listener);

            // Try again.
            if let Some(status) = child.lock().unwrap().get_mut().try_wait()? {
                return Ok(status);
            }

            // Wait on the listener.
            listener.await;
        }
    }

    /// Do we have any registered zombie processes?
    pub(crate) fn has_zombies(&'static self) -> bool {
        !self
            .zombies
            .lock()
            .unwrap_or_else(|x| x.into_inner())
            .is_empty()
    }
}

/// The wrapper around the child.
pub(crate) struct ChildGuard {
    inner: Option<std::process::Child>,
}

impl ChildGuard {
    /// Get a mutable reference to the inner child.
    pub(crate) fn get_mut(&mut self) -> &mut std::process::Child {
        self.inner.as_mut().unwrap()
    }

    /// Begin the reaping process for this child.
    pub(crate) fn reap(&mut self, reaper: &'static Reaper) {
        if let Ok(None) = self.get_mut().try_wait() {
            reaper
                .zombies
                .lock()
                .unwrap()
                .push(self.inner.take().unwrap());
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        use async_channel::{Sender, Receiver, bounded};
        use std::ffi::c_void;
        use std::os::windows::io::AsRawHandle;
        use std::ptr;

        use windows_sys::Win32::{
            Foundation::{BOOLEAN, HANDLE},
            System::Threading::{
                RegisterWaitForSingleObject, INFINITE, WT_EXECUTEINWAITTHREAD, WT_EXECUTEONLYONCE,
            },
        };

        /// Waits for the next SIGCHLD signal.
        struct Pipe {
            /// The sender channel for the SIGCHLD signal.
            sender: Sender<()>,

            /// The receiver channel for the SIGCHLD signal.
            receiver: Receiver<()>,
        }

        impl Pipe {
            /// Creates a new pipe.
            fn new() -> io::Result<Pipe> {
                let (sender, receiver) = bounded(1);
                Ok(Pipe {
                    sender,
                    receiver
                })
            }

            /// Waits for the next SIGCHLD signal.
            async fn wait(&self) {
                self.receiver.recv().await.ok();
            }

            /// Register a process object into this pipe.
            fn register(&self, child: &std::process::Child) -> io::Result<()> {
                // Called when a child exits.
                #[allow(clippy::infallible_destructuring_match)]
                unsafe extern "system" fn callback(_: *mut c_void, _: BOOLEAN) {
                    let reaper = match &crate::Reaper::get().sys {
                        super::Reaper::Signal(reaper) => reaper,
                    };

                    reaper.pipe.sender.try_send(()).ok();
                }

                // Register this child process to invoke `callback` on exit.
                let mut wait_object = ptr::null_mut();
                let ret = unsafe {
                    RegisterWaitForSingleObject(
                        &mut wait_object,
                        child.as_raw_handle() as HANDLE,
                        Some(callback),
                        std::ptr::null_mut(),
                        INFINITE,
                        WT_EXECUTEINWAITTHREAD | WT_EXECUTEONLYONCE,
                    )
                };

                if ret == 0 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(())
                }
            }
        }
    } else if #[cfg(unix)] {
        use async_signal::{Signal, Signals};
        use futures_lite::prelude::*;

        /// Waits for the next SIGCHLD signal.
        struct Pipe {
            /// The iterator over SIGCHLD signals.
            signals: Signals,
        }

        impl Pipe {
            /// Creates a new pipe.
            fn new() -> io::Result<Pipe> {
                Ok(Pipe {
                    signals: Signals::new(Some(Signal::Child))?,
                })
            }

            /// Waits for the next SIGCHLD signal.
            async fn wait(&self) {
                (&self.signals).next().await;
            }

            /// Register a process object into this pipe.
            fn register(&self, _child: &std::process::Child) -> io::Result<()> {
                Ok(())
            }
        }
    }
}
