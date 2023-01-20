SendWrapper
===========

This [Rust] crate implements a wrapper type called `SendWrapper` which allows you to move around non-[`Send`] types
between threads, as long as you access the contained value only from within the original thread. You also have to
make sure that the wrapper is dropped from within the original thread. If any of these constraints is violated,
a panic occurs.

The idea for this crate was born in the context of a [`GTK+`]/[`gtk-rs`]-based application. [`GTK+`] applications
are strictly single-threaded. It is not allowed to call any [`GTK+`] method from a thread different to the main
thread. Consequently, all [`gtk-rs`] structs are non-[`Send`].

Sometimes you still want to do some work in background. It is possible to enqueue [`GTK+`] calls from there to be
executed in the main thread [using `Glib`]. This way you can know, that the [`gtk-rs`] structs involved are only
accessed in the main thread and will also be dropped there. This crate makes it possible for [`gtk-rs`] structs
to leave the main thread.

# Examples

```rust
use send_wrapper::SendWrapper;
use std::rc::Rc;
use std::thread;
use std::sync::mpsc::channel;

// This import is important. It allows you to unwrap the value using deref(),
// deref_mut() or Deref coercion.
use std::ops::{Deref, DerefMut};

// Rc is a non-Send type.
let value = Rc::new(42);

// We now wrap the value with `SendWrapper` (value is moved inside).
let wrapped_value = SendWrapper::new(value);

// A channel allows us to move the wrapped value between threads.
let (sender, receiver) = channel();

let t = thread::spawn(move || {

	// This would panic (because of dereferencing in wrong thread):
	// let value = wrapped_value.deref();

	// Move SendWrapper back to main thread, so it can be dropped from there.
	// If you leave this out the thread will panic because of dropping from wrong thread.
	sender.send(wrapped_value).unwrap();

});

let wrapped_value = receiver.recv().unwrap();

// Now you can use the value again.
let value = wrapped_value.deref();

// alternatives for dereferencing:
// let value = *wrapped_value;
// let value: &NonSendType = &wrapped_value;

// alternatives for mutable dereferencing (value and wrapped_value must be mutable too, then):
// let mut value = wrapped_value.deref_mut();
// let mut value = &mut *wrapped_value;
// let mut value: &mut NonSendType = &mut wrapped_value;
```


## Wrapping `Future`s and `Stream`s

To use `SendWrapper` on `Future`s or `Stream`s, you should enable the Cargo feature `futures` first:
```toml
send_wrapper = { version = "0.5", features = ["futures"] }
```

Then, you can transparently wrap your `Future` or `Stream`:
```rust
use futures::{executor, future::{self, BoxFuture}};
use send_wrapper::SendWrapper;

// `Rc` is a `!Send` type,
let value = Rc::new(42);
// so this `Future` is `!Send` too as increments `Rc`'s inner counter.
let future = future::lazy(|_| value.clone());

// We now wrap the `future` with `SendWrapper` (value is moved inside),
let wrapped_future = SendWrapper::new(future);
// so now it's `Send` + `Sync` (`BoxFuture` trait object contains `Send` requirement).
let boxed_future: BoxFuture<_> = Box::pin(wrapped_future);

let t = thread::spawn(move || {
	// This would panic (because `future` is polled in wrong thread):
	// executor::block_on(boxed_future)
});
```


# Changelog

See [CHANGELOG.md](CHANGELOG.md)

# License

`send_wrapper` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE, and LICENSE-MIT for details.


[Rust]: https://www.rust-lang.org
[`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
[`gtk-rs`]: http://gtk-rs.org/
[`GTK+`]: https://www.gtk.org/
[using `Glib`]: http://gtk-rs.org/docs/glib/source/fn.idle_add.html
