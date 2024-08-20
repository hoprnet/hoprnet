use alloc::vec::Vec;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;

use crate::{futures_unordered::MIN_CAPACITY, FuturesUnorderedBounded, MergeBounded};

/// A combined stream that releases values in any order that they come.
///
/// This differs from [`crate::Merge`] in that [`MergeUnbounded`] does not have a fixed capacity
/// but instead grows on demand. It uses [`crate::FuturesUnordered`] under the hood.
///
/// # Example
///
/// ```
/// use std::future::ready;
/// use futures::stream::{self, StreamExt};
/// use futures::executor::block_on;
/// use futures_buffered::MergeUnbounded;
///
/// block_on(async {
///     let a = stream::once(ready(2));
///     let b = stream::once(ready(3));
///     let mut s = MergeUnbounded::from_iter([a, b]);
///
///     let mut counter = 0;
///     while let Some(n) = s.next().await {
///         if n == 3 {
///             s.push(stream::once(ready(4)));
///         }
///         counter += n;
///     }
///     assert_eq!(counter, 2+3+4);
/// })
/// ```
pub struct MergeUnbounded<S> {
    pub(crate) groups: Vec<MergeBounded<S>>,
    poll_next: usize,
}

impl<S> Default for MergeUnbounded<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> MergeUnbounded<S> {
    /// Create a new, empty [`MergeUnbounded`].
    ///
    /// Calling [`poll_next`](futures_core::Stream::poll_next) will return `Poll::Ready(None)`
    /// until a stream is added with [`Self::push`].
    pub const fn new() -> Self {
        Self {
            groups: Vec::new(),
            poll_next: 0,
        }
    }

    /// Push a stream into the set.
    ///
    /// This method adds the given stream to the set. This method will not
    /// call [`poll_next`](futures_core::Stream::poll_next) on the submitted stream. The caller must
    /// ensure that [`MergeUnbounded::poll_next`](Stream::poll_next) is called
    /// in order to receive wake-up notifications for the given stream.
    #[track_caller]
    pub fn push(&mut self, stream: S) {
        let last = match self.groups.last_mut() {
            Some(last) => last,
            None => {
                self.groups.push(MergeBounded {
                    streams: FuturesUnorderedBounded::new(MIN_CAPACITY),
                });
                self.groups.last_mut().unwrap()
            }
        };
        match last.try_push(stream) {
            Ok(()) => {}
            Err(stream) => {
                let mut next = MergeBounded {
                    streams: FuturesUnorderedBounded::new(last.streams.capacity() * 2),
                };
                next.push(stream);
                self.groups.push(next);
            }
        }
    }
}

impl<S: Stream + Unpin> Stream for MergeUnbounded<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            // rem,
            groups,
            poll_next,
        } = &mut *self;
        if groups.is_empty() {
            return Poll::Ready(None);
        }

        for _ in 0..groups.len() {
            if *poll_next >= groups.len() {
                *poll_next = 0;
            }

            let poll = Pin::new(&mut groups[*poll_next]).poll_next(cx);
            match poll {
                Poll::Ready(Some(x)) => {
                    return Poll::Ready(Some(x));
                }
                Poll::Ready(None) => {
                    let group = groups.remove(*poll_next);
                    debug_assert!(group.streams.is_empty());

                    if groups.is_empty() {
                        // group should contain at least 1 set
                        groups.push(group);
                        return Poll::Ready(None);
                    }

                    // we do not want to drop the last set as it contains
                    // the largest allocation that we want to keep a hold of
                    if *poll_next == groups.len() {
                        groups.push(group);
                        *poll_next = 0;
                    }
                }
                Poll::Pending => {
                    *poll_next += 1;
                }
            }
        }
        Poll::Pending
    }
}

impl<S: Stream + Unpin> FromIterator<S> for MergeUnbounded<S> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = S>,
    {
        let iter = iter.into_iter();
        // let mut this =
        //     Self::with_capacity(usize::max(iter.size_hint().0, MIN_CAPACITY));
        let mut this = Self::new();
        for stream in iter {
            this.push(stream);
        }
        this
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
    use futures::stream;
    use futures::task::LocalSpawnExt;
    use futures::StreamExt;

    #[test]
    fn merge_tuple_4() {
        block_on(async {
            let a = stream::repeat(2).take(2);
            let b = stream::repeat(3).take(3);
            let c = stream::repeat(5).take(5);
            let d = stream::repeat(7).take(7);
            let mut s: MergeUnbounded<_> = [a, b, c, d].into_iter().collect();

            let mut counter = 0;
            while let Some(n) = s.next().await {
                counter += n;
            }
            assert_eq!(counter, 4 + 9 + 25 + 49);
        })
    }

    #[test]
    fn add_streams() {
        block_on(async {
            let a = stream::repeat(2).take(2);
            let b = stream::repeat(3).take(3);
            let mut s = MergeUnbounded::default();
            assert_eq!(s.next().await, None);

            s.push(a);
            s.push(b);

            let mut counter = 0;
            while let Some(n) = s.next().await {
                counter += n;
            }

            let b = stream::repeat(4).take(4);
            s.push(b);

            while let Some(n) = s.next().await {
                counter += n;
            }

            assert_eq!(counter, 4 + 9 + 16);
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
                        let s: MergeUnbounded<_> =
                            [receive1, receive2, receive3].into_iter().collect();
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
