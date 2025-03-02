use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use crate::prelude::errors::SessionError;
use crate::prelude::{Frame, Segment};
use crate::session::frames::{
    FrameBuilder, FrameDashMap, FrameInspector, FrameMap, FrameMapEntry, FrameMapOccupiedEntry, FrameMapVacantEntry,
};

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
#[pin_project::pin_project]
pub struct Reassembler<M> {
    incomplete_frames: M,
    complete_frames: VecDeque<Result<Frame, SessionError>>,
    tx_waker: Option<Waker>,
    rx_waker: Option<Waker>,
    is_closed: bool,
    max_age: Duration,
    capacity: usize,
    last_expiration: Instant,
}

impl<M: FrameMap> Reassembler<M> {
    /// Indicates how many incomplete frames there could be per one complete/discarded frame.
    const INCOMPLETE_FRAME_RATIO: usize = 2;

    pub fn new(max_age: Duration, capacity: usize) -> Self {
        Self {
            incomplete_frames: M::with_capacity(Self::INCOMPLETE_FRAME_RATIO * capacity + 1),
            complete_frames: VecDeque::with_capacity(capacity),
            tx_waker: None,
            rx_waker: None,
            is_closed: false,
            last_expiration: Instant::now(),
            max_age,
            capacity,
        }
    }

    fn expire_frames(&mut self) -> usize {
        let mut expired = 0;

        // Since the retaining operation is potentially expensive,
        // we do it actually only if there's a real chance that a frame is expired
        // or if the reassembler is closing (= everything is expired right away).
        if self.is_closed || self.last_expiration.elapsed() >= self.max_age {
            self.incomplete_frames.retain(|id, b| {
                if b.last_recv.elapsed() >= self.max_age || self.is_closed {
                    self.complete_frames.push_back(Err(SessionError::FrameDiscarded(*id)));
                    tracing::trace!(frame_id = id, "frame discarded");
                    expired += 1;
                    false
                } else {
                    true
                }
            });

            self.last_expiration = Instant::now();
        }

        expired
    }
}

impl Reassembler<FrameDashMap> {
    pub fn inspect(&self) -> FrameInspector {
        FrameInspector(self.incomplete_frames.clone())
    }
}

impl<M: FrameMap> futures::Sink<Segment> for Reassembler<M> {
    type Error = SessionError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tracing::trace!("Reassembler::poll_ready");
        if self.is_closed {
            return Poll::Ready(Err(SessionError::ReassemblerClosed));
        }

        // If we're at the capacity of incomplete frames,
        // check if we can expire more than the amount that's over the capacity.
        if self.incomplete_frames.len() >= self.capacity * Self::INCOMPLETE_FRAME_RATIO
            && self.expire_frames() <= self.incomplete_frames.len() - self.capacity
        {
            return Poll::Ready(Err(SessionError::TooManyIncompleteFrames));
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
                remaining = self.capacity - self.complete_frames.len(),
                "Reassembler::poll_ready ready",
            );
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Segment) -> Result<(), Self::Error> {
        tracing::trace!("Reassembler::start_send");
        if self.is_closed {
            return Err(SessionError::ReassemblerClosed);
        }

        let maybe_frame = match self.incomplete_frames.entry(item.frame_id) {
            FrameMapEntry::Occupied(mut e) => {
                let builder = e.get_builder_mut();
                match builder.add_segment(item) {
                    Ok(_) => {
                        if builder.is_complete() {
                            tracing::trace!(frame_id = e.frame_id(), "Reassembler::start_send frame is complete");
                            Some(e.finalize().try_into())
                        } else {
                            None
                        }
                    }
                    Err(error) => {
                        tracing::error!(frame_id = e.frame_id(), %error, "encountered invalid segment");
                        None
                    }
                }
            }
            FrameMapEntry::Vacant(e) => {
                let builder = FrameBuilder::from(item);
                if builder.is_complete() {
                    tracing::trace!(
                        frame_id = builder.frame_id(),
                        "Reassembler::start_send single segment frame is complete",
                    );
                    Some(builder.try_into())
                } else {
                    e.insert_builder(builder);
                    None
                }
            }
        };

        if let Some(frame) = maybe_frame {
            self.complete_frames.push_back(frame);
            tracing::trace!("Reassembler::start_send pushed new");
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

impl<M: FrameMap> futures::Stream for Reassembler<M> {
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
                    .inspect(|f| tracing::trace!(frame_id = f.frame_id, "Reassembler::poll_next ready"))
                    .inspect_err(|e| tracing::trace!(error = %e, "Reassembler::poll_next ready error")),
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
    use bitvec::prelude::*;
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

        let _ = jh.await?;
        Ok(())
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
                // Frame 2 had missing segment, therefore there should be an error
                assert!(matches!(actual[i], Err(SessionError::FrameDiscarded(2))));
            }
        }

        let _ = jh.await?;
        Ok(())
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

        // Since `forward` closed the sink, even the incomplete Frame 5 should be yielded as error
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
                // Frame 5 had a missing segment, therefore there should be an error
                assert!(matches!(actual[i], Err(SessionError::FrameDiscarded(5))));
            }
        }

        let _ = jh.await?;
        Ok(())
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
