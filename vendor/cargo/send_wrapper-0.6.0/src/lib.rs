// Copyright 2017 Thomas Keh.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This [Rust] library implements a wrapper type called [`SendWrapper`] which allows you to move around non-[`Send`] types
//! between threads, as long as you access the contained value only from within the original thread. You also have to
//! make sure that the wrapper is dropped from within the original thread. If any of these constraints is violated,
//! a panic occurs. [`SendWrapper<T>`] implements [`Send`] and [`Sync`] for any type `T`.
//!
//! The idea for this library was born in the context of a [`GTK+`]/[`gtk-rs`]-based application. [`GTK+`] applications
//! are strictly single-threaded. It is not allowed to call any [`GTK+`] method from a thread different to the main
//! thread. Consequently, all [`gtk-rs`] structs are non-[`Send`].
//!
//! Sometimes you still want to do some work in background. It is possible to enqueue [`GTK+`] calls from there to be
//! executed in the main thread [using `Glib`]. This way you can know, that the [`gtk-rs`] structs involved are only
//! accessed in the main thread and will also be dropped there. This library makes it possible that [`gtk-rs`] structs
//! can leave the main thread at all.
//!
//! # Examples
//!
//! ```rust
//! use send_wrapper::SendWrapper;
//! use std::rc::Rc;
//! use std::thread;
//! use std::sync::mpsc::channel;
//!
//! // This import is important if you want to use deref() or
//! // deref_mut() instead of Deref coercion.
//! use std::ops::{Deref, DerefMut};
//!
//! // Rc is a non-Send type.
//! let value = Rc::new(42);
//!
//! // We now wrap the value with `SendWrapper` (value is moved inside).
//! let wrapped_value = SendWrapper::new(value);
//!
//! // A channel allows us to move the wrapped value between threads.
//! let (sender, receiver) = channel();
//!
//! let t = thread::spawn(move || {
//!
//!     // This would panic (because of dereferencing in wrong thread):
//!     // let value = wrapped_value.deref();
//!
//! 	// Move SendWrapper back to main thread, so it can be dropped from there.
//! 	// If you leave this out the thread will panic because of dropping from wrong thread.
//! 	sender.send(wrapped_value).unwrap();
//!
//! });
//!
//! let wrapped_value = receiver.recv().unwrap();
//!
//! // Now you can use the value again.
//! let value = wrapped_value.deref();
//!
//! // alternatives for dereferencing:
//! let value = &*wrapped_value;
//! let value: &Rc<_> = &wrapped_value;
//!
//! let mut wrapped_value = wrapped_value;
//! // alternatives for mutable dereferencing:
//! let value = wrapped_value.deref_mut();
//! let value = &mut *wrapped_value;
//! let value: &mut Rc<_> = &mut wrapped_value;
//! ```
//!
//! # Features
//!
//! This crate has a single feature called `futures` that enables [`Future`] and [`Stream`] implementations for [`SendWrapper`].
//! You can enable it in `Cargo.toml` like so:
//!
//! ```toml
//! send_wrapper = { version = "...", features = ["futures"] }
//! ```
//!
//! # License
//!
//! `send_wrapper` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).
//!
//! See LICENSE-APACHE.txt, and LICENSE-MIT.txt for details.
//!
//! [Rust]: https://www.rust-lang.org
//! [`gtk-rs`]: http://gtk-rs.org/
//! [`GTK+`]: https://www.gtk.org/
//! [using `Glib`]: http://gtk-rs.org/docs/glib/source/fn.idle_add.html
//! [`Future`]: std::future::Future
//! [`Stream`]: futures_core::Stream
// To build docs locally use `RUSTDOCFLAGS="--cfg docsrs" cargo doc --open --all-features`
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "futures")]
#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
mod futures;

use std::fmt;
use std::mem::{self, ManuallyDrop};
use std::ops::{Deref, DerefMut, Drop};
use std::thread::{self, ThreadId};

/// A wrapper which allows you to move around non-[`Send`]-types between threads, as long as you access the contained
/// value only from within the original thread and make sure that it is dropped from within the original thread.
pub struct SendWrapper<T> {
	data: ManuallyDrop<T>,
	thread_id: ThreadId,
}

