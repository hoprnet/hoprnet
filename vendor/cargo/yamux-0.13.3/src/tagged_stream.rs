use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A stream that yields its tag with every item.
#[pin_project::pin_project]
pub struct TaggedStream<K, S> {
    key: K,
    #[pin]
    inner: S,

    reported_none: bool,
}

impl<K, S> TaggedStream<K, S> {
    pub fn new(key: K, inner: S) -> Self {
        Self {
            key,
            inner,
            reported_none: false,
        }
    }

    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<K, S> Stream for TaggedStream<K, S>
where
    K: Copy,
    S: Stream,
{
    type Item = (K, Option<S::Item>);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.reported_none {
            return Poll::Ready(None);
        }

        match futures::ready!(this.inner.poll_next(cx)) {
            Some(item) => Poll::Ready(Some((*this.key, Some(item)))),
            None => {
                *this.reported_none = true;

                Poll::Ready(Some((*this.key, None)))
            }
        }
    }
}
