use futures::{AsyncRead, Stream};
use rand::prelude::{thread_rng, Distribution};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RetryToken {
    pub num_retry: usize,
    pub started_at: Instant,
    backoff_base: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RetryResult {
    Wait(Duration),
    RetryNow(RetryToken),
    Expired,
}

impl RetryToken {
    pub fn new(now: Instant, backoff_base: f64) -> Self {
        Self {
            num_retry: 0,
            started_at: now,
            backoff_base,
        }
    }

    fn retry_in(&self, base: Duration, max_duration: Duration, jitter_dev: f64) -> Option<Duration> {
        let jitter_coeff = if jitter_dev > 0.0 {
            // Should not use jitter with sigma > 0.25
            rand_distr::Normal::new(1.0, jitter_dev.min(0.25))
                .unwrap()
                .sample(&mut thread_rng())
                .abs()
        } else {
            1.0
        };

        // jitter * base * backoff_base ^ num_retry
        let duration = base.mul_f64(jitter_coeff * self.backoff_base.powi(self.num_retry as i32));
        (duration < max_duration).then_some(duration)
    }

    pub fn check(&self, now: Instant, base: Duration, max: Duration, jitter_dev: f64) -> RetryResult {
        match self.retry_in(base, max, jitter_dev) {
            None => RetryResult::Expired,
            Some(retry_in) if self.started_at + retry_in >= now => RetryResult::Wait(self.started_at + retry_in - now),
            _ => RetryResult::RetryNow(Self {
                num_retry: self.num_retry + 1,
                started_at: self.started_at,
                backoff_base: self.backoff_base,
            }),
        }
    }
}

pub(crate) struct AsyncReadStreamer<R: AsyncRead + Unpin, const S: usize>(R);

impl<R: AsyncRead + Unpin, const S: usize> AsyncReadStreamer<R, S> {
    pub fn new(reader: R) -> Self {
        Self(reader)
    }
}

impl<R: AsyncRead + Unpin, const S: usize> Stream for AsyncReadStreamer<R, S> {
    type Item = std::io::Result<Box<[u8]>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = vec![0u8; S];

        match Pin::new(&mut self.0).poll_read(cx, &mut buffer) {
            Poll::Ready(result) => match result {
                Ok(size) => {
                    if size == 0 {
                        Poll::Ready(None)
                    } else {
                        buffer.truncate(size);
                        Poll::Ready(Some(Ok(buffer.into_boxed_slice())))
                    }
                }
                Err(err) => Poll::Ready(Some(Err(err))),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::TryStreamExt;

    #[async_std::test]
    async fn test_async_read_streamer_complete_chunk() {
        let data = b"Hello, World!!";
        let mut streamer = AsyncReadStreamer::<_, 14>::new(&data[..]);
        let mut results = Vec::new();

        while let Some(res) = streamer.try_next().await.unwrap() {
            results.push(res);
        }

        assert_eq!(results, vec![Box::from(*data)]);
    }

    #[async_std::test]
    async fn test_async_read_streamer_complete_more_chunks() {
        let data = b"Hello, World and do it twice";
        let mut streamer = AsyncReadStreamer::<_, 14>::new(&data[..]);
        let mut results = Vec::new();

        while let Some(res) = streamer.try_next().await.unwrap() {
            results.push(res);
        }

        let (data1, data2) = data.split_at(14);
        assert_eq!(results, vec![Box::from(data1), Box::from(data2)]);
    }

    #[async_std::test]
    async fn test_async_read_streamer_complete_more_chunks_with_incomplete() {
        let data = b"Hello, World and do it twice, ...";
        let streamer = AsyncReadStreamer::<_, 14>::new(&data[..]);

        let results = streamer.try_collect::<Vec<_>>().await.expect("should get chunks");

        let (data1, rest) = data.split_at(14);
        let (data2, data3) = rest.split_at(14);
        assert_eq!(results, vec![Box::from(data1), Box::from(data2), Box::from(data3)]);
    }

    #[async_std::test]
    async fn test_async_read_streamer_incomplete_chunk() {
        let data = b"Hello, World!!";
        let reader = &data[0..8]; // An incomplete chunk
        let mut streamer = AsyncReadStreamer::<_, 14>::new(reader);

        assert_eq!(
            Some(Box::from(reader)),
            streamer.try_next().await.expect("should get chunk")
        );
    }
}