impl<T> SendWrapper<T> {
	/// Create a `SendWrapper<T>` wrapper around a value of type `T`.
	/// The wrapper takes ownership of the value.
	pub fn new(data: T) -> SendWrapper<T> {
		SendWrapper {
			data: ManuallyDrop::new(data),
			thread_id: thread::current().id(),
		}
	}

	/// Returns `true` if the value can be safely accessed from within the current thread.
	pub fn valid(&self) -> bool {
		self.thread_id == thread::current().id()
	}

	/// Takes the value out of the `SendWrapper<T>`.
	///
	/// # Panics
	///
	/// Panics if it is called from a different thread than the one the `SendWrapper<T>` instance has
	/// been created with.
	#[track_caller]
	pub fn take(self) -> T {
		self.assert_valid_for_deref();

		// Prevent drop() from being called, as it would drop `self.data` twice
		let mut this = ManuallyDrop::new(self);

		// Safety:
		// - We've just checked that it's valid to access `T` from the current thread
		// - We only move out from `self.data` here and in drop, so `self.data` is present
		unsafe { ManuallyDrop::take(&mut this.data) }
	}

	#[track_caller]
	fn assert_valid_for_deref(&self) {
		if !self.valid() {
			invalid_deref()
		}
	}

	#[track_caller]
	fn assert_valid_for_poll(&self) {
		if !self.valid() {
			invalid_poll()
		}
	}
}

unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

impl<T> Deref for SendWrapper<T> {
	type Target = T;

	/// Returns a reference to the contained value.
	///
	/// # Panics
	///
	/// Dereferencing panics if it is done from a different thread than the one the `SendWrapper<T>` instance has been
	/// created with.
	#[track_caller]
	fn deref(&self) -> &T {
		self.assert_valid_for_deref();

		// Access the value.
		//
		// Safety: We just checked that it is valid to access `T` on the current thread.
		&*self.data
	}
}

impl<T> DerefMut for SendWrapper<T> {
	/// Returns a mutable reference to the contained value.
	///
	/// # Panics
	///
	/// Dereferencing panics if it is done from a different thread than the one the `SendWrapper<T>` instance has been
	/// created with.
	#[track_caller]
	fn deref_mut(&mut self) -> &mut T {
		self.assert_valid_for_deref();

		// Access the value.
		//
		// Safety: We just checked that it is valid to access `T` on the current thread.
		&mut *self.data
	}
}

impl<T> Drop for SendWrapper<T> {
	/// Drops the contained value.
	///
	/// # Panics
	///
	/// Dropping panics if it is done from a different thread than the one the `SendWrapper<T>` instance has been
	/// created with.
	///
	/// Exceptions:
	/// - There is no extra panic if the thread is already panicking/unwinding.
	///   This is because otherwise there would be double panics (usually resulting in an abort)
	///   when dereferencing from a wrong thread.
	/// - If `T` has a trivial drop ([`needs_drop::<T>()`] is false) then this method never panics.
	///
	/// [`needs_drop::<T>()`]: std::mem::needs_drop
	#[track_caller]
	fn drop(&mut self) {
		// If the drop is trivial (`needs_drop` = false), then dropping `T` can't access it
		// and so it can be safely dropped on any thread.
		if !mem::needs_drop::<T>() || self.valid() {
			unsafe {
				// Drop the inner value
				//
				// Safety:
				// - We've just checked that it's valid to drop `T` on this thread
				// - We only move out from `self.data` here and in drop, so `self.data` is present
				ManuallyDrop::drop(&mut self.data);
			}
		} else {
			invalid_drop()
		}
	}
}

impl<T: fmt::Debug> fmt::Debug for SendWrapper<T> {
	/// Formats the value using the given formatter.
	///
	/// # Panics
	///
	/// Formatting panics if it is done from a different thread than the one
	/// the `SendWrapper<T>` instance has been created with.
	#[track_caller]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("SendWrapper")
			.field("data", self.deref())
			.field("thread_id", &self.thread_id)
			.finish()
	}
}

