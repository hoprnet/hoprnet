use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use crate::prelude::errors::SessionError;
use crate::prelude::{Frame, FrameId, Segment};
use crate::session::frame::SeqNum;

#[derive(Debug)]
struct FrameBuilder {
    segments: BTreeSet<Segment>,
    last_recv: Instant,
}

impl From<Segment> for FrameBuilder {
    fn from(value: Segment) -> Self {
        Self {
            segments: BTreeSet::from_iter([value]),
            last_recv: Instant::now(),
        }
    }
}

impl FrameBuilder {
    pub fn add_segment(&mut self, segment: Segment) {
        self.segments.insert(segment);
        self.last_recv = Instant::now();
    }

    pub fn frame_id(&self) -> FrameId {
        self.segments.first().unwrap().frame_id
    }

    pub fn remaining(&self) -> SeqNum {
        self.segments.first().unwrap().seq_len - self.segments.len() as SeqNum
    }

    pub fn build(self) -> Result<Frame, SessionError> {
        let frame_id = self.frame_id();

        if self.remaining() > 0 {
            return Err(SessionError::IncompleteFrame(frame_id));
        }

        Ok(Frame {
            frame_id,
            data: self
                .segments
                .into_iter()
                .flat_map(|s| s.data.into_vec())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }
}

#[derive(Debug)]
#[pin_project::pin_project]
pub struct Reassembler {
    frames: HashMap<FrameId, FrameBuilder>,
    complete_frames: VecDeque<Result<Frame, SessionError>>,
    tx_waker: Option<Waker>,
    rx_waker: Option<Waker>,
    is_closed: bool,
    max_age: Duration,
    capacity: usize,
}

impl Reassembler {
    pub const INCOMPLETE_FRAME_RATIO: usize = 2;

    pub fn new(max_age: Duration, capacity: usize) -> Self {
        Self {
            frames: HashMap::with_capacity(Self::INCOMPLETE_FRAME_RATIO * capacity + 1),
            complete_frames: VecDeque::with_capacity(capacity),
            tx_waker: None,
            rx_waker: None,
            is_closed: false,
            max_age,
            capacity,
        }
    }

    fn expire_frames(&mut self) -> usize {
        let mut expired = 0;
        self.frames.retain(|id, b| {
            if b.last_recv.elapsed() >= self.max_age || self.is_closed {
                self.complete_frames.push_back(Err(SessionError::FrameDiscarded(*id)));
                tracing::trace!("frame {id} discarded");
                expired += 1;
                false
            } else {
                true
            }
        });

        expired
    }
}

impl futures::Sink<Segment> for Reassembler {
    type Error = SessionError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tracing::trace!("Reassembler::poll_ready");
        if self.is_closed {
            return Poll::Ready(Err(SessionError::ReassemblerClosed));
        }

        // If we're at the capacity of incomplete frames,
        // check if we can expire more than the amount that's over the capacity.
        // Otherwise, give up.
        if self.frames.len() >= self.capacity * Self::INCOMPLETE_FRAME_RATIO {
            if self.expire_frames() <= self.frames.len() - self.capacity {
                return Poll::Ready(Err(SessionError::TooManyIncompleteFrames));
            }
        }

        if self.complete_frames.len() >= self.capacity {
            self.rx_waker = Some(cx.waker().clone());

            // Give the stream a chance to yield an element
            if let Some(waker) = self.tx_waker.take() {
                waker.wake();
            }

            tracing::trace!("Reassembler::poll_ready pending");
            Poll::Pending
        } else {
            tracing::trace!(
                "Reassembler::poll_ready ready (remaining {})",
                self.capacity - self.complete_frames.len()
            );
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Segment) -> Result<(), Self::Error> {
        if self.is_closed {
            return Err(SessionError::ReassemblerClosed);
        }

        let maybe_frame = match self.frames.entry(item.frame_id) {
            Entry::Occupied(mut e) => {
                let builder = e.get_mut();
                builder.add_segment(item);
                if builder.remaining() == 0 {
                    tracing::trace!("frame {} is complete", e.key());
                    Some(e.remove().build())
                } else {
                    None
                }
            }
            Entry::Vacant(e) => {
                let builder = FrameBuilder::from(item);
                if builder.remaining() == 0 {
                    tracing::trace!("single segment frame {} is complete", builder.frame_id());
                    Some(builder.build())
                } else {
                    e.insert(builder);
                    None
                }
            }
        };

        if let Some(frame) = maybe_frame {
            self.complete_frames.push_back(frame);
            if let Some(waker) = self.tx_waker.take() {
                waker.wake();
            }
        }

        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tracing::trace!("Reassembler::poll_flush");
        if self.is_closed {
            return Poll::Ready(Err(SessionError::ReassemblerClosed));
        }

        if let Some(waker) = self.tx_waker.take() {
            waker.wake();
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tracing::trace!("Reassembler::poll_close");
        if self.is_closed {
            return Poll::Ready(Err(SessionError::ReassemblerClosed));
        }

        self.is_closed = true;
        if let Some(waker) = self.tx_waker.take() {
            waker.wake();
        }
        if let Some(waker) = self.rx_waker.take() {
            waker.wake();
        }

        Poll::Ready(Ok(()))
    }
}

impl futures::Stream for Reassembler {
    type Item = Result<Frame, SessionError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        tracing::trace!("Reassembler::poll_next");
        self.expire_frames();

        if self.is_closed && self.complete_frames.is_empty() {
            tracing::trace!("Reassembler::poll_next done");
            return Poll::Ready(None);
        }

        if let Some(complete) = self.complete_frames.pop_front() {
            if let Some(waker) = self.rx_waker.take() {
                waker.wake();
            }
            Poll::Ready(Some(
                complete
                    .inspect(|f| tracing::trace!("Reassembler::poll_next ready {}", f.frame_id))
                    .inspect_err(|e| tracing::trace!("Reassembler::poll_next ready error ({e})")),
            ))
        } else if !self.is_closed {
            // The next sink operation will wake us up, but only if we give it a chance
            self.tx_waker = Some(cx.waker().clone());

            if let Some(waker) = self.rx_waker.take() {
                waker.wake();
            }

            tracing::trace!("Reassembler::poll_next pending");
            Poll::Pending
        } else {
            tracing::trace!("Reassembler::poll_next done");
            Poll::Ready(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::prelude::FutureExt;
    use futures::{pin_mut, SinkExt, StreamExt, TryStreamExt};
    use hex_literal::hex;
    use rand::prelude::SliceRandom;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    const RNG_SEED: [u8; 32] = hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

    #[test_log::test(async_std::test)]
    pub async fn reassembler_should_reassemble_frames() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = Reassembler::new(Duration::from_secs(5), 1024).split();

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| f.segment(22).unwrap())
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let jh = hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

        let mut actual = r_stream
            .try_collect::<Vec<_>>()
            .timeout(Duration::from_secs(5))
            .await??;

        assert_eq!(actual.len(), expected.len());

        actual.sort();
        assert_eq!(actual, expected);

        Ok(jh.await?)
    }

    #[test_log::test(async_std::test)]
    pub async fn reassembler_should_discard_incomplete_frames_on_expiration() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = Reassembler::new(Duration::from_millis(45), 1024).split();

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| f.segment(22).unwrap())
            .filter(|s| s.frame_id != 2 || s.seq_idx != 1)
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let seg_count = segments.len();
        let jh = hopr_async_runtime::prelude::spawn(
            futures::stream::iter(segments)
                .enumerate()
                .then(move |(i, s)| {
                    async move {
                        // Delay the very last segment,
                        // so the incomplete frame gets a chance to expire
                        if i == seg_count - 1 {
                            futures::future::ok(s).delay(Duration::from_millis(55)).await
                        } else {
                            Ok(s)
                        }
                    }
                })
                .forward(r_sink),
        );

        let mut actual = r_stream.collect::<Vec<_>>().timeout(Duration::from_secs(5)).await?;

        assert_eq!(actual.len(), expected.len());

        actual.sort_by(|a, b| match (a, b) {
            (Ok(a), Ok(b)) => a.frame_id.cmp(&b.frame_id),
            (Err(SessionError::FrameDiscarded(a)), Ok(b)) => a.cmp(&b.frame_id),
            (Ok(a), Err(SessionError::FrameDiscarded(b))) => a.frame_id.cmp(b),
            (Err(SessionError::FrameDiscarded(a)), Err(SessionError::FrameDiscarded(b))) => a.cmp(b),
            _ => panic!("unexpected result"),
        });

        for i in 0..expected.len() {
            if i != 1 {
                assert!(matches!(&actual[i], Ok(f) if *f == expected[i]));
            } else {
                assert!(matches!(actual[i], Err(SessionError::FrameDiscarded(2))));
            }
        }

        Ok(jh.await?)
    }

    #[test_log::test(async_std::test)]
    pub async fn reassembler_should_discard_incomplete_frames_on_close() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = Reassembler::new(Duration::from_millis(100), 1024).split();

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| f.segment(22).unwrap())
            .filter(|s| s.frame_id != 5 || s.seq_idx != 2)
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let jh = hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

