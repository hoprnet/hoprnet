use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;

use crate::FuturesUnorderedBounded;

/// A combined stream that releases values in any order that they come
///
/// # Example
///
/// ```
/// use std::future::ready;
/// use futures::stream::{self, StreamExt};
/// use futures::executor::block_on;
/// use futures_buffered::Merge;
///
/// block_on(async {
///     let a = stream::once(ready(2));
///     let b = stream::once(ready(3));
///     let c = stream::once(ready(5));
///     let d = stream::once(ready(7));
///     let mut s = Merge::from_iter([a, b, c, d]);
///
///     let mut counter = 0;
///     while let Some(n) = s.next().await {
///         counter += n;
///     }
///     assert_eq!(counter, 2+3+5+7);
/// })
/// ```
pub struct Merge<S> {
    pub(crate) streams: FuturesUnorderedBounded<S>,
}

impl<S> Merge<S> {
    /// Push a stream into the set.
    ///
    /// This method adds the given stream to the set. This method will not
    /// call [`poll_next`](futures_core::Stream::poll_next) on the submitted stream. The caller must
    /// ensure that [`Merge::poll_next`](Stream::poll_next) is called
    /// in order to receive wake-up notifications for the given stream.
    ///
    /// # Panics
    /// This method will panic if the buffer is currently full. See [`Merge::try_push`] to get a result instead
    #[track_caller]
    pub fn push(&mut self, stream: S) {
        if self.try_push(stream).is_err() {
            panic!("attempted to push into a full `Merge`")
        }
    }

    /// Push a future into the set.
    ///
    /// This method adds the given future to the set. This method will not
    /// call [`poll`](core::future::Future::poll) on the submitted future. The caller must
    /// ensure that [`FuturesUnorderedBounded::poll_next`](Stream::poll_next) is called
    /// in order to receive wake-up notifications for the given future.
    ///
    /// # Errors
    /// This method will error if the buffer is currently full, returning the future back
    pub fn try_push(&mut self, stream: S) -> Result<(), S> {
        self.streams.try_push_with(stream, core::convert::identity)
    }
}

impl<S: Stream> Stream for Merge<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.streams.poll_inner_no_remove(cx, S::poll_next) {
                // if we have a value from the stream, wake up that slot again
                Poll::Ready(Some((i, Some(x)))) => {
                    // safety: i is always within capacity
                    unsafe {
                        self.streams.shared.push(i);
                    }
                    break Poll::Ready(Some(x));
                }
                // if a stream completed, remove it from the queue
                Poll::Ready(Some((i, None))) => {
                    self.streams.tasks.remove(i);
                }
                Poll::Pending => break Poll::Pending,
                Poll::Ready(None) => break Poll::Ready(None),
            }
        }
    }
}

impl<S: Stream> FromIterator<S> for Merge<S> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = S>,
    {
        Self {
            streams: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;
    use core::task::Waker;

    use super::*;
    use alloc::collections::VecDeque;
    use alloc::rc::Rc;
    use futures::executor::block_on;
    use futures::executor::LocalPool;
    use futures::prelude::*;
    use futures::stream;
    use futures::task::LocalSpawnExt;

    #[test]
    fn merge_tuple_4() {
        block_on(async {
            let a = stream::repeat(2).take(2);
            let b = stream::repeat(3).take(3);
            let c = stream::repeat(5).take(5);
            let d = stream::repeat(7).take(7);
            let mut s: Merge<_> = [a, b, c, d].into_iter().collect();

            let mut counter = 0;
            while let Some(n) = s.next().await {
                counter += n;
            }
            assert_eq!(counter, 4 + 9 + 25 + 49);
        })
    }

    /// This test case uses channels so we'll have streams that return Pending from time to time.
    ///
    /// The purpose of this test is to make sure we have the waking logic working.
    #[test]
    fn merge_channels() {
        struct LocalChannel<T> {
            queue: VecDeque<T>,
            waker: Option<Waker>,
            closed: bool,
        }

        struct LocalReceiver<T> {
            channel: Rc<RefCell<LocalChannel<T>>>,
        }

        impl<T> Stream for LocalReceiver<T> {
            type Item = T;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut channel = self.channel.borrow_mut();

                match channel.queue.pop_front() {
                    Some(item) => Poll::Ready(Some(item)),
                    None => {
                        if channel.closed {
                            Poll::Ready(None)
                        } else {
                            channel.waker = Some(cx.waker().clone());
                            Poll::Pending
                        }
                    }
                }
            }
        }

        struct LocalSender<T> {
            channel: Rc<RefCell<LocalChannel<T>>>,
        }

        impl<T> LocalSender<T> {
            fn send(&self, item: T) {
                let mut channel = self.channel.borrow_mut();

                channel.queue.push_back(item);

                let _ = channel.waker.take().map(Waker::wake);
            }
        }

        impl<T> Drop for LocalSender<T> {
            fn drop(&mut self) {
                let mut channel = self.channel.borrow_mut();
                channel.closed = true;
                let _ = channel.waker.take().map(Waker::wake);
            }
        }

        fn local_channel<T>() -> (LocalSender<T>, LocalReceiver<T>) {
            let channel = Rc::new(RefCell::new(LocalChannel {
                queue: VecDeque::new(),
                waker: None,
                closed: false,
            }));

            (
                LocalSender {
                    channel: channel.clone(),
                },
                LocalReceiver { channel },
            )
        }

        let mut pool = LocalPool::new();

        let done = Rc::new(RefCell::new(false));
        let done2 = done.clone();

        pool.spawner()
            .spawn_local(async move {
                let (send1, receive1) = local_channel();
                let (send2, receive2) = local_channel();
                let (send3, receive3) = local_channel();

                let (count, ()) = futures::future::join(
                    async {
                        let s: Merge<_> = [receive1, receive2, receive3].into_iter().collect();
                        s.fold(0, |a, b| async move { a + b }).await
                    },
                    async {
                        for i in 1..=4 {
                            send1.send(i);
                            send2.send(i);
                            send3.send(i);
                        }
                        drop(send1);
                        drop(send2);
                        drop(send3);
                    },
                )
                .await;

                assert_eq!(count, 30);

                *done2.borrow_mut() = true;
            })
            .unwrap();

        while !*done.borrow() {
            pool.run_until_stalled()
        }
    }
}
