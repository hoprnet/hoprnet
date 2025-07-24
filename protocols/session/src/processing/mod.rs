//! This module contains the three main parts for frame processing.
//! Each of them is represented using an adaptor
//! extension to [`futures::Stream`] or [`futures::Sink`]
//!
//!
//! 1. Segmenter
//! 2. Reassembler
//! 3. Sequencer
//!
//! Reassembler followed by a Sequencer is commonly called frame Reconstructor.

mod reassembly;
// mod segmenter_old;
mod segmenter;
mod sequencer;
/// Types necessary for frame reconstruction and segmentation.
pub(crate) mod types;

pub(crate) use reassembly::ReassemblerExt;
pub(crate) use segmenter::SegmenterExt;
pub(crate) use sequencer::SequencerExt;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{AsyncReadExt, AsyncWriteExt, SinkExt, StreamExt, TryStreamExt, pin_mut};
    use futures_time::future::FutureExt;
    use rand::prelude::*;

    use super::*;
    use crate::{
        errors::SessionError,
        frames::{Frame, OrderedFrame},
        utils::segment,
    };

    const RNG_SEED: [u8; 32] = hex_literal::hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

    #[tokio::test]
    async fn framed_reconstructor_should_reconstruct_frames() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
                is_terminating: false,
            })
            .collect::<Vec<_>>();

        let (reassm_tx, reassm_rx) = futures::channel::mpsc::unbounded();

        let reassm_rx = reassm_rx
            .reassembler(Duration::from_secs(5), 1024)
            .filter_map(|maybe_frame| match maybe_frame {
                Ok(frame) => futures::future::ready(Some(OrderedFrame(frame))),
                Err(_) => futures::future::ready(None),
            })
            .sequencer(Duration::from_secs(5), 1024)
            .and_then(|frame| futures::future::ok(frame.0));

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 22, f.frame_id).unwrap())
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(reassm_tx)).await??;

        let actual = reassm_rx
            .try_collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await??;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn frame_reconstructor_should_discard_missing_segment() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
                is_terminating: false,
            })
            .collect::<Vec<_>>();

        let (reassm_tx, reassm_rx) = futures::channel::mpsc::unbounded();

        let reassm_rx = reassm_rx
            .reassembler(Duration::from_secs(5), 1024)
            .filter_map(|maybe_frame| match maybe_frame {
                Ok(frame) => futures::future::ready(Some(OrderedFrame(frame))),
                Err(_) => futures::future::ready(None),
            })
            .sequencer(Duration::from_secs(5), 1024)
            .and_then(|frame| futures::future::ok(frame.0));

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 22, f.frame_id).unwrap())
            .filter(|s| s.frame_id != 4 || s.seq_idx != 1)
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(reassm_tx)).await??;

        let actual = reassm_rx
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        assert_eq!(actual.len(), expected.len());
        for i in 0..expected.len() {
            if i != 3 {
                assert!(matches!(&actual[i], Ok(frame) if expected[i].eq(frame)));
            } else {
                assert!(matches!(actual[i], Err(SessionError::FrameDiscarded(4))))
            }
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_segmenter_reconstructor_should_work_together() -> anyhow::Result<()> {
        const DATA_SIZE: usize = 9001;
        const FRAME_SIZE: usize = 1500;

        const MTU: usize = 1000;

        let (reassm_tx, reassm_rx) = futures::channel::mpsc::unbounded();

        let data_out = reassm_rx
            .reassembler(Duration::from_secs(1), 1024)
            .filter_map(|maybe_frame| match maybe_frame {
                Ok(frame) => futures::future::ready(Some(OrderedFrame(frame))),
                Err(_) => futures::future::ready(None),
            })
            .sequencer(Duration::from_secs(1), 1024)
            .and_then(|frame| futures::future::ok(frame.0));

        let mut data_in = reassm_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .segmenter::<MTU>(FRAME_SIZE);

        let data_written = hopr_crypto_random::random_bytes::<DATA_SIZE>();

        let data_read = tokio::task::spawn(async move {
            let mut frame_count = 0;
            let mut out = Vec::new();
            let data_out = data_out
                .inspect(|frame| {
                    tracing::debug!("{:?}", frame);
                    frame_count += 1;
                })
                .map_err(std::io::Error::other)
                .into_async_read();

            pin_mut!(data_out);
            data_out.read_to_end(&mut out).await?;
            Ok::<_, std::io::Error>((out, frame_count))
        })
        .timeout(futures_time::time::Duration::from_secs(5));

        data_in.write_all(&data_written).await?;
        data_in.flush().await?;
        data_in.close().await?;

        let (data_read, frame_count) = data_read.await???;

        assert_eq!(&data_written, data_read.as_slice());
        assert_eq!(DATA_SIZE / FRAME_SIZE + 1, frame_count);

        Ok(())
    }
}
