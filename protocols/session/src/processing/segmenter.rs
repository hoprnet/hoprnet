//! This module defines a [`Segmenter`] adaptor for [`futures::Sink`].
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::ready;
use tracing::instrument;

use crate::{
    errors::SessionError,
    frames::{FrameId, Segment, SeqIndicator, SeqNum},
    protocol::SessionMessage,
};

/// Segmenter is an adaptor to [`futures::Sink`] of [`Segment`] items
/// that turns it into [`futures::io::AsyncWrite`].
///
/// Bytes written to the Segmenter are buffered up and/or chopped into [`Segment`]
/// of at most `C` in payload size (the `data` member).
///
/// The data are grouped into [`Frames`](frames::Frame) of the size given by the `frame_size`
/// parameter, segments in each such group share the same [`FrameId`].
/// This acts as a natural buffering feature of a Segmenter.
///
/// Segmenter can optionally send a [terminating](Segment::terminating) when `poll_close`
/// is called. If an unflushed segment is in the buffer, it will be marked as terminating, otherwise
/// an empty terminating segment is emitted.
///
/// Segmenter is essentially inverse of [`Reassembler`](super::reassembly::Reassembler).
///
/// Use [`SegmenterExt`] to turn a `Segment` sink into an `AsyncWrite` object using the `Segmenter`.
#[must_use = "sinks do nothing unless polled"]
#[pin_project::pin_project]
pub struct Segmenter<const C: usize, S> {
    #[pin]
    tx: S,
    seg_buffer: Vec<u8>,
    ready_segments: Vec<Segment>,
    next_frame_id: FrameId,
    current_frame_len: usize,
    frame_size: usize,
    closed: bool,
    flush_each_segment: bool,
    send_terminating_segment: bool,
}

