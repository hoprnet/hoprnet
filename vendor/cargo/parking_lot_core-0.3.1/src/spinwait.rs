// Copyright 2016 Amanieu d'Antras
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[cfg(unix)]
use libc;
use std::sync::atomic::spin_loop_hint;
#[cfg(not(any(windows, unix)))]
use std::thread;
#[cfg(windows)]
use winapi;

// Yields the rest of the current timeslice to the OS
#[cfg(windows)]
#[inline]
fn thread_yield() {
    // Note that this is manually defined here rather than using the definition
    // through `winapi`. The `winapi` definition comes from the `synchapi`
    // header which enables the "synchronization.lib" library. It turns out,
    // however that `Sleep` comes from `kernel32.dll` so this activation isn't
    // necessary.
    //
    // This was originally identified in rust-lang/rust where on MinGW the
    // libsynchronization.a library pulls in a dependency on a newer DLL not
    // present in older versions of Windows. (see rust-lang/rust#49438)
    //
    // This is a bit of a hack for now and ideally we'd fix MinGW's own import
    // libraries, but that'll probably take a lot longer than patching this here
    // and avoiding the `synchapi` feature entirely.
    extern "system" {
        fn Sleep(a: winapi::shared::minwindef::DWORD);
    }
    unsafe {
        // We don't use SwitchToThread here because it doesn't consider all
        // threads in the system and the thread we are waiting for may not get
        // selected.
        Sleep(0);
    }
}
#[cfg(unix)]
#[inline]
fn thread_yield() {
    unsafe {
        libc::sched_yield();
    }
}
#[cfg(not(any(windows, unix)))]
#[inline]
fn thread_yield() {
    thread::yield_now();
}

// Wastes some CPU time for the given number of iterations,
// using a hint to indicate to the CPU that we are spinning.
#[inline]
fn cpu_relax(iterations: u32) {
    for _ in 0..iterations {
        spin_loop_hint()
    }
}

/// A counter used to perform exponential backoff in spin loops.
pub struct SpinWait {
    counter: u32,
}

impl SpinWait {
    /// Creates a new `SpinWait`.
    #[inline]
    pub fn new() -> SpinWait {
        SpinWait { counter: 0 }
    }

    /// Resets a `SpinWait` to its initial state.
    #[inline]
    pub fn reset(&mut self) {
        self.counter = 0;
    }

    /// Spins until the sleep threshold has been reached.
    ///
    /// This function returns whether the sleep threshold has been reached, at
    /// which point further spinning has diminishing returns and the thread
    /// should be parked instead.
    ///
    /// The spin strategy will initially use a CPU-bound loop but will fall back
    /// to yielding the CPU to the OS after a few iterations.
    #[inline]
    pub fn spin(&mut self) -> bool {
        if self.counter >= 10 {
            return false;
        }
        self.counter += 1;
        if self.counter <= 3 {
            cpu_relax(1 << self.counter);
        } else {
            thread_yield();
        }
        true
    }

    /// Spins without yielding the thread to the OS.
    ///
    /// Instead, the backoff is simply capped at a maximum value. This can be
    /// used to improve throughput in `compare_exchange` loops that have high
    /// contention.
    #[inline]
    pub fn spin_no_yield(&mut self) {
        self.counter += 1;
        if self.counter > 10 {
            self.counter = 10;
        }
        cpu_relax(1 << self.counter);
    }
}

impl Default for SpinWait {
    #[inline]
    fn default() -> SpinWait {
        SpinWait::new()
    }
}
