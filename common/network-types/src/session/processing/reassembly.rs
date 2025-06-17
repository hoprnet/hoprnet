use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures_time::future::Timer;
use tracing::instrument;

use crate::session::{
    errors::SessionError,
    frames::{
        Frame, FrameBuilder, FrameDashMap, FrameHashMap, FrameId, FrameInspector, FrameMap, FrameMapEntry,
        FrameMapOccupiedEntry, FrameMapVacantEntry, Segment,
    },
};

#[must_use = "streams do nothing unless polled"]
#[pin_project::pin_project]
pub struct Reassembler<S, M> {
    #[pin]
    inner: S,
    #[pin]
    timer: futures_time::task::Sleep,
    incomplete_frames: M,
    expired_frames: Vec<FrameId>,
    max_age: Duration,
    capacity: usize,
    last_expiration: Instant,
}

impl<S: futures::Stream<Item = Segment>, M: FrameMap> Reassembler<S, M> {
    fn new(inner: S, incomplete_frames: M, max_age: Duration, capacity: usize) -> Self {
        Self {
            inner,
            timer: futures_time::task::sleep(max_age.into()),
            incomplete_frames,
            expired_frames: Vec::with_capacity(capacity),
            last_expiration: Instant::now(),
            max_age,
            capacity,
        }
    }
}

impl<S: futures::Stream<Item = Segment>, M: FrameMap> futures::Stream for Reassembler<S, M> {
    type Item = Result<Frame, SessionError>;

    #[instrument(name = "Reassembler::poll_next", level = "trace", skip(self, cx))]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            if let Some(frame_id) = this.expired_frames.pop() {
                tracing::trace!(frame_id, "emit discarded frame");
                return Poll::Ready(Some(Err(SessionError::FrameDiscarded(frame_id))));
            }

            // Poll the timer only if there are incomplete frames
            let timer_poll = if this.incomplete_frames.len() > 0 {
                this.timer.as_mut().poll(cx)
            } else {
                Poll::Pending
            };

            // Poll the inner stream only if there's space in the reassembler
            let inner_poll = if this.incomplete_frames.len() <= *this.capacity {
                this.inner.as_mut().poll_next(cx)
            } else {
                // This essentially forces the incomplete frames to be expired
                tracing::warn!("reassembler has reached its capacity");
                Poll::Pending
            };

            tracing::trace!("polling next");
            match (inner_poll, timer_poll) {
                (Poll::Ready(Some(item)), timer) => {
                    if timer.is_ready() {
                        this.timer.as_mut().reset_timer();
                    }

                    // Since the retaining operation is potentially expensive,
                    // we do it actually only if there's a real chance that a frame is expired
                    if this.last_expiration.elapsed() >= *this.max_age {
                        this.incomplete_frames.retain(|id, builder| {
                            if builder.last_recv.elapsed() >= *this.max_age && *id != item.frame_id {
                                this.expired_frames.push(*id);
                                false
                            } else {
                                true
                            }
                        });
                        *this.last_expiration = Instant::now();
                    }

                    match this.incomplete_frames.entry(item.frame_id) {
                        FrameMapEntry::Occupied(mut e) => {
                            let builder = e.get_builder_mut();
                            match builder.add_segment(item) {
                                Ok(_) => {
                                    if builder.is_complete() {
                                        tracing::trace!(frame_id = builder.frame_id(), "frame is complete");
                                        return Poll::Ready(Some(e.finalize().try_into()));
                                    }
                                }
                                Err(error) => {
                                    tracing::error!(%error, "encountered invalid segment");
                                }
                            }
                        }
                        FrameMapEntry::Vacant(e) => {
                            let builder = FrameBuilder::from(item);
                            if builder.is_complete() {
                                tracing::trace!(frame_id = builder.frame_id(), "segment frame is complete");
                                return Poll::Ready(Some(builder.try_into()));
                            } else {
                                e.insert_builder(builder);
                            }
                        }
                    };
                }
                (Poll::Ready(None), _) => {
                    // Inner stream closed, dump all incomplete frames
                    tracing::trace!("inner stream closed, dumping incomplete frames");
                    if this.incomplete_frames.len() > 0 {
                        this.incomplete_frames.retain(|id, _| {
                            this.expired_frames.push(*id);
                            false
                        });
                    } else {
                        tracing::trace!("done");
                        return Poll::Ready(None);
                    }
                }
                (Poll::Pending, Poll::Ready(_)) => {
                    // Check if some frames are expired
                    this.incomplete_frames.retain(|id, builder| {
                        if builder.last_recv.elapsed() >= *this.max_age {
                            this.expired_frames.push(*id);
                            false
                        } else {
                            true
                        }
                    });
                    *this.last_expiration = Instant::now();
                    this.timer.as_mut().reset_timer();
                }
                (Poll::Pending, Poll::Pending) => return Poll::Pending,
            }
        }
    }
}

pub trait ReassemblerExt: futures::Stream<Item = Segment> {
    fn reassembler(self, timeout: Duration, capacity: usize) -> Reassembler<Self, FrameHashMap>
    where
        Self: Sized,
    {
        Reassembler::new(
            self,
            FrameHashMap::with_capacity(FrameInspector::INCOMPLETE_FRAME_RATIO * capacity + 1),
            timeout,
            capacity,
        )
    }

    fn reassembler_with_inspector(
        self,
        timeout: Duration,
        capacity: usize,
        inspector: FrameInspector,
    ) -> Reassembler<Self, FrameDashMap>
    where
        Self: Sized,
    {
        Reassembler::new(self, inspector.0.clone(), timeout, capacity)
    }
}

