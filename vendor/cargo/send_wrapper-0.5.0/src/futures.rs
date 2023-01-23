//! [`Future`] and [`Stream`] support for [`SendWrapper`].
//!
//! [`Future`]: std::future::Future
//! [`Stream`]: futures_core::Stream

use std::{
	future::Future,
	ops::{Deref as _, DerefMut as _},
	pin::Pin,
	task,
};

use futures_core::Stream;

use super::SendWrapper;

const POLL_ERROR: &'static str =
	"Polling SendWrapper<T> variable from a thread different to the one it has been created with.";

impl<F: Future> Future for SendWrapper<F> {
	type Output = F::Output;

	/// Polls this [`SendWrapper`] [`Future`].
	///
	/// # Panics
	/// Polling panics if it is done from a different thread than the one the [`SendWrapper`]
	/// instance has been created with.
	fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
		if !self.valid() {
			panic!(POLL_ERROR);
		}
		// This is safe as `SendWrapper` itself points to the inner `Future`.
		// So, as long as `SendWrapper` is pinned, the inner `Future` is pinned too.
		unsafe { self.map_unchecked_mut(Self::deref_mut) }.poll(cx)
	}
}

impl<S: Stream> Stream for SendWrapper<S> {
	type Item = S::Item;

	/// Polls this [`SendWrapper`] [`Stream`].
	///
	/// # Panics
	/// Polling panics if it is done from a different thread than the one the [`SendWrapper`]
	/// instance has been created with.
	fn poll_next(
		self: Pin<&mut Self>,
		cx: &mut task::Context<'_>,
	) -> task::Poll<Option<Self::Item>> {
		if !self.valid() {
			panic!(POLL_ERROR);
		}
		// This is safe as `SendWrapper` itself points to the inner `Stream`.
		// So, as long as `SendWrapper` is pinned, the inner `Stream` is pinned too.
		unsafe { self.map_unchecked_mut(Self::deref_mut) }.poll_next(cx)
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.deref().size_hint()
	}
}

#[cfg(test)]
mod tests {
	use std::thread;

	use futures_executor as executor;
	use futures_util::{future, stream, StreamExt};

	use crate::SendWrapper;

	#[test]
	fn test_future() {
		let w1 = SendWrapper::new(future::ready(42));
		let w2 = w1.clone();
		assert_eq!(
			format!("{:?}", executor::block_on(w1)),
			format!("{:?}", executor::block_on(w2)),
		);
	}

	#[test]
	fn test_future_panic() {
		let w = SendWrapper::new(future::ready(42));
		let t = thread::spawn(move || executor::block_on(w));
		assert!(t.join().is_err());
	}

	#[test]
	fn test_stream() {
		let mut w1 = SendWrapper::new(stream::once(future::ready(42)));
		let mut w2 = SendWrapper::new(stream::once(future::ready(42)));
		assert_eq!(
			format!("{:?}", executor::block_on(w1.next())),
			format!("{:?}", executor::block_on(w2.next())),
		);
	}

	#[test]
	fn test_stream_panic() {
		let mut w = SendWrapper::new(stream::once(future::ready(42)));
		let t = thread::spawn(move || executor::block_on(w.next()));
		assert!(t.join().is_err());
	}
}
