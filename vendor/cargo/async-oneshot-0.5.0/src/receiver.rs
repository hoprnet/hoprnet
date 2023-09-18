use crate::*;
use core::{future::Future, pin::Pin};
use core::task::{Context, Poll};

/// The receiving half of a oneshot channel.
#[derive(Debug)]
pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
    done: bool,
}

impl<T> Receiver<T> {
    #[inline(always)]
    pub(crate) fn new(inner: Arc<Inner<T>>) -> Self {
        Receiver { inner, done: false }
    }

    /// Closes the channel by causing an immediate drop.
    #[inline(always)]
    pub fn close(self) { }

    #[inline(always)]
    fn handle_state(&mut self, state: crate::inner::State) -> Poll<Result<T, Closed>> {
        if state.ready() {
            Poll::Ready(Ok(self.inner.take_value()))
        } else if state.closed() {
            Poll::Ready(Err(Closed()))
        } else {
            Poll::Pending
        }.map(|x| {
            self.done = true;
            x
        })
    }

    /// Attempts to receive. On failure, if the channel is not closed,
    /// returns self to try again.
    #[inline]
    pub fn try_recv(mut self) -> Result<T, TryRecvError<T>> {
        let state = self.inner.state();
        match self.handle_state(state) {
            Poll::Ready(Ok(x)) => Ok(x),
            Poll::Ready(Err(Closed())) => Err(TryRecvError::Closed),
            Poll::Pending => Err(TryRecvError::Empty(self)),
        }
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, Closed>;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Result<T, Closed>> {
        let this = Pin::into_inner(self);
        match this.handle_state(this.inner.state()) {
            Poll::Pending => {},
            x => return x,
        }
        let state = this.inner.set_recv(ctx.waker().clone());
        match this.handle_state(state) {
            Poll::Pending => {},
            x => return x,
        }
        if state.send() { this.inner.send().wake_by_ref(); }
        Poll::Pending
    }
}

impl<T> Drop for Receiver<T> {
    #[inline(always)]
    fn drop(&mut self) {
        if !self.done {
            let state = self.inner.state();
            if !state.closed() && !state.ready() {
                let old = self.inner.close();
                if old.send() { self.inner.send().wake_by_ref(); }
            }
        }
    }
}