impl<T: ?Sized> ReassemblerExt for T where T: futures::Stream<Item = Segment> {}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use anyhow::anyhow;
    use futures::{SinkExt, StreamExt, TryStreamExt, pin_mut};
    use futures_time::future::FutureExt;
    use hex_literal::hex;
    use rand::{SeedableRng, prelude::SliceRandom, rngs::StdRng};

    use super::*;
    use crate::session::processing::segment;

    const RNG_SEED: [u8; 32] = hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

    fn result_comparator(a: &Result<Frame, SessionError>, b: &Result<Frame, SessionError>) -> Ordering {
        match (a, b) {
            (Ok(a), Ok(b)) => a.frame_id.cmp(&b.frame_id),
            (Err(SessionError::FrameDiscarded(a)), Ok(b)) => a.cmp(&b.frame_id),
            (Ok(a), Err(SessionError::FrameDiscarded(b))) => a.frame_id.cmp(b),
            (Err(SessionError::FrameDiscarded(a)), Err(SessionError::FrameDiscarded(b))) => a.cmp(b),
            _ => panic!("unexpected result"),
        }
    }

    #[test_log::test(tokio::test)]
    pub async fn reassembler_should_reassemble_frames() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = futures::channel::mpsc::unbounded();
        let r_stream = r_stream.reassembler(Duration::from_secs(5), 1024);

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 22, f.frame_id).unwrap())
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let jh = hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

        let mut actual = r_stream
            .try_collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await??;

        assert_eq!(actual.len(), expected.len());

        actual.sort_by(|a, b| a.frame_id.cmp(&b.frame_id));
        assert_eq!(actual, expected);

        let _ = jh.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    pub async fn reassembler_should_discard_incomplete_frames_on_expiration() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = futures::channel::mpsc::unbounded();
        let r_stream = r_stream.reassembler(Duration::from_millis(45), 1024);

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 22, f.frame_id).unwrap())
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
                            futures::future::ok(s)
                                .delay(futures_time::time::Duration::from_millis(55))
                                .await
                        } else {
                            Ok(s)
                        }
                    }
                })
                .forward(r_sink),
        );

        let mut actual = r_stream
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        assert_eq!(actual.len(), expected.len());

        actual.sort_by(result_comparator);

        for i in 0..expected.len() {
            if i != 1 {
                assert!(matches!(&actual[i], Ok(f) if *f == expected[i]));
            } else {
                // Frame 2 had a missing segment, therefore there should be an error
                assert!(matches!(actual[i], Err(SessionError::FrameDiscarded(2))));
            }
        }

        let _ = jh.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    pub async fn reassembler_should_discard_incomplete_frames_on_close() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = futures::channel::mpsc::unbounded();
        let r_stream = r_stream.reassembler(Duration::from_millis(100), 1024);

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 22, f.frame_id).unwrap())
            .filter(|s| s.frame_id != 5 || s.seq_idx != 2)
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let jh = hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

        let mut actual = r_stream
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        // Since `forward` closed the sink, even the incomplete Frame 5 should be yielded as error
        assert_eq!(actual.len(), expected.len());

        actual.sort_by(result_comparator);

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

    #[test_log::test(tokio::test)]
    pub async fn reassembler_should_wait_and_discard_if_full() -> anyhow::Result<()> {
        let expected = (1u32..=5)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<30>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = futures::channel::mpsc::unbounded();
        let r_stream = r_stream.reassembler(Duration::from_millis(200), 3);

        pin_mut!(r_sink);
        pin_mut!(r_stream);

        // This creates 5 frames with 2 segments each
        let segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 20, f.frame_id).unwrap())
            .collect::<Vec<_>>();

        let to_send = [
            // Frame 1: Segment 2, Segment 1 missing
            segments[1].clone(),
            // Frame 2: Segment 1, Segment 2 missing
            segments[2].clone(),
            // Frame 3: Segment 2, Segment 1 missing
            segments[5].clone(),
        ];

        let start = Instant::now();

        // Reassembler now contains 3 incomplete frames
        r_sink.send_all(&mut futures::stream::iter(to_send).map(Ok)).await?;

        // It must not yield anything
        assert!(
            r_stream
                .next()
                .timeout(futures_time::time::Duration::from_millis(20))
                .await
                .is_err()
        );

        // Entire Frames 4 & 5
        r_sink
            .send_all(
                &mut futures::stream::iter([
                    segments[6].clone(),
                    segments[7].clone(),
                    segments[8].clone(),
                    segments[9].clone(),
                ])
                .map(Ok),
            )
            .await?;

        let mut reassembled = Vec::new();
        for _ in 0..5 {
            reassembled.push(r_stream.next().await.ok_or(anyhow!("missing frame"))?);
        }
        reassembled.sort_by(result_comparator);

        assert!(matches!(reassembled[0], Err(SessionError::FrameDiscarded(1))));
        assert!(matches!(reassembled[1], Err(SessionError::FrameDiscarded(2))));
        assert!(matches!(reassembled[2], Err(SessionError::FrameDiscarded(3))));
        assert!(matches!(&reassembled[3], Ok(f) if f == &expected[3].clone()));
        assert!(matches!(&reassembled[4], Ok(f) if f == &expected[4].clone()));

        r_sink.close().await?;
        assert_eq!(None, r_stream.try_next().await?);

        assert!(start.elapsed() >= Duration::from_millis(200));

        Ok(())
    }
}
