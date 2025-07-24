use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
};

use tracing::instrument;

use crate::{
    frames::{FrameId, Segment},
    protocol::SessionMessage,
    utils::segment_into,
};

#[pin_project::pin_project]
pub struct Segmenter<const C: usize, S> {
    #[pin]
    inner: S,
    state: State,
    frame: Vec<u8>,
    ready_segments: VecDeque<Segment>,
    frame_size: usize,
    frame_id: FrameId,
    is_closed: bool,
    send_terminating_segment: bool,
}

enum State {
    BufferingFrame,
    WritingFrame,
}

impl<const C: usize, S> Segmenter<C, S>
where
    S: futures::Sink<Segment>,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    fn new(inner: S, frame_size: usize, send_terminating_segment: bool) -> Self {
        Self {
            inner,
            state: State::BufferingFrame,
            frame: Vec::with_capacity(frame_size),
            ready_segments: VecDeque::with_capacity(frame_size.div_ceil(C - SessionMessage::<C>::SEGMENT_OVERHEAD)),
            frame_size,
            frame_id: 1,
            is_closed: false,
            send_terminating_segment,
        }
    }
}

impl<const C: usize, S> futures::io::AsyncWrite for Segmenter<C, S>
where
    S: futures::Sink<Segment>,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    #[instrument(name = "Segmenter::poll_write", level = "trace", skip(self, cx, buf), fields(frame_id = self.frame_id, buf_len = buf.len(), frame_size = self.frame.len(), ready_segments = self.ready_segments.len()), ret)]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        if self.is_closed {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "segmenter closed",
            )));
        }

        let mut this = self.project();
        loop {
            match this.state {
                State::BufferingFrame => {
                    // If there's space in the frame, keep writing to it
                    if *this.frame_size > this.frame.len() {
                        let to_write = buf.len().min(*this.frame_size - this.frame.len());
                        this.frame.extend_from_slice(&buf[..to_write]);

                        return Poll::Ready(Ok(to_write));
                    } else {
                        // No more space in the frame buffer, we need to segment it
                        // and write segments to the downstream
                        if let Err(error) = segment_into(
                            this.frame.as_slice(),
                            C - SessionMessage::<C>::SEGMENT_OVERHEAD,
                            *this.frame_id,
                            this.ready_segments,
                        ) {
                            return Poll::Ready(Err(std::io::Error::other(error)));
                        }

                        tracing::trace!(num_segments = this.ready_segments.len(), "frame ready");

                        this.frame.clear();
                        *this.frame_id += 1;
                        *this.state = State::WritingFrame;
                    }
                }
                State::WritingFrame => {
                    if !this.ready_segments.is_empty() {
                        // Keep writing segments to downstream
                        futures::ready!(this.inner.as_mut().poll_ready(cx).map_err(std::io::Error::other))?;

                        let segment = this.ready_segments.pop_front().unwrap();
                        tracing::trace!(seg_id = %segment.id(), "segment goes out");
                        this.inner.as_mut().start_send(segment).map_err(std::io::Error::other)?;
                    } else {
                        // Once we're done, we can buffer another frame
                        *this.state = State::BufferingFrame;
                        tracing::trace!("all segments out");
                    }
                }
            }
        }
    }

    #[instrument(name = "Segmenter::poll_flush", level = "trace", skip(self, cx), fields(frame_id = self.frame_id, frame_size = self.frame.len(), ready_segments = self.ready_segments.len()), ret)]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        if self.is_closed {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "segmenter closed",
            )));
        }

        let mut this = self.project();
        loop {
            // If there's any data in the unfinished frame, segment it
            if !this.frame.is_empty() {
                // Flush the downstream sink first
                if let Err(error) = futures::ready!(this.inner.as_mut().poll_flush(cx).map_err(std::io::Error::other)) {
                    return Poll::Ready(Err(error));
                }

                // Segment whatever data is in the frame
                // At this point ready_segments must be empty,
                // because poll_write always makes sure it is before returning Ready
                if let Err(error) = segment_into(
                    this.frame.as_slice(),
                    C - SessionMessage::<C>::SEGMENT_OVERHEAD,
                    *this.frame_id,
                    this.ready_segments,
                ) {
                    return Poll::Ready(Err(std::io::Error::other(error)));
                }
                tracing::trace!(num_segments = this.ready_segments.len(), "flushed frame ready");

                this.frame.clear();
                *this.frame_id += 1;
            } else if !this.ready_segments.is_empty() {
                futures::ready!(this.inner.as_mut().poll_ready(cx).map_err(std::io::Error::other))?;

                let segment = this.ready_segments.pop_front().unwrap();
                tracing::trace!(seg_id = %segment.id(), "segment flushing out");

                if let Err(error) = this.inner.as_mut().start_send(segment) {
                    return Poll::Ready(Err(std::io::Error::other(error)));
                }
            } else {
                // Both buffers are empty, so only flush the downstream
                tracing::trace!("all segments flushed out");
                return this.inner.as_mut().poll_flush(cx).map_err(std::io::Error::other);
            }
        }
    }

    #[instrument(name = "Segmenter::poll_close", level = "trace", skip(self, cx), fields(frame_id = self.frame_id) , ret)]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut this = self.project();

        if *this.send_terminating_segment && !*this.is_closed {
            futures::ready!(this.inner.as_mut().poll_ready(cx).map_err(std::io::Error::other))?;
            let dummy = Segment::terminating(*this.frame_id);
            this.inner.as_mut().start_send(dummy).map_err(std::io::Error::other)?;
            tracing::trace!("sent terminating segment");
        }

        *this.is_closed = true;
        this.inner.as_mut().poll_close(cx).map_err(std::io::Error::other)
    }
}

