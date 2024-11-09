use futures::channel::mpsc::SendError;
use futures::{ready, Sink};
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::prelude::protocol::SessionMessage;
use crate::session::frame::FrameId;
use crate::session::frame::SeqNum;
use crate::session::Segment;

#[pin_project]
pub struct Segmenter<const C: usize> {
    frame_size: usize,
    seg_buffer: Vec<u8>,
    ready_segments: Vec<Segment>,
    next_frame_id: FrameId,
    seq_len_bytes: usize,
    closed: bool,
    #[pin]
    tx: futures::channel::mpsc::Sender<Segment>,
}

impl<const C: usize> Segmenter<C> {
    const PAYLOAD_CAPACITY: usize = C - SessionMessage::<C>::SEGMENT_OVERHEAD;

    pub fn new(frame_size: usize, capacity: usize) -> (Self, impl futures::Stream<Item = Segment>) {
        assert!(frame_size >= C, "frame size must be at least MTU");
        assert!(
            frame_size <= C * SessionMessage::<C>::MAX_SEGMENTS_PER_FRAME,
            "frame size too big for the given MTU"
        );

        let (tx, rx) = futures::channel::mpsc::channel::<Segment>(capacity);

        (
            Self {
                frame_size,
                seg_buffer: Vec::with_capacity(Self::PAYLOAD_CAPACITY),
                ready_segments: Vec::with_capacity(frame_size / C + 1),
                next_frame_id: 1,
                seq_len_bytes: 0,
                closed: false,
                tx,
            },
            rx,
        )
    }

    fn poll_flush_segments(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError>> {
        let mut this = self.project();
        let num_seg = this.ready_segments.len();

        for (i, mut seg) in this.ready_segments.drain(..).enumerate() {
            seg.seq_idx = i as SeqNum;
            seg.seq_len = num_seg as SeqNum;

            let _ = ready!(this.tx.as_mut().poll_ready(cx))?;
            this.tx.as_mut().start_send(seg)?;
        }

        let _ = ready!(this.tx.as_mut().poll_flush(cx))?;
        *this.next_frame_id += 1;
        *this.seq_len_bytes = 0;

        Poll::Ready(Ok(()))
    }

    fn complete_segment(&mut self) {
        let new_segment = Segment {
            frame_id: self.next_frame_id,
            seq_idx: 0,
            seq_len: 0,
            data: self.seg_buffer.clone().into_boxed_slice(),
        };
        self.seg_buffer.clear();
        self.ready_segments.push(new_segment);
        self.seq_len_bytes += Self::PAYLOAD_CAPACITY;
    }
}

impl<const C: usize> futures::io::AsyncWrite for Segmenter<C> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        if self.closed {
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        if buf.len() == 0 {
            return Poll::Ready(Ok(0));
        }

        let len_to_write = buf.len().min(Self::PAYLOAD_CAPACITY - self.seg_buffer.len());
        self.seg_buffer.extend_from_slice(&buf[..len_to_write]);

        if self.seg_buffer.len() == Self::PAYLOAD_CAPACITY {
            self.complete_segment();
            if self.seq_len_bytes == self.frame_size {
                ready!(self.as_mut().poll_flush_segments(cx)).map_err(std::io::Error::other)?;
            }
        }

        Poll::Ready(Ok(len_to_write))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        if self.closed {
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        if !self.seg_buffer.is_empty() {
            self.complete_segment();
        }
        self.as_mut().poll_flush_segments(cx).map_err(std::io::Error::other)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        if self.closed {
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        self.closed = true;

        if !self.seg_buffer.is_empty() {
            self.complete_segment();
        }

        let _ = ready!(self.as_mut().poll_flush_segments(cx).map_err(std::io::Error::other))?;
        self.project().tx.poll_close(cx).map_err(std::io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::anyhow;
    use async_std::prelude::FutureExt;
    use futures::{pin_mut, AsyncWriteExt, StreamExt};
    use std::time::Duration;

    #[async_std::test]
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

    #[async_std::test]
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

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(1, seg.frame_id);
        assert_eq!(0, seg.seq_idx);
        assert_eq!(3, seg.seq_len);
        assert_eq!(500, seg.data.len());
        assert_eq!(&data[..500], seg.data.as_ref());

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(1, seg.frame_id);
        assert_eq!(1, seg.seq_idx);
        assert_eq!(3, seg.seq_len);
        assert_eq!(500, seg.data.len());
        assert_eq!(&data[500..1000], seg.data.as_ref());

        let seg = segments.next().await.ok_or(anyhow!("no more segments"))?;
        assert_eq!(1, seg.frame_id);
        assert_eq!(2, seg.seq_idx);
        assert_eq!(3, seg.seq_len);
        assert_eq!(500, seg.data.len());
        assert_eq!(&data[1000..], seg.data.as_ref());

        writer.close().await?;

        assert_eq!(None, segments.next().await);
        Ok(())
    }

    #[async_std::test]
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

    #[async_std::test]
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

    #[async_std::test]
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
