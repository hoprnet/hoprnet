//! Contains implementation of a `Session` message protocol.
//!
//! # What is `Session` protocol?
//! `Session` protocol is a simple protocol for unreliable networks that implements
//! basic TCP-like features, such as segmentation, retransmission and acknowledgement.
//!
//! The goal of this protocol is to establish a read-write session between two parties,
//! where one is a message sender and the other one is the receiver. The messages are called
//! *frames* which are split and are delivered as *segments* from the sender to the recipient.
//! The session has some reliability guarantees given by the retransmission and acknowledgement
//! capabilities of individual segments.
//!
//! # Overview of the module
//! - Protocol messages are defined in the [`protocol`] submodule.
//! - Protocol state machine is defined in the [`state`] submodule.
//! - Frames, segmentation and reassembly are defined in the `frame` submodule.
//!

//! Contains errors thrown from this module.
pub mod errors;
mod frame;
pub mod protocol;
mod reassembly;
mod sequencer;
pub mod state;
mod utils;

pub use frame::{Frame, FrameId, FrameInfo, FrameReassembler, Segment, SegmentId};

pub use utils::linear_half_normal_shuffle;

fn build_reconstructor(reassembler: reassembly::Reassembler, sequencer: sequencer::Sequencer<Frame>)
-> (
    impl futures::Sink<Segment, Error = errors::SessionError>,
    impl futures::Stream<Item = Result<Frame, errors::SessionError>>,
) {
    use futures::StreamExt;

    let (sink, rs_stream) = reassembler.split();
    let (seq_sink, stream) = sequencer.split();

    hopr_async_runtime::prelude::spawn(async {
        match rs_stream
            .filter_map(|maybe_frame| async {
                // Frames that fail to reassemble will eventually be
                // discarded in the sequencer as missing,
                // so we're safe to filter them out here and only log them.
                maybe_frame
                    .inspect_err(|e| tracing::error!("failed to reassemble frame: {e}"))
                    .ok()
                    .map(Ok)
            })
            .forward(seq_sink)
            .await
        {
            Ok(_) => tracing::debug!("frame reconstructor finished"),
            Err(e) => tracing::error!("frame reconstructor finished with error: {e}"),
        }
    });
    (sink, stream)
}

pub fn frame_reconstructor(
    frame_timeout: std::time::Duration,
    capacity: usize,
) -> (
    impl futures::Sink<Segment, Error = errors::SessionError>,
    impl futures::Stream<Item = Result<Frame, errors::SessionError>>,
) {
    build_reconstructor(
        reassembly::Reassembler::new(frame_timeout, capacity),
        sequencer::Sequencer::new(sequencer::SequencerConfig {
            timeout: frame_timeout,
            capacity,
            ..Default::default()
        })
    )
}

#[cfg(feature = "frame-inspector")]
pub fn frame_reconstructor_with_inspector(
    frame_timeout: std::time::Duration,
    capacity: usize,
) -> (
    impl futures::Sink<Segment, Error = errors::SessionError>,
    impl futures::Stream<Item = Result<Frame, errors::SessionError>>,
    reassembly::FrameInspector
) {
    let reassembler = reassembly::Reassembler::new(frame_timeout, capacity);
    let inspector = reassembler.inspect();

    let (sink, stream) = build_reconstructor(
        reassembly::Reassembler::new(frame_timeout, capacity),
        sequencer::Sequencer::new(sequencer::SequencerConfig {
            timeout: frame_timeout,
            capacity,
            ..Default::default()
        })
    );

    (sink, stream, inspector)
}

#[cfg(test)]
mod tests {
    use super::*;

    use async_std::prelude::FutureExt;
    use futures::{StreamExt, TryStreamExt};
    use rand::prelude::*;
    use std::time::Duration;

    use crate::prelude::errors::SessionError;

    const RNG_SEED: [u8; 32] = hex_literal::hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

    #[async_std::test]
    pub async fn framed_reconstructor_should_reconstruct_frames() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, seq_stream) = frame_reconstructor(Duration::from_secs(5), 1024);

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| f.segment(22).unwrap())
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let jh = hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

        let actual = seq_stream
            .try_collect::<Vec<_>>()
            .timeout(Duration::from_secs(5))
            .await??;

        assert_eq!(actual, expected);

        Ok(jh.await?)
    }

    #[test_log::test(async_std::test)]
    pub async fn frame_reconstructor_should_discard_missing_segment() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, seq_stream) = frame_reconstructor(Duration::from_millis(50), 1024);

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| f.segment(22).unwrap())
            .filter(|s| s.frame_id != 4 || s.seq_idx != 1)
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let jh = hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

        let actual = seq_stream.collect::<Vec<_>>().timeout(Duration::from_secs(5)).await?;

        assert_eq!(actual.len(), expected.len());
        for i in 0..expected.len() {
            if i != 3 {
                assert!(matches!(&actual[i], Ok(frame) if expected[i].eq(frame)));
            } else {
                assert!(matches!(actual[i], Err(SessionError::FrameDiscarded(4))))
            }
        }

        Ok(jh.await?)
    }
}