/// Sink extension methods for segmenting binary data into a sink.
pub trait SegmenterExt: futures::Sink<Segment> {
    /// Attaches a [`crate::processing::segmenter::Segmenter`] to the underlying sink.
    fn segmenter<const C: usize>(self, frame_size: usize) -> Segmenter<C, Self>
    where
        Self: Sized,
        Self::Error: std::error::Error + Send + Sync + 'static,
    {
        Segmenter::new(self, frame_size, false)
    }

    /// Attaches a [`crate::processing::segmenter::Segmenter`] to the underlying sink.
    /// The `Segmenter` also sends a [terminating](Segment::terminating) when closed.
    fn segmenter_with_terminating_segment<const C: usize>(self, frame_size: usize) -> Segmenter<C, Self>
    where
        Self: Sized,
        Self::Error: std::error::Error + Send + Sync + 'static,
    {
        Segmenter::new(self, frame_size, true)
    }
}

impl<T: ?Sized> SegmenterExt for T where T: futures::Sink<Segment> {}

#[cfg(test)]
mod tests {
    use anyhow::{Context, anyhow};
    use futures::{AsyncWriteExt, Stream, StreamExt, pin_mut};
    use futures_time::future::FutureExt;

    use super::*;
    use crate::{frames::SeqNum, utils::segment};

    const MTU: usize = 1000;
    const SMTU: usize = MTU - SessionMessage::<MTU>::SEGMENT_OVERHEAD;
    const FRAME_SIZE: usize = 1500;

    const SEGMENTS_PER_FRAME: usize = FRAME_SIZE / MTU + 1;