impl<const C: usize, S> Segmenter<C, S>
where
    S: futures::sink::Sink<Segment, Error = SessionError>,
{
    const PAYLOAD_CAPACITY: usize = C - SessionMessage::<C>::SEGMENT_OVERHEAD;

    /// Creates a new instance, wrapping the given `inner` Segment sink.
    ///
    /// The `frame_size` value will be clamped into the `[C, (C - SessionMessage::SEGMENT_OVERHEAD) * SeqIndicator::MAX
    /// + 1]` interval.
    pub fn new(inner: S, frame_size: usize, send_terminating_segment: bool, flush_each_segment: bool) -> Self {
        // We must clamp to at most SeqIndicator::MAX + 1 segments per frame.
        // This is true for Session protocol without partial acknowledgements.
        // When partial acknowledgements are enabled, the maximum frame size is less,
        // and the caller must take care of it.
        let frame_size = frame_size.clamp(
            C,
            (C - SessionMessage::<C>::SEGMENT_OVERHEAD) * (SeqIndicator::MAX + 1) as usize,
        );

        Self {
            seg_buffer: Vec::with_capacity(Self::PAYLOAD_CAPACITY),
            ready_segments: Vec::with_capacity(frame_size / C + 1),
            next_frame_id: 1,
            current_frame_len: 0,
            closed: false,
            tx: inner,
            frame_size,
            flush_each_segment,
            send_terminating_segment,
        }
    }

    #[instrument(name = "Segmenter::poll_flush_segments", level = "trace", skip(self, cx), fields(frame_id = self.next_frame_id, seq_len = self.ready_segments.len()), ret)]
    fn poll_flush_segments(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SessionError>> {
        let seq_len = self.ready_segments.len();
        debug_assert!(seq_len <= SeqIndicator::MAX as usize);

        let mut this = self.project();
        tracing::trace!("flushing segments");

        let all_segments = this.ready_segments.drain(..).collect::<Vec<_>>();
        for (i, mut seg) in all_segments.into_iter().enumerate() {
            seg.seq_idx = i as SeqNum;
            seg.seq_flags = SeqIndicator::new_with_flags(seq_len as SeqNum, seg.seq_flags.is_terminating());

            ready!(this.tx.as_mut().poll_ready(cx))?;

            tracing::trace!(seq_idx = seg.seq_idx, "segment out");
            this.tx.as_mut().start_send(seg)?;

            if *this.flush_each_segment {
                ready!(this.tx.as_mut().poll_flush(cx))?;
                tracing::trace!(seg_idx = i, "segment flushed out");
            }
        }

        if !*this.flush_each_segment {
            ready!(this.tx.as_mut().poll_flush(cx))?;
            tracing::trace!("frame flushed out");
        }

        // Increase Frame ID only if there were segments to send
        if seq_len > 0 {
            *this.next_frame_id = this.next_frame_id.wrapping_add(1);
            *this.current_frame_len = 0;
        }

        Poll::Ready(Ok(()))
    }

    #[instrument(name = "Segmenter::complete_segment", level = "trace", skip(self), fields(frame_id = self.next_frame_id, seq_len = self.ready_segments.len()))]
    fn complete_segment(self: Pin<&mut Self>) {
        let this = self.project();
        let new_segment = Segment {
            frame_id: *this.next_frame_id,
            seq_idx: 0,
            seq_flags: SeqIndicator::default(),
            data: this.seg_buffer.clone().into_boxed_slice(),
        };
        this.seg_buffer.clear();

        let seg_len = new_segment.data.len();
        *this.current_frame_len += seg_len;

        tracing::trace!(
            bytes_added = seg_len,
            remaining_in_frame = *this.frame_size - *this.current_frame_len,
            "completed segment"
        );

        this.ready_segments.push(new_segment);
    }

    fn create_terminating_segment(self: Pin<&mut Self>) {
        let this = self.project();
        if let Some(last_segment) = this.ready_segments.last_mut() {
            last_segment.seq_flags = last_segment.seq_flags.with_terminating_bit(true);
        } else {
            this.ready_segments.push(Segment::terminating(*this.next_frame_id));
        }
    }
}

impl<const C: usize, S> futures::io::AsyncWrite for Segmenter<C, S>
where
    S: futures::sink::Sink<Segment, Error = SessionError>,
{
    #[instrument(name = "Segmenter::poll_write", level = "trace", skip(self, cx, buf), fields(len = buf.len()), ret)]
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        tracing::trace!("polling write");
        if self.closed {
            tracing::trace!("error closed");
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        if buf.is_empty() {
            tracing::trace!("ready empty buffer");
            return Poll::Ready(Ok(0));
        }

        if self.next_frame_id == 0 {
            tracing::warn!("end of frame sequence reached");
            return Poll::Ready(Err(std::io::ErrorKind::QuotaExceeded.into()));
        }

        // if self.current_frame_len == self.frame_size {
        // tracing::trace!("frame is complete before write");
        // self.as_mut().complete_segment();
        // ready!(self.as_mut().poll_flush_segments(cx)).map_err(std::io::Error::other)?;
        // }

        // Write only as much as there is space in the segment or the frame
        let new_len_in_buffer = buf
            .len()
            .min(Self::PAYLOAD_CAPACITY - self.seg_buffer.len())
            .min(self.frame_size - self.current_frame_len);
        debug_assert!(new_len_in_buffer > 0);

        self.as_mut()
            .project()
            .seg_buffer
            .extend_from_slice(&buf[..new_len_in_buffer]);

        tracing::trace!(
            buf_len = self.seg_buffer.len(),
            len = new_len_in_buffer,
            remaining = Self::PAYLOAD_CAPACITY - self.seg_buffer.len(),
            "add segment data @ cap {}",
            Self::PAYLOAD_CAPACITY
        );

        // If the chunk finishes the frame, flush it
        if self.current_frame_len + new_len_in_buffer == self.frame_size {
            tracing::trace!(last_write = new_len_in_buffer, "frame is complete");
            self.as_mut().complete_segment();
            ready!(self.as_mut().poll_flush_segments(cx)).map_err(std::io::Error::other)?;
        } else if self.seg_buffer.len() == Self::PAYLOAD_CAPACITY {
            tracing::trace!("segment is complete");
            self.as_mut().complete_segment();

            if self.current_frame_len == self.frame_size {
                tracing::trace!("frame is complete by the completed segment");
                ready!(self.as_mut().poll_flush_segments(cx)).map_err(std::io::Error::other)?;
            }
        }

        Poll::Ready(Ok(new_len_in_buffer))
    }

    #[instrument(name = "Segmenter::poll_flush", level = "trace", skip(self, cx), ret)]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tracing::trace!("flush");

        if self.closed {
            tracing::trace!("error closed");
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        if self.next_frame_id == 0 {
            tracing::warn!("end of frame sequence reached");
            return Poll::Ready(Err(std::io::ErrorKind::QuotaExceeded.into()));
        }

        // Make sure no segment part is buffered
        if !self.seg_buffer.is_empty() {
            tracing::trace!(
                frame_id = self.next_frame_id,
                len = self.seg_buffer.len(),
                "partial frame"
            );
            self.as_mut().complete_segment();
        }

        self.as_mut().poll_flush_segments(cx).map_err(std::io::Error::other)
    }

    #[instrument(name = "Segmenter::poll_close", level = "trace", skip(self, cx), ret)]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tracing::trace!("close");

        if self.closed {
            tracing::trace!("error closed");
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        if self.next_frame_id == 0 {
            let this = self.project();
            *this.closed = true;
            return Poll::Ready(Ok(()));
        }

        if !self.seg_buffer.is_empty() {
            tracing::trace!(
                frame_id = self.next_frame_id,
                len = self.seg_buffer.len(),
                "partial frame"
            );
            self.as_mut().complete_segment();
        }

        if self.send_terminating_segment {
            self.as_mut().create_terminating_segment();
        }

        ready!(self.as_mut().poll_flush_segments(cx).map_err(std::io::Error::other))?;

        let this = self.project();
        ready!(this.tx.poll_close(cx).map_err(std::io::Error::other))?;

        *this.closed = true;
        Poll::Ready(Ok(()))
    }
}

/// Sink extension methods for segmenting binary data into a sink.
pub trait SegmenterExt: futures::Sink<Segment, Error = SessionError> {
    /// Attaches a [`Segmenter`] to the underlying sink.
    fn segmenter<const C: usize>(self, frame_size: usize) -> Segmenter<C, Self>
    where
        Self: Sized,
    {
        Segmenter::new(self, frame_size, false, false)
    }

    /// Attaches a [`Segmenter`] to the underlying sink.
    /// The `Segmenter` also sends a [terminating](Segment::terminating) when closed.
    fn segmenter_with_terminating_segment<const C: usize>(self, frame_size: usize) -> Segmenter<C, Self>
    where
        Self: Sized,
    {
        Segmenter::new(self, frame_size, true, false)
    }
}

impl<T: ?Sized> SegmenterExt for T where T: futures::Sink<Segment, Error = SessionError> {}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use futures::{AsyncWriteExt, SinkExt, Stream, StreamExt, pin_mut};
    use futures_time::future::FutureExt;

    use super::*;
    use crate::utils::segment;

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
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            let start_frame_id = start_frame_id as usize;
            let frame_id = i / SEGMENTS_PER_FRAME + start_frame_id;
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
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

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
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        let mut all_data = Vec::new();
        for _ in 0..num_frames {
            let mut offset = 0;
            let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

            while offset < data.len() {
                let written = writer.write(&data[offset..]).await?;
                assert!(written <= SMTU);
                offset += written;
            }
            all_data.extend(data);
        }

        pin_mut!(segments);
        assert_frame_segments(1, num_frames, &mut segments, &all_data).await?;

        writer.close().await?;

        assert_eq!(None, segments.next().await);
        Ok(())
    }

    #[tokio::test]
    async fn segmenter_full_frame_segmentation_must_be_consistent() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

        writer.write_all(&data).await?;
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
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        writer.send_terminating_segment = true;

        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

        writer.write_all(&data).await?;
        writer.close().await?;

        // Segmenter already takes into account the SessionMessage overhead
        let mut expected = segment(&data, SMTU, 1)?;
        expected.push(Segment::terminating(2));
        let actual = segments.collect::<Vec<_>>().await;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn segmenter_partial_frame_segmentation_must_also_include_terminating_segment() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        writer.send_terminating_segment = true;

        let data = hopr_crypto_random::random_bytes::<{ FRAME_SIZE - 1 }>();

        writer.write_all(&data).await?;
        writer.close().await?;

        // Segmenter already takes into account the SessionMessage overhead
        let mut expected = segment(&data, SMTU, 1)?;
        expected.last_mut().unwrap().seq_flags = expected.last_mut().unwrap().seq_flags.with_terminating_bit(true);
        let actual = segments.collect::<Vec<_>>().await;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn segmenter_should_segment_complete_frame_with_misaligned_mtu() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        let mut offset = 0;
        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

        while offset < data.len() {
            let written = writer.write(&data[offset..]).await?;
            let expected_written = SMTU.min(data.len() - offset);
            assert_eq!(expected_written, written);
            offset += written;
        }

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

    #[tokio::test]
    async fn segmenter_should_segment_multiple_complete_frames_and_incomplete_frame_on_close() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        let data = hopr_crypto_random::random_bytes::<{ FRAME_SIZE + 4 }>();

        writer.write_all(&data).await?;

        pin_mut!(segments);

        assert_frame_segments(1, 1, &mut segments, &data).await?;

        segments
            .next()
            .timeout(futures_time::time::Duration::from_millis(10))
            .await
            .expect_err("should time out");

        writer.close().await?;

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(2, seg.frame_id);
        assert_eq!(0, seg.seq_idx);
        assert_eq!(1, seg.seq_flags.seq_len());
        assert_eq!(4, seg.data.len());
        assert_eq!(&data[FRAME_SIZE..], seg.data.as_ref());

        assert_eq!(None, segments.next().await);

        Ok(())
    }

    #[tokio::test]
    async fn segmenter_should_segment_multiple_complete_frames_and_incomplete_frame_on_flush() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

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

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(2, seg.frame_id);
        assert_eq!(0, seg.seq_idx);
        assert_eq!(1, seg.seq_flags.seq_len());
        assert_eq!(4, seg.data.len());
        assert_eq!(&data[FRAME_SIZE..], seg.data.as_ref());

        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();
        writer.write_all(&data).await?;

        assert_frame_segments(3, 1, &mut segments, &data).await?;

        Ok(())
    }
}
