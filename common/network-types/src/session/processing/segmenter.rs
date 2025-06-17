use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::ready;
use tracing::instrument;

use crate::session::{
    errors::SessionError,
    frames::{FrameId, Segment, SeqNum},
    protocol::SessionMessage,
};

/// `C` is MTU size.
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
}

impl<const C: usize, S> Segmenter<C, S>
where
    S: futures::sink::Sink<Segment, Error = SessionError>,
{
    const PAYLOAD_CAPACITY: usize = C - SessionMessage::<C>::SEGMENT_OVERHEAD;

    /// Creates a new instance, wrapping the given `inner` Segment sink.
    ///
    /// The `frame_size` value will be clamped into the `[C, C * SessionMessage::<C>::MAX_SEGMENTS_PER_FRAME]` interval.
    pub fn new(inner: S, frame_size: usize) -> Self {
        let frame_size = frame_size.clamp(C, C * SessionMessage::<C>::MAX_SEGMENTS_PER_FRAME);

        Self {
            seg_buffer: Vec::with_capacity(Self::PAYLOAD_CAPACITY),
            ready_segments: Vec::with_capacity(frame_size / C + 1),
            next_frame_id: 1,
            current_frame_len: 0,
            closed: false,
            tx: inner,
            frame_size,
            flush_each_segment: false,
        }
    }

    #[instrument(name = "Segmenter::poll_flush_segments", level = "trace", skip(self, cx), fields(frame_id = self.next_frame_id, seq_len = self.ready_segments.len()))]
    fn poll_flush_segments(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SessionError>> {
        let seq_len = self.ready_segments.len();
        let mut this = self.project();
        tracing::trace!("flushing segments");

        let all_segments = this.ready_segments.drain(..).collect::<Vec<_>>();
        for (i, mut seg) in all_segments.into_iter().enumerate() {
            seg.seq_idx = i as SeqNum;
            seg.seq_len = seq_len as SeqNum;

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

        *this.next_frame_id += 1;
        *this.current_frame_len = 0;

        Poll::Ready(Ok(()))
    }

    #[instrument(name = "Segmenter::complete_segment", level = "trace", skip(self), fields(frame_id = self.next_frame_id, seq_len = self.ready_segments.len()))]
    fn complete_segment(self: Pin<&mut Self>) {
        let this = self.project();
        let new_segment = Segment {
            frame_id: *this.next_frame_id,
            seq_idx: 0,
            seq_len: 0,
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
}

impl<const C: usize, S> futures::io::AsyncWrite for Segmenter<C, S>
where
    S: futures::sink::Sink<Segment, Error = SessionError>,
{
    #[instrument(name = "Segmenter::poll_write", level = "trace", skip(self, cx, buf), fields(len = buf.len()))]
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

        // Write only as much as there is space in the segment or the frame
        let new_len_in_buffer = buf
            .len()
            .min(Self::PAYLOAD_CAPACITY - self.seg_buffer.len())
            .min(self.frame_size - self.current_frame_len);

        if new_len_in_buffer > 0 {
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
        }

        // If the chunk finishes the frame, flush it
        if self.current_frame_len + new_len_in_buffer == self.frame_size {
            tracing::trace!("frame is complete");
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

        tracing::trace!(len = new_len_in_buffer, "ready");
        Poll::Ready(Ok(new_len_in_buffer))
    }

    #[instrument(name = "Segmenter::poll_flush", level = "trace", skip(self, cx))]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tracing::trace!("flush");

        if self.closed {
            tracing::trace!("error closed");
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

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

    #[instrument(name = "Segmenter::poll_close", level = "trace", skip(self, cx))]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tracing::trace!("close");

        if self.closed {
            tracing::trace!("error closed");
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        if !self.seg_buffer.is_empty() {
            tracing::trace!(
                frame_id = self.next_frame_id,
                len = self.seg_buffer.len(),
                "partial frame"
            );
            self.as_mut().complete_segment();
        }

        ready!(self.as_mut().poll_flush_segments(cx).map_err(std::io::Error::other))?;

        let this = self.project();
        ready!(this.tx.poll_close(cx).map_err(std::io::Error::other))?;

        *this.closed = true;
        Poll::Ready(Ok(()))
    }
}

pub trait SegmenterExt: futures::Sink<Segment, Error = SessionError> {
    fn segmenter<const C: usize>(self, frame_size: usize) -> Segmenter<C, Self>
    where
        Self: Sized,
    {
        Segmenter::new(self, frame_size)
    }
}

impl<T: ?Sized> SegmenterExt for T where T: futures::Sink<Segment, Error = SessionError> {}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use futures::{AsyncWriteExt, SinkExt, StreamExt, pin_mut};
    use futures_time::future::FutureExt;

    use super::*;
    use crate::session::processing::segment;

    const MTU: usize = 1000;
    const SMTU: usize = MTU - SessionMessage::<MTU>::SEGMENT_OVERHEAD;
    const FRAME_SIZE: usize = 1500;

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
        assert_eq!(1, seg.seq_len);
        assert_eq!(0, seg.seq_idx);
        assert_eq!(b"test", seg.data.as_ref());

        Ok(())
    }

    #[tokio::test]
    async fn segmenter_should_segment_complete_frame() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        let mut offset = 0;
        let data = hopr_crypto_random::random_bytes::<FRAME_SIZE>();

        while offset < data.len() {
            let written = writer.write(&data[offset..]).await?;
            assert_eq!(written, SMTU);
            offset += SMTU;
        }

        pin_mut!(segments);

        for i in 0..FRAME_SIZE / MTU {
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            assert_eq!(1, seg.frame_id);
            assert_eq!(i as SeqNum, seg.seq_idx);
            assert_eq!((FRAME_SIZE / MTU) as SeqNum, seg.seq_len);
            assert_eq!(SMTU, seg.data.len());
            assert_eq!(&data[i * SMTU..i * SMTU + SMTU], seg.data.as_ref());
        }

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
            assert_eq!(((FRAME_SIZE / SMTU) + 1) as SeqNum, seg.seq_len);
            assert_eq!(SMTU, seg.data.len());
            assert_eq!(&data[i * SMTU..i * SMTU + SMTU], seg.data.as_ref());
        }

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(1, seg.frame_id);
        assert_eq!((FRAME_SIZE / SMTU) as SeqNum, seg.seq_idx);
        assert_eq!(((FRAME_SIZE / SMTU) + 1) as SeqNum, seg.seq_len);
        assert_eq!(FRAME_SIZE % SMTU, seg.data.len());
        assert_eq!(&data[FRAME_SIZE - FRAME_SIZE % SMTU..], seg.data.as_ref());

        assert_eq!(None, segments.next().await);
        Ok(())
    }

    #[tokio::test]
    async fn segmenter_should_segment_multiple_complete_frames() -> anyhow::Result<()> {
        const NUM_FRAMES: usize = 3;

        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);
        let data = hopr_crypto_random::random_bytes::<{ NUM_FRAMES * FRAME_SIZE }>();

        writer.write_all(&data).await?;

        pin_mut!(segments);

        let count_segments = (NUM_FRAMES * FRAME_SIZE) / MTU;

        for i in 0..count_segments {
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            assert_eq!((i / count_segments + 1) as FrameId, seg.frame_id);
            assert_eq!((i % count_segments) as SeqNum, seg.seq_idx);
            assert_eq!((FRAME_SIZE / MTU) as SeqNum, seg.seq_len);
            assert_eq!(SMTU, seg.data.len());
            assert_eq!(&data[i * SMTU..i * SMTU + SMTU], seg.data.as_ref());
        }

        writer.close().await?;

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

        for i in 0..9 {
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            assert_eq!((i / 3 + 1) as FrameId, seg.frame_id);
            assert_eq!((i % 3) as SeqNum, seg.seq_idx);
            assert_eq!(3, seg.seq_len);
            assert_eq!(500, seg.data.len());
            assert_eq!(&data[i * 500..i * 500 + 500], seg.data.as_ref());
        }

        segments
            .next()
            .timeout(futures_time::time::Duration::from_millis(10))
            .await
            .expect_err("should time out");

        writer.close().await?;

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(4, seg.frame_id);
        assert_eq!(0, seg.seq_idx);
        assert_eq!(1, seg.seq_len);
        assert_eq!(4, seg.data.len());
        assert_eq!(&data[4500..], seg.data.as_ref());

        assert_eq!(None, segments.next().await);

        Ok(())
    }

    #[tokio::test]
    async fn segmenter_should_segment_multiple_complete_frames_and_incomplete_frame_on_flush() -> anyhow::Result<()> {
        let (segments_tx, segments) = futures::channel::mpsc::unbounded();
        let mut writer = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        let data = hopr_crypto_random::random_bytes::<4504>();

        writer.write_all(&data).await?;

        pin_mut!(segments);

        for i in 0..9 {
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            assert_eq!((i / 3 + 1) as FrameId, seg.frame_id);
            assert_eq!((i % 3) as SeqNum, seg.seq_idx);
            assert_eq!(3, seg.seq_len);
            assert_eq!(500, seg.data.len());
            assert_eq!(&data[i * 500..i * 500 + 500], seg.data.as_ref());
        }

        segments
            .next()
            .timeout(futures_time::time::Duration::from_millis(10))
            .await
            .expect_err("should time out");

        writer.flush().await?;

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(4, seg.frame_id);
        assert_eq!(0, seg.seq_idx);
        assert_eq!(1, seg.seq_len);
        assert_eq!(4, seg.data.len());
        assert_eq!(&data[4500..], seg.data.as_ref());

        let data = hopr_crypto_random::random_bytes::<1500>();
        writer.write_all(&data).await?;

        for i in 0..3 {
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            assert_eq!(5, seg.frame_id);
            assert_eq!((i % 3) as SeqNum, seg.seq_idx);
            assert_eq!(3, seg.seq_len);
            assert_eq!(500, seg.data.len());
            assert_eq!(&data[i * 500..i * 500 + 500], seg.data.as_ref());
        }

        Ok(())
    }
}
