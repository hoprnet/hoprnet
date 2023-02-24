// Copyright 2017 Thomas Keh.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This [Rust] library implements a wrapper type called `SendWrapper` which allows you to move around non-[`Send`] types
//! between threads, as long as you access the contained value only from within the original thread. You also have to
//! make sure that the wrapper is dropped from within the original thread. If any of these constraints is violated,
//! a panic occurs.
//!
//! The idea for this library was born in the context of a [`GTK+`]/[`gtk-rs`]-based application. [`GTK+`] applications
//! are strictly single-threaded. It is not allowed to call any [`GTK+`] method from a thread different to the main
//! thread. Consequently, all [`gtk-rs`] structs are non-[`Send`].
//!
//! Sometimes you still want to do some work in background. It is possible to enqueue [`GTK+`] calls from there to be
//! executed in the main thread [using `Glib`]. This way you can know, that the [`gtk-rs`] structs involved are only
//! accessed in the main thread and will also be dropped there. This library makes it possible that [`gtk-rs`] structs
//! can leave the main thread at all, like required in the given
//!
//! # Examples
//!
//! ```rust
//! use send_wrapper::SendWrapper;
//! use std::rc::Rc;
//! use std::thread;
//! use std::sync::mpsc::channel;
//!
//! // This import is important. It allows you to unwrap the value using deref(),
//! // deref_mut() or Deref coercion.
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
//!// This would panic (because of dereferencing in wrong thread):
//!// let value = wrapped_value.deref();
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
//! // let value = *wrapped_value;
//! // let value: &NonSendType = &wrapped_value;
//!
//! // alternatives for mutable dereferencing (value and wrapped_value must be mutable too, then):
//! // let mut value = wrapped_value.deref_mut();
//! // let mut value = &mut *wrapped_value;
//! // let mut value: &mut NonSendType = &mut wrapped_value;
//! ```
//!
//! # License
//!
//! `send_wrapper` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).
//!
//! See LICENSE-APACHE.txt, and LICENSE-MIT.txt for details.
//!
//! [Rust]: https://www.rust-lang.org
//! [`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
//! [`gtk-rs`]: http://gtk-rs.org/
//! [`GTK+`]: https://www.gtk.org/
//! [using `Glib`]: http://gtk-rs.org/docs/glib/source/fn.idle_add.html

use std::ops::{Drop,Deref,DerefMut};
use std::marker::Send;
use std::thread;
use std::thread::ThreadId;

const DEREF_ERROR: &'static str = "Dropped SendWrapper<T> variable from a thread different to the one it has been created with.";
const DROP_ERROR: &'static str = "Dereferenced SendWrapper<T> variable from a thread different to the one it has been created with.";

/// A wrapper which allows you to move around non-[`Send`]-types between threads, as long as you access the contained
/// value only from within the original thread and make sure that it is dropped from within the original thread.
pub struct SendWrapper<T> {
	data: *mut T,
	thread_id: ThreadId,
}

impl<T> SendWrapper<T> {

	/// Create a SendWrapper<T> wrapper around a value of type T.
	/// The wrapper takes ownership of the value.
	pub fn new(data: T) -> SendWrapper<T> {
		SendWrapper {
			data: Box::into_raw(Box::new(data)),
			thread_id: thread::current().id()
		}
	}

	/// Returns if the value can be safely accessed from within the current thread.
	pub fn valid(&self) -> bool {
		self.thread_id == thread::current().id()
	}

	/// Takes the value out of the SendWrapper.
	///
	/// #Panics
	/// Panics if it is called from a different thread than the one the SendWrapper<T> instance has
	/// been created with.
	pub fn take(self) -> T {
		if !self.valid() {
			panic!(DEREF_ERROR);
		}
		let result = unsafe { Box::from_raw(self.data) };
		// Prevent drop() from being called, as it would drop self.data twice
		std::mem::forget(self);
		*result
	}
}

unsafe impl<T> Send for SendWrapper<T> { }
unsafe impl<T> Sync for SendWrapper<T> { }

impl<T> Deref for SendWrapper<T> {
	type Target = T;

	/// Returns a reference to the contained value.
	///
	/// # Panics
	/// Derefencing panics if it is done from a different thread than the one the SendWrapper<T> instance has been
	/// created with.
	fn deref(&self) -> &T {
		if !self.valid() {
			panic!(DEREF_ERROR);
		}
		unsafe {
			// Access the value. We just checked that it is valid.
			&*self.data
		}
	}
}

impl<T> DerefMut for SendWrapper<T> {

	/// Returns a mutable reference to the contained value.
	///
	/// # Panics
	/// Derefencing panics if it is done from a different thread than the one the SendWrapper<T> instance has been
	/// created with.
	fn deref_mut(&mut self) -> &mut T {
		if !self.valid() {
			panic!(DEREF_ERROR);
		}
		unsafe {
			// Access the value. We just checked that it is valid.
			&mut *self.data
		}
	}
}

impl<T> Drop for SendWrapper<T> {

	/// Drops the contained value.
	///
	/// # Panics
	/// Dropping panics if it is done from a different thread than the one the SendWrapper<T> instance has been
	/// created with. As an exception, there is no extra panic if the thread is already panicking/unwinding. This is
	/// because otherwise there would be double panics (usually resulting in an abort) when dereferencing from a wrong
	/// thread.
	fn drop(&mut self) {
		if self.valid() {
			unsafe {
				// Create a boxed value from the raw pointer. We just checked that the pointer is valid.
				// Box handles the dropping for us when _dropper goes out of scope.
				let _dropper = Box::from_raw(self.data);
			}
		} else {
			if !std::thread::panicking() {
				// panic because of dropping from wrong thread
				// only do this while not unwinding (coud be caused by deref from wrong thread)
				panic!(DROP_ERROR);
			}
		}
	}
}

#[cfg(test)]
mod tests {

	use SendWrapper;
	use std::thread;
	use std::sync::mpsc::channel;
	use std::ops::Deref;
	use std::rc::Rc;

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

}