        let mut actual = r_stream.collect::<Vec<_>>().timeout(Duration::from_secs(5)).await?;

        assert_eq!(actual.len(), expected.len());

        actual.sort_by(|a, b| match (a, b) {
            (Ok(a), Ok(b)) => a.frame_id.cmp(&b.frame_id),
            (Err(SessionError::FrameDiscarded(a)), Ok(b)) => a.cmp(&b.frame_id),
            (Ok(a), Err(SessionError::FrameDiscarded(b))) => a.frame_id.cmp(b),
            (Err(SessionError::FrameDiscarded(a)), Err(SessionError::FrameDiscarded(b))) => a.cmp(b),
            _ => panic!("unexpected result"),
        });

        for i in 0..expected.len() {
            if i != 4 {
                assert!(matches!(&actual[i], Ok(f) if *f == expected[i]));
            } else {
                assert!(matches!(actual[i], Err(SessionError::FrameDiscarded(5))));
            }
        }

        Ok(jh.await?)
    }

    #[test_log::test(async_std::test)]
    pub async fn reassembler_should_wait_if_full() -> anyhow::Result<()> {
        let expected = (1u32..=5)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<30>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = Reassembler::new(Duration::from_secs(5), 3).split();

        pin_mut!(r_sink);
        pin_mut!(r_stream);

        let segments = expected
            .iter()
            .cloned()
            .flat_map(|f| f.segment(20).unwrap())
            .collect::<Vec<_>>();

        // Frame 1
        r_sink.send(segments[1].clone()).await?;
        r_sink.send(segments[0].clone()).await?;

        // Frame 2
        r_sink.send(segments[2].clone()).await?;
        r_sink.send(segments[3].clone()).await?;

        // Frame 3
        r_sink.send(segments[5].clone()).await?;
        r_sink.send(segments[4].clone()).await?;

        // Cannot insert more segments until some frames are yielded
        assert!(r_sink
            .send(segments[7].clone())
            .timeout(Duration::from_millis(50))
            .await
            .is_err());

        // Yield Frame 1
        assert_eq!(Some(expected[0].clone()), r_stream.try_next().await?);

        // This inserts segment 7 (from SplitSink's slot) and segment 6, thus completing Frame 4
        r_sink.send(segments[6].clone()).await?;

        // Cannot insert more segments until some frames are yielded
        assert!(r_sink
            .send(segments[8].clone())
            .timeout(Duration::from_millis(50))
            .await
            .is_err());

        // Yield Frame 2
        assert_eq!(Some(expected[1].clone()), r_stream.try_next().await?);

        // Yield Frame 3
        assert_eq!(Some(expected[2].clone()), r_stream.try_next().await?);

        // Completes Frame 5 (via SplitSink's slot)
        r_sink.send(segments[9].clone()).await?;
        r_sink.close().await?;

        assert_eq!(Some(expected[3].clone()), r_stream.try_next().await?);
        assert_eq!(Some(expected[4].clone()), r_stream.try_next().await?);
        assert_eq!(None, r_stream.try_next().await?);

        Ok(())
    }

    #[test_log::test(async_std::test)]
    pub async fn reassembler_should_fail_with_too_many_incomplete_frames() -> anyhow::Result<()> {
        let expected = (1u32..=5)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<30>().into(),
            })
            .collect::<Vec<_>>();

        let reassembler = Reassembler::new(
            Duration::from_secs(5),
            2, // = max 4 incomplete frames before fail
        );

        pin_mut!(reassembler);

        let segments = expected
            .iter()
            .cloned()
            .flat_map(|f| f.segment(20).unwrap())
            .collect::<Vec<_>>();

        reassembler.send(segments[0].clone()).await?;
        reassembler.send(segments[2].clone()).await?;
        reassembler.send(segments[4].clone()).await?;
        reassembler.send(segments[6].clone()).await?;

        assert!(matches!(
            reassembler.send(segments[7].clone()).await,
            Err(SessionError::TooManyIncompleteFrames)
        ));

        Ok(())
    }
}
