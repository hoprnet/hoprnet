mod reassembly;
mod segmenter;
mod sequencer;

use std::rc::Rc;
use futures::SinkExt;
pub(crate) use reassembly::Reassembler;
pub(crate) use segmenter::Segmenter;
pub(crate) use sequencer::Sequencer;
use tracing::Instrument;

use crate::session::{
    errors::SessionError,
    frames::{Frame, FrameDashMap, FrameHashMap, FrameInspector, FrameMap, OrderedFrame, Segment},
};
use crate::session::frames::FrameId;
use crate::utils::Woc;

fn build_reconstructor<M, R>(
    id: &str,
    reassembler: Reassembler<M>,
    sequencer: Sequencer<OrderedFrame>,
    reassembled_frame_ids: R,
) -> (
    impl futures::Sink<Segment, Error = SessionError>,
    impl futures::Stream<Item = Result<Frame, SessionError>>,
)
where 
    M: FrameMap + Send + 'static, 
    R: futures::Sink<FrameId, Error = SessionError> + Send + Unpin + 'static 
{
    use futures::StreamExt;

    let (sink, rs_stream) = reassembler.split();
    let (seq_sink, stream) = sequencer.split();

    let id = id.to_string();
    hopr_async_runtime::prelude::spawn(
        async {
            match rs_stream
                .filter_map(|maybe_frame| async {
                    // Frames that fail to reassemble will eventually be
                    // discarded in the sequencer as missing,
                    // so we're safe to filter them out here and only log them.
                    maybe_frame
                        .inspect_err(|error| tracing::error!(%error, "failed to reassemble frame"))
                        .ok()
                        .map(|f| Ok(Woc::new(OrderedFrame(f))))
                })
                .forward(
                    // The first sink in Fanout gets a cloned value, the second one gets the non-cloned value (original)
                    reassembled_frame_ids
                        .with(|clone: Woc<OrderedFrame>| futures::future::ready(clone.inspect(|f| f.0.frame_id).ok_or(SessionError::InvalidState("value cannot be dropped".into()))))
                        .fanout(seq_sink.with(|original: Woc<OrderedFrame>| futures::future::ready(original.into_inner().ok_or(SessionError::InvalidState("value is guaranteed to be original".into())))))
                )
                .await
            {
                Ok(_) => tracing::debug!("frame reconstructor finished"),
                Err(error) => tracing::error!(%error, "frame reconstructor finished with error"),
            }
        }
        .instrument(tracing::debug_span!("FrameReconstructor", session_id = %id)),
    );
    (sink, stream.map(|f| f.map(|of| of.0)))
}

/// Builds a frame reconstructor - a [`Reassembler`] followed by [`Sequencer`].
///
/// The incoming segments are first reassembled into complete frames by the [`Reassembler`],
/// and then passed into the [`Sequencer`] which returns them in the right order by the Frame ID.
pub fn frame_reconstructor(
    id: &str,
    frame_timeout: std::time::Duration,
    capacity: usize,
) -> (
    impl futures::Sink<Segment, Error = SessionError>,
    impl futures::Stream<Item = Result<Frame, SessionError>>,
) {
    build_reconstructor(
        id,
        Reassembler::<FrameHashMap>::new(frame_timeout, capacity),
        Sequencer::new(sequencer::SequencerConfig {
            timeout: frame_timeout,
            capacity,
            ..Default::default()
        }),
        futures::sink::drain().sink_map_err(|_| SessionError::DataTooLong), // TODO
    )
}

/// Similar to [`frame_reconstructor`], but returns also a [`FrameInspector`] that can be used to
/// inspect the incomplete frames in the reassembler.
///
/// This reconstructor is slower than the one created via [`frame_reconstructor`], but is required
/// in stateful sockets.
pub fn frame_reconstructor_with_inspector(
    id: &str,
    frame_timeout: std::time::Duration,
    capacity: usize,
) -> (
    impl futures::Sink<Segment, Error = SessionError>,
    impl futures::Stream<Item = Result<Frame, SessionError>>,
    FrameInspector,
) {
    let reassembler = Reassembler::<FrameDashMap>::new(frame_timeout, capacity);
    let inspector = reassembler.new_inspector();

    let (sink, stream) = build_reconstructor(
        id,
        reassembler,
        Sequencer::new(sequencer::SequencerConfig {
            timeout: frame_timeout,
            capacity,
            ..Default::default()
        }),
        futures::sink::drain().sink_map_err(|_| SessionError::DataTooLong), // TODO
    );

    (sink, stream, inspector)
}

