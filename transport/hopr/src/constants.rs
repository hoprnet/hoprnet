use std::time::Duration;

/// The maximum waiting time for a message send to produce a half-key challenge reply
pub const PACKET_QUEUE_TIMEOUT_MILLISECONDS: std::time::Duration = std::time::Duration::from_millis(15000);

/// Maximum number of outgoing application-layer packets buffered before the writer
/// observes backpressure (`Poll::Pending` on `poll_write`).
///
/// Caps the burst submitted to `buffered(8×N_cpus)` so tail packets never
/// exceed `PACKET_ENCODING_TIMEOUT` (150 ms). With Sphinx encoding at ~21 ms/packet
/// and 256 slots, the worst-case Rayon queue depth stays well under the timeout on
/// machines with up to ~36 cores. Safe formula: `≤ 7 × available_parallelism()`.
pub(crate) const MAXIMUM_MSG_OUTGOING_BUFFER_SIZE: usize = 256;

/// Default number of probes buffered in the probe lane before the probe emitter observes
/// backpressure. Operators set `output_priority.probe_lane_capacity` to change it.
///
/// The lane is small on purpose. `continuously_scan` records a probe in its timeout cache only once
/// the send returns, so whatever a probe waits in this lane is already inside its 3 s timeout. A lane
/// of 16 bounds that wait to 16 probes, at most 59 kB (16 × 3,669 B), and the back-pressure holds the
/// emitter instead: the probe share is applied by the scheduler, which is where the pacing belongs.
pub(crate) const MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE: usize = 16;

/// Time within Start protocol must finish session initiation.
/// This base value is always multiplied by the (max) number of hops, times 2 (for both-ways).
pub(crate) const SESSION_INITIATION_TIMEOUT_BASE: Duration = Duration::from_secs(5);

#[cfg(test)]
mod tests {
    use super::{MAXIMUM_MSG_OUTGOING_BUFFER_SIZE, MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE};

    /// Guard against accidentally inflating `MAXIMUM_MSG_OUTGOING_BUFFER_SIZE` back to a large
    /// value that would overflow the Rayon encoding queue.
    ///
    /// Safe formula: ≤ 7 × available_parallelism. 256 covers machines with up to ~36 cores.
    // Constant by construction — that is the whole point of the guard. Clippy suggests a `const`
    // block instead, which would be stronger but cannot format the offending value into the
    // message: `assert!` in const context takes a literal `&str` only. The value is worth more
    // than the earlier failure here.
    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn outgoing_buffer_size_within_backpressure_limit() {
        assert!(
            MAXIMUM_MSG_OUTGOING_BUFFER_SIZE <= 256,
            "MAXIMUM_MSG_OUTGOING_BUFFER_SIZE={MAXIMUM_MSG_OUTGOING_BUFFER_SIZE} exceeds the safe threshold; tail \
             packets will exceed PACKET_ENCODING_TIMEOUT under burst writes"
        );
    }

    /// Guard against growing the default probe lane. `continuously_scan` starts a probe's timeout
    /// before the probe waits in this lane, so every queued slot is timeout budget a probe spends not
    /// being sent. The lane stays small enough that the back-pressure, not the buffer, paces the
    /// emitter, and `output_priority.probe_lane_capacity` accepts no larger value.
    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn probe_lane_stays_small_enough_to_keep_the_probe_timeout_honest() {
        assert!(
            MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE > 0,
            "a zero-sized probe lane would stall the emitter on every probe"
        );
        assert!(
            MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE <= 32,
            "MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE={MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE} lets probes queue past \
             the probe timeout they already started"
        );
    }

    /// Verify that `CrossfireSink` signals backpressure (`Poll::Pending`) once the channel
    /// reaches `MAXIMUM_MSG_OUTGOING_BUFFER_SIZE`, preventing the Rayon encoding queue from
    /// receiving an unbounded burst.
    #[test]
    fn outgoing_channel_signals_backpressure_when_full() {
        use std::{
            pin::Pin,
            task::{Context, Poll},
        };

        use futures::Sink;
        use hopr_utils::network_types::crossfire_sink::bounded_sink_channel;

        let (mut sink, _rx) = bounded_sink_channel::<usize>(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE);
        let waker = futures::task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);

        // Standard Sink protocol: each poll_ready → start_send pair sends one item.
        // At i=0 the channel is empty; at i=N-1 the final item is buffered.
        for i in 0..MAXIMUM_MSG_OUTGOING_BUFFER_SIZE {
            assert!(
                matches!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Ready(Ok(()))),
                "poll_ready must be Ready before capacity is reached (item {i})"
            );
            Pin::new(&mut sink).start_send(i).unwrap();
        }
        // This poll_ready sends the last buffered item; channel is now exactly full.
        assert!(
            matches!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Ready(Ok(()))),
            "poll_ready must be Ready when flushing the final item into a full-but-not-yet-full channel"
        );

        // One extra item: buffer it, then poll_ready must indicate the channel is saturated.
        Pin::new(&mut sink)
            .start_send(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE)
            .unwrap();
        assert!(
            matches!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Pending),
            "CrossfireSink must return Poll::Pending when channel is at capacity ({})",
            MAXIMUM_MSG_OUTGOING_BUFFER_SIZE
        );
    }
}
