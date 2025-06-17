use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::Stream;
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

impl<S: Stream<Item = Segment>, M: FrameMap> Reassembler<S, M> {
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

impl<S: Stream<Item = Segment>, M: FrameMap> futures::Stream for Reassembler<S, M> {
    type Item = Result<Frame, SessionError>;

    #[instrument(name = "Reassembler::poll_next", level = "trace", skip(self, cx))]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            if let Some(frame_id) = this.expired_frames.pop() {
                tracing::trace!(frame_id, "emit discarded frame");
                return Poll::Ready(Some(Err(SessionError::FrameDiscarded(frame_id))));
            }

            tracing::trace!("polling next");
            match (this.inner.as_mut().poll_next(cx), this.timer.as_mut().poll(cx)) {
                (Poll::Ready(Some(item)), timer) => {
                    if timer.is_ready() {
                        this.timer.as_mut().reset_timer();
                        //*this.timer = futures_time::task::sleep((*this.max_age).into());
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
                }
                (Poll::Pending, Poll::Pending) => return Poll::Pending,
            }
        }
    }
}

pub trait ReassemblerExt: Stream<Item = Segment> {
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

impl<T: ?Sized> ReassemblerExt for T where T: Stream<Item = Segment> {}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt, TryStreamExt, pin_mut};
    use futures_time::future::FutureExt;
    use hex_literal::hex;
    use rand::{SeedableRng, prelude::SliceRandom, rngs::StdRng};

    use super::*;
    use crate::session::processing::segment;

    const RNG_SEED: [u8; 32] = hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

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

    #[test_log::test(tokio::test)]
    pub async fn reassembler_should_wait_if_full() -> anyhow::Result<()> {
        let expected = (1u32..=5)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<30>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, r_stream) = futures::channel::mpsc::unbounded();
        let r_stream = r_stream.reassembler(Duration::from_secs(5), 1024);

        pin_mut!(r_sink);
        pin_mut!(r_stream);

        let segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 22, f.frame_id).unwrap())
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
        assert!(
            r_sink
                .send(segments[7].clone())
                .timeout(futures_time::time::Duration::from_millis(50))
                .await
                .is_err()
        );

        // Yield Frame 1
        assert_eq!(Some(expected[0].clone()), r_stream.try_next().await?);

        // This inserts segment 7 (from SplitSink's slot) and segment 6, thus completing Frame 4
        r_sink.send(segments[6].clone()).await?;

        // Cannot insert more segments until some frames are yielded
        assert!(
            r_sink
                .send(segments[8].clone())
                .timeout(futures_time::time::Duration::from_millis(50))
                .await
                .is_err()
        );

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
}