impl<T: Clone> Clone for SendWrapper<T> {
	/// Returns a copy of the value.
	///
	/// # Panics
	///
	/// Cloning panics if it is done from a different thread than the one
	/// the `SendWrapper<T>` instance has been created with.
	#[track_caller]
	fn clone(&self) -> Self {
		Self::new(self.deref().clone())
	}
}

#[cold]
#[inline(never)]
#[track_caller]
fn invalid_deref() -> ! {
	const DEREF_ERROR: &'static str = "Dereferenced SendWrapper<T> variable from a thread different to the one it has been created with.";

	panic!("{}", DEREF_ERROR)
}

#[cold]
#[inline(never)]
#[track_caller]
fn invalid_poll() -> ! {
	const POLL_ERROR: &'static str = "Polling SendWrapper<T> variable from a thread different to the one it has been created with.";

	panic!("{}", POLL_ERROR)
}

#[cold]
#[inline(never)]
#[track_caller]
fn invalid_drop() {
	const DROP_ERROR: &'static str = "Dropped SendWrapper<T> variable from a thread different to the one it has been created with.";

	if !std::thread::panicking() {
		// panic because of dropping from wrong thread
		// only do this while not unwinding (could be caused by deref from wrong thread)
		panic!("{}", DROP_ERROR)
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Deref;
	use std::rc::Rc;
	use std::sync::mpsc::channel;
	use std::sync::Arc;
	use std::thread;

	use super::SendWrapper;

	#[test]
	fn test_deref() {
		let (sender, receiver) = channel();
		let w = SendWrapper::new(Rc::new(42));
		{
			let _x = w.deref();
		}
		let t = thread::spawn(move || {
			// move SendWrapper back to main thread, so it can be dropped from there
			sender.send(w).unwrap();
		});
		let w2 = receiver.recv().unwrap();
		{
			let _x = w2.deref();
		}
		assert!(t.join().is_ok());
	}

	#[test]
	fn test_deref_panic() {
		let w = SendWrapper::new(Rc::new(42));
		let t = thread::spawn(move || {
			let _x = w.deref();
		});
		let join_result = t.join();
		assert!(join_result.is_err());
	}

	#[test]
	fn test_drop_panic() {
		let w = SendWrapper::new(Rc::new(42));
		let t = thread::spawn(move || {
			let _x = w;
		});
		let join_result = t.join();
		assert!(join_result.is_err());
	}

	#[test]
	fn test_valid() {
		let w = SendWrapper::new(Rc::new(42));
		assert!(w.valid());
		thread::spawn(move || {
			assert!(!w.valid());
		});
	}

	#[test]
	fn test_take() {
		let w = SendWrapper::new(Rc::new(42));
		let inner: Rc<usize> = w.take();
		assert_eq!(42, *inner);
	}

	#[test]
	fn test_take_panic() {
		let w = SendWrapper::new(Rc::new(42));
		let t = thread::spawn(move || {
			let _ = w.take();
		});
		assert!(t.join().is_err());
	}

	#[test]
	fn test_sync() {
		// Arc<T> can only be sent to another thread if T Sync
		let arc = Arc::new(SendWrapper::new(42));
		thread::spawn(move || {
			let _ = arc;
		});
	}

	#[test]
	fn test_debug() {
		let w = SendWrapper::new(Rc::new(42));
		let info = format!("{:?}", w);
		assert!(info.contains("SendWrapper {"));
		assert!(info.contains("data: 42,"));
		assert!(info.contains("thread_id: ThreadId("));
	}

	#[test]
	fn test_debug_panic() {
		let w = SendWrapper::new(Rc::new(42));
		let t = thread::spawn(move || {
			let _ = format!("{:?}", w);
		});
		assert!(t.join().is_err());
	}

	#[test]
	fn test_clone() {
		let w1 = SendWrapper::new(Rc::new(42));
		let w2 = w1.clone();
		assert_eq!(format!("{:?}", w1), format!("{:?}", w2));
	}

	#[test]
	fn test_clone_panic() {
		let w = SendWrapper::new(Rc::new(42));
		let t = thread::spawn(move || {
			let _ = w.clone();
		});
		assert!(t.join().is_err());
	}
}
