use crate::prelude::protocol::SessionMessage;
use crate::session::errors::SessionError;
use crate::session::frame::FrameId;
use crate::session::frame::SeqNum;
use crate::session::Segment;
use futures::{ready, Sink, SinkExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::instrument;

/// `C` is MTU size, `F` is frame size.
pub struct Segmenter<const C: usize> {
    seg_buffer: Vec<u8>,
    ready_segments: Vec<Segment>,
    next_frame_id: FrameId,
    current_frame_len: usize,
    frame_size: usize,
    closed: bool,
    tx: Pin<Box<dyn Sink<Segment, Error = SessionError> + Send>>,
    flush_each_segment: bool,
}

impl<const C: usize> Segmenter<C> {
    const PAYLOAD_CAPACITY: usize = C - SessionMessage::<C>::SEGMENT_OVERHEAD;

    pub fn new(frame_size: usize, capacity: usize) -> (Self, impl futures::Stream<Item = Segment> + Send) {
        assert!(frame_size >= C, "frame size must be at least MTU");
        assert!(
            frame_size <= C * SessionMessage::<C>::MAX_SEGMENTS_PER_FRAME,
            "frame size too big for the given MTU"
        );

        let (tx, rx) = futures::channel::mpsc::channel::<Segment>(capacity);

        (
            Self {
                seg_buffer: Vec::with_capacity(Self::PAYLOAD_CAPACITY),
                ready_segments: Vec::with_capacity(frame_size / C + 1),
                next_frame_id: 1,
                current_frame_len: 0,
                closed: false,
                tx: Box::pin(tx.sink_map_err(|e| SessionError::ProcessingError(e.to_string()))),
                frame_size,
                flush_each_segment: false, // TODO:
            },
            rx,
        )
    }

    #[instrument(name = "Segmenter::poll_flush_segments", level = "trace", skip(self, cx), fields(frame_id = self.next_frame_id, seq_len = self.ready_segments.len()))]
    fn poll_flush_segments(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SessionError>> {
        let seq_len = self.ready_segments.len();
        tracing::trace!("flushing segments");

        let all_segments = self.ready_segments.drain(..).collect::<Vec<_>>();
        for (i, mut seg) in all_segments.into_iter().enumerate() {
            seg.seq_idx = i as SeqNum;
            seg.seq_len = seq_len as SeqNum;

            ready!(self.tx.as_mut().poll_ready(cx))?;

            tracing::trace!(frame_id = seg.frame_id, seq_idx = seg.seq_idx, "segment out");
            self.tx.as_mut().start_send(seg)?;

            if self.flush_each_segment {
                ready!(self.tx.as_mut().poll_flush(cx))?;
                tracing::trace!(frame_id = self.next_frame_id, seg_idx = i, "segment flushed out");
            }
        }

        if !self.flush_each_segment {
            ready!(self.tx.as_mut().poll_flush(cx))?;
            tracing::trace!(frame_id = self.next_frame_id, "frame flushed out");
        }

        self.next_frame_id += 1;
        self.current_frame_len = 0;

        Poll::Ready(Ok(()))
    }

    #[instrument(name = "Segmenter::complete_segment", level = "trace", skip(self), fields(frame_id = self.next_frame_id, seq_len = self.ready_segments.len()))]
    fn complete_segment(&mut self) {
        let new_segment = Segment {
            frame_id: self.next_frame_id,
            seq_idx: 0,
            seq_len: 0,
            data: self.seg_buffer.clone().into_boxed_slice(),
        };
        self.seg_buffer.clear();

        let seg_len = new_segment.data.len();
        self.current_frame_len += seg_len;

        tracing::trace!(
            frame_id = self.next_frame_id,
            seq_idx = self.ready_segments.len(),
            bytes_added = seg_len,
            remaining_in_frame = self.frame_size - self.current_frame_len,
            "completed segment"
        );

        self.ready_segments.push(new_segment);
    }
}

impl<const C: usize> futures::io::AsyncWrite for Segmenter<C> {
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

        let len_to_write = buf.len().min(Self::PAYLOAD_CAPACITY - self.seg_buffer.len());
        self.seg_buffer.extend_from_slice(&buf[..len_to_write]);
        tracing::trace!(
            len = len_to_write,
            remaining = Self::PAYLOAD_CAPACITY - self.seg_buffer.len(),
            "add segment data"
        );

        if self.seg_buffer.len() == Self::PAYLOAD_CAPACITY {
            if self.current_frame_len + len_to_write > self.frame_size {
                tracing::trace!("frame full");
                ready!(self.as_mut().poll_flush_segments(cx)).map_err(std::io::Error::other)?;
            }

            self.complete_segment();

            if self.current_frame_len == self.frame_size {
                ready!(self.as_mut().poll_flush_segments(cx)).map_err(std::io::Error::other)?;
            }
        }

        tracing::trace!(len = len_to_write, "ready");
        Poll::Ready(Ok(len_to_write))
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
            self.complete_segment();
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

        self.closed = true;

        if !self.seg_buffer.is_empty() {
            tracing::trace!(
                frame_id = self.next_frame_id,
                len = self.seg_buffer.len(),
                "partial frame"
            );
            self.complete_segment();
        }

        ready!(self.as_mut().poll_flush_segments(cx).map_err(std::io::Error::other))?;
        self.tx.as_mut().poll_close(cx).map_err(std::io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::anyhow;
    use futures::{pin_mut, AsyncWriteExt, StreamExt};
    use std::time::Duration;

    #[tokio::test]
    async fn segmenter_should_not_segment_small_data_unless_flushed() -> anyhow::Result<()> {
        let (mut writer, segments) = Segmenter::<510>::new(1500, 1024);
        writer.write_all(b"test").await?;

        pin_mut!(segments);
        segments
            .next()
            .timeout(Duration::from_millis(10))
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
        let (mut writer, segments) = Segmenter::<510>::new(1500, 1024);

        let mut offset = 0;
        let data = hopr_crypto_random::random_bytes::<1500>();

        while offset < data.len() {
            let written = writer.write(&data[offset..]).await?;
            assert_eq!(written, 500);
            offset += 500;
        }

        pin_mut!(segments);

        for i in 0..3 {
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            assert_eq!(1, seg.frame_id);
            assert_eq!(i as SeqNum, seg.seq_idx);
            assert_eq!(3, seg.seq_len);
            assert_eq!(500, seg.data.len());
            assert_eq!(&data[i * 500..i * 500 + 500], seg.data.as_ref());
        }

        writer.close().await?;

        assert_eq!(None, segments.next().await);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn segmenter_should_segment_complete_frame_with_misaligned_mtu() -> anyhow::Result<()> {
        const MTU: usize = 462;
        const SMTU: usize = MTU - SessionMessage::<MTU>::SEGMENT_OVERHEAD;

        let (mut writer, segments) = Segmenter::<MTU>::new(1500, 1024);

        let mut offset = 0;
        let data = hopr_crypto_random::random_bytes::<1500>();

        while offset < data.len() {
            let written = writer.write(&data[offset..]).await?;
            let expected_written = SMTU.min(data.len() - offset);
            assert_eq!(expected_written, written);
            offset += written;
        }

        writer.close().await?;

        pin_mut!(segments);

        for i in 0..(1500 / MTU) {
            let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
            assert_eq!(1, seg.frame_id);
            assert_eq!(i as SeqNum, seg.seq_idx);
            assert_eq!(((1500 / SMTU) + 1) as SeqNum, seg.seq_len);
            assert_eq!(SMTU, seg.data.len());
            assert_eq!(&data[i * SMTU..i * SMTU + SMTU], seg.data.as_ref());
        }

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(1, seg.frame_id);
        assert_eq!(3, seg.seq_idx);
        assert_eq!(((1500 / SMTU) + 1) as SeqNum, seg.seq_len);
        assert_eq!(1500 % SMTU, seg.data.len());
        assert_eq!(&data[1500 - 1500 % SMTU..], seg.data.as_ref());

        assert_eq!(None, segments.next().await);
        Ok(())
    }

    #[tokio::test]
    async fn segmenter_should_segment_multiple_complete_frames() -> anyhow::Result<()> {
        let (mut writer, segments) = Segmenter::<510>::new(1500, 1024);

        let data = hopr_crypto_random::random_bytes::<4500>();

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

        writer.close().await?;

        assert_eq!(None, segments.next().await);

        Ok(())
    }

    #[tokio::test]
    async fn segmenter_should_segment_multiple_complete_frames_and_incomplete_frame_on_close() -> anyhow::Result<()> {
        let (mut writer, segments) = Segmenter::<510>::new(1500, 1024);

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
            .timeout(Duration::from_millis(10))
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
        let (mut writer, segments) = Segmenter::<510>::new(1500, 1024);

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
            .timeout(Duration::from_millis(10))
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