/// Helper function to segment `data` into segments of a given ` max_segment_size ` length.
/// All segments are tagged with the same `frame_id`.
#[cfg(test)]
pub fn segment<T: AsRef<[u8]>>(
    data: T,
    max_segment_size: usize,
    frame_id: u32,
) -> crate::session::errors::Result<Vec<Segment>> {
    use crate::session::frames::SeqNum;

    if frame_id == 0 {
        return Err(SessionError::InvalidFrameId);
    }

    if max_segment_size == 0 {
        return Err(SessionError::InvalidSegmentSize);
    }

    let data = data.as_ref();

    let num_chunks = data.len().div_ceil(max_segment_size);
    if num_chunks > SeqNum::MAX as usize {
        return Err(SessionError::DataTooLong);
    }

    let chunks = data.chunks(max_segment_size);

    let seq_len = chunks.len() as SeqNum;
    Ok(chunks
        .enumerate()
        .map(|(idx, data)| Segment {
            frame_id,
            seq_len,
            seq_idx: idx as u8,
            data: data.into(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{AsyncReadExt, AsyncWriteExt, StreamExt, TryStreamExt};
    use futures_time::future::FutureExt;
    use rand::prelude::*;

    use super::*;
    use crate::prelude::errors::SessionError;

    const RNG_SEED: [u8; 32] = hex_literal::hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

    #[tokio::test]
    async fn framed_reconstructor_should_reconstruct_frames() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, seq_stream) = frame_reconstructor("test", Duration::from_secs(5), 1024);

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 22, f.frame_id).unwrap())
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let jh = hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

        let actual = seq_stream
            .try_collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await??;

        assert_eq!(actual, expected);

        let _ = jh.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn frame_reconstructor_should_discard_missing_segment() -> anyhow::Result<()> {
        let expected = (1u32..=10)
            .map(|frame_id| Frame {
                frame_id,
                data: hopr_crypto_random::random_bytes::<100>().into(),
            })
            .collect::<Vec<_>>();

        let (r_sink, seq_stream) = frame_reconstructor("test", Duration::from_millis(50), 1024);

        let mut segments = expected
            .iter()
            .cloned()
            .flat_map(|f| segment(f.data, 22, f.frame_id).unwrap())
            .filter(|s| s.frame_id != 4 || s.seq_idx != 1)
            .collect::<Vec<_>>();

        let mut rng = StdRng::from_seed(RNG_SEED);
        segments.shuffle(&mut rng);

        let jh = hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

        let actual = seq_stream
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

        let _ = jh.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_segmenter_reconstructor_should_work_together() -> anyhow::Result<()> {
        let (mut data_in, segments_out) = Segmenter::<462>::new(1500, 1024);
        let (segments_in, data_out) = frame_reconstructor("test", Duration::from_secs(10), 1024);

        let jh = tokio::task::spawn(segments_out.map(Ok).forward(segments_in));

        let data_written = hopr_crypto_random::random_bytes::<9001>();

        let data_read = tokio::task::spawn(async move {
            let mut out = Vec::new();
            let mut data_out = data_out
                .inspect(|frame| tracing::debug!("{:?}", frame))
                .map_err(std::io::Error::other)
                .into_async_read();

            data_out.read_to_end(&mut out).await?;
            Ok::<_, std::io::Error>(out)
        })
        .timeout(futures_time::time::Duration::from_secs(5));

        data_in.write_all(&data_written).await?;
        data_in.close().await?;

        let data_read = data_read.await???;
        jh.await??;

        assert_eq!(&data_written, data_read.as_slice());

        Ok(())
    }
}