    async fn assert_frame_segments(
        start_frame_id: FrameId,
        num_frames: usize,
        segments: &mut (impl Stream<Item = Segment> + Unpin),
        data: &[u8],
    ) -> anyhow::Result<()> {
        for i in 0..num_frames * SEGMENTS_PER_FRAME {
            let start_frame_id = start_frame_id as usize;
            let frame_id = i / SEGMENTS_PER_FRAME + start_frame_id;
            tracing::debug!("testing frame id {frame_id} {}", (i % SEGMENTS_PER_FRAME) as SeqNum);

            let seg = segments
                .next()
                .timeout(futures_time::time::Duration::from_millis(500))
                .await
                .context(format!("assert_frame_segments {i}"))?
                .ok_or(anyhow!("no more segments"))?;

            assert_eq!(frame_id as FrameId, seg.frame_id);
            assert_eq!((i % SEGMENTS_PER_FRAME) as SeqNum, seg.seq_idx);
            assert_eq!((FRAME_SIZE / MTU + 1) as SeqNum, seg.seq_flags.seq_len());
            if i % SEGMENTS_PER_FRAME == 0 {
                assert_eq!(SMTU, seg.data.len());
                assert_eq!(
                    &data[(frame_id - start_frame_id) * FRAME_SIZE + i % SEGMENTS_PER_FRAME * SMTU
                        ..(frame_id - start_frame_id) * FRAME_SIZE + i % SEGMENTS_PER_FRAME * SMTU + SMTU],
                    seg.data.as_ref()
                );
            } else {
                assert_eq!(FRAME_SIZE % SMTU, seg.data.len());
                assert_eq!(
                    &data[(frame_id - start_frame_id) * FRAME_SIZE + i % SEGMENTS_PER_FRAME * SMTU
                        ..(frame_id - start_frame_id) * FRAME_SIZE + i % SEGMENTS_PER_FRAME * SMTU + FRAME_SIZE % SMTU],
                    seg.data.as_ref()
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn segmenter_should_not_segment_small_data_unless_flushed() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx.segmenter::<MTU>(FRAME_SIZE);

        writer.write_all(b"test").await?;

        pin_mut!(segments);
        segments
            .next()
            .timeout(futures_time::time::Duration::from_millis(10))
            .await
            .expect_err("should time out");

        writer.flush().await?;

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(1, seg.frame_id);
        assert_eq!(1, seg.seq_flags.seq_len());
        assert_eq!(0, seg.seq_idx);
        assert_eq!(b"test", seg.data.as_ref());

        Ok(())
    }

    #[parameterized::parameterized(num_frames = { 1, 3, 5, 11 })]
    #[parameterized_macro(tokio::test)]
    async fn segmenter_should_segment_complete_frames(num_frames: usize) -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx.segmenter::<MTU>(FRAME_SIZE);

        let mut all_data = Vec::new();
        for _ in 0..num_frames {
            let mut offset = 0;
            let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

            while offset < data.len() {
                let written = writer.write(&data[offset..]).await?;
                offset += written;
            }
            all_data.extend(data);
        }

        writer.flush().await?;

        pin_mut!(segments);
        assert_frame_segments(1, num_frames, &mut segments, &all_data).await?;

        writer.close().await?;

        assert_eq!(None, segments.next().await);
        Ok(())
    }

    #[tokio::test]
    async fn segmenter_full_frame_segmentation_must_be_consistent_with_segment_function() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx.segmenter::<MTU>(FRAME_SIZE);

        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

        writer.write_all(&data).await?;
        writer.flush().await?;
        writer.close().await?;

        // Segmenter already takes into account the SessionMessage overhead
        let expected = segment(&data, SMTU, 1)?;
        let actual = segments.collect::<Vec<_>>().await;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn segmenter_full_frame_segmentation_must_also_include_terminating_segment() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx.segmenter_with_terminating_segment::<MTU>(FRAME_SIZE);

        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

        writer.write_all(&data).await?;
        writer.flush().await?;
        writer.close().await?;

        // Segmenter already takes into account the SessionMessage overhead
        let mut expected = segment(&data, SMTU, 1)?;
        expected.push(Segment::terminating(2));
        let actual = segments.collect::<Vec<_>>().await;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn segmenter_should_segment_complete_frame_with_misaligned_mtu() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx.segmenter::<MTU>(FRAME_SIZE);

        let mut offset = 0;
        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

        while offset < data.len() {
            let written = writer.write(&data[offset..]).await?;
            offset += written;
        }

        writer.flush().await?;
        writer.close().await?;

        pin_mut!(segments);

        for i in 0..(FRAME_SIZE / MTU) {
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            assert_eq!(1, seg.frame_id);
            assert_eq!(i as SeqNum, seg.seq_idx);
            assert_eq!(((FRAME_SIZE / SMTU) + 1) as SeqNum, seg.seq_flags.seq_len());
            assert_eq!(SMTU, seg.data.len());
            assert_eq!(&data[i * SMTU..i * SMTU + SMTU], seg.data.as_ref());
        }

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(1, seg.frame_id);
        assert_eq!((FRAME_SIZE / SMTU) as SeqNum, seg.seq_idx);
        assert_eq!(((FRAME_SIZE / SMTU) + 1) as SeqNum, seg.seq_flags.seq_len());
        assert_eq!(FRAME_SIZE % SMTU, seg.data.len());
        assert_eq!(&data[FRAME_SIZE - FRAME_SIZE % SMTU..], seg.data.as_ref());

        assert_eq!(None, segments.next().await);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn segmenter_should_segment_multiple_complete_frames_and_incomplete_frame_on_flush() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx.segmenter::<MTU>(FRAME_SIZE);

        let data = hopr_crypto_random::random_bytes::<{ FRAME_SIZE + 4 }>();
        writer.write_all(&data).await?;

        pin_mut!(segments);

        assert_frame_segments(1, 1, &mut segments, &data).await?;

        segments
            .next()
            .timeout(futures_time::time::Duration::from_millis(10))
            .await
            .expect_err("should time out");

        writer.flush().await?;

        let seg = segments
            .next()
            .timeout(futures_time::time::Duration::from_millis(500))
            .await?
            .ok_or(anyhow!("no more segments"))?;
        assert_eq!(2, seg.frame_id);
        assert_eq!(0, seg.seq_idx);
        assert_eq!(1, seg.seq_flags.seq_len());
        assert_eq!(4, seg.data.len());
        assert_eq!(&data[FRAME_SIZE..], seg.data.as_ref());

        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();
        writer.write_all(&data).await?;
        writer.flush().await?;

        assert_frame_segments(3, 1, &mut segments, &data).await?;

        Ok(())
    }
}
