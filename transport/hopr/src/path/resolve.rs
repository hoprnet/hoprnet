//! The stage that turns a [`DestinationRouting`] into a [`ResolvedTransportRouting`] for every
//! packet this node originates.
//!
//! It sits between the merged outgoing-data stream and the SPHINX encoder, so **every** packet the
//! node originates passes through it: session payloads, Start-protocol replies, SURB keep-alives,
//! probes and cover traffic. Forwarded packets and acknowledgements do not — they are relayed from
//! inside the ingress pipeline and never reach this stage.
//!
//! That asymmetry is why this stage deserves its own tests: a fault here silences origination
//! node-wide while forwarding, acking and receiving carry on looking perfectly healthy.

use futures::{Stream, StreamExt};
use hopr_api::types::internal::routing::{DestinationRouting, ResolvedTransportRouting};
use hopr_crypto_packet::prelude::{HoprSurb, PacketSignal};
use hopr_protocol_app::prelude::ApplicationDataOut;
use tracing::{Instrument, error, trace, warn};

use super::errors::Result as PathResult;

/// How often a return route whose SURBs have not arrived is retried.
const SURB_RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(5);

/// How long the stage keeps retrying a return route whose SURBs have not arrived.
///
/// A momentary gap is normal — the SURB pool refills asynchronously, and dropping a return packet
/// on the first miss loses data on a session that was only waiting. That is why the retry exists.
///
/// What it must not do is wait indefinitely. When the counterparty is gone no SURB is ever coming,
/// and because this stage preserves submission order, one such packet withholds every other packet
/// the node would originate, for the life of the process.
///
/// One second is ~200 retry cycles, far longer than a refill needs, and inside the session's 3 s
/// frame timeout — a packet held past that is discarded by the receiver anyway, so waiting longer
/// cannot save it and can only cost the node its egress.
pub(crate) const SURB_RESOLUTION_WAIT: std::time::Duration = std::time::Duration::from_secs(1);

/// Resolves the routing of every outgoing packet, emitting the resolved packets in submission
/// order.
///
/// `resolve` is the resolution step itself — in production
/// [`PathPlanner::resolve_routing`](super::PathPlanner::resolve_routing), taking the payload size
/// hint, the maximum number of SURBs the packet can carry, and the unresolved routing.
///
/// A packet whose routing cannot be resolved is dropped and counted. A packet whose *return* routing
/// finds no SURB is retried for up to `surb_wait` before being dropped the same way; see
/// [`SURB_RESOLUTION_WAIT`] for why that bound has to exist.
///
/// Ordering is deliberate: out-of-order delivery to the entry's reassembler makes the sequencer
/// discard frames that arrive after `frame_timeout`. It is also why an unbounded wait here is fatal
/// rather than merely slow — [`buffered`](futures::StreamExt::buffered) withholds completed futures
/// behind an unfinished one, so the stall is node-wide rather than confined to one packet.
pub(crate) fn resolve_routing_stage<St, F, Fut>(
    input: St,
    resolve: F,
    distress_threshold: usize,
    concurrency: usize,
    surb_wait: std::time::Duration,
) -> impl Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>
where
    St: Stream<Item = (DestinationRouting, ApplicationDataOut)>,
    F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
    Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>)>>,
{
    input
        .map(move |(unresolved, mut data)| {
            let resolve = resolve.clone();
            async move {
                // Retry on SURB starvation: the SURB pool on the exit side refills asynchronously
                // (target 600, ~300/sec via keep-alive). Silently dropping return-path packets when
                // the pool is momentarily empty causes irreversible data loss; instead we yield
                // briefly so the pool can replenish before retrying — but only for `surb_wait`,
                // after which no SURB is coming and continuing to wait costs the node its egress.
                let deadline = std::time::Instant::now() + surb_wait;
                loop {
                    hopr_transport_session::counters::ROUTING_RESOLUTION_ATTEMPTS
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    trace!(?unresolved, "resolving routing for packet");
                    match resolve(
                        data.data.total_len(),
                        data.estimate_surbs_with_msg(),
                        unresolved.clone(),
                    )
                    .await
                    {
                        Ok((resolved, rem_surbs)) => {
                            // Set the SURB distress/out-of-SURBs flag if applicable.
                            // These flags are translated into HOPR protocol packet signals and are
                            // applicable only on the return path.
                            let mut signals_to_dst = data
                                .packet_info
                                .as_ref()
                                .map(|info| info.signals_to_destination)
                                .unwrap_or_default();

                            if resolved.is_return() {
                                signals_to_dst = match rem_surbs {
                                    Some(rem) if (1..distress_threshold.max(2)).contains(&rem) => {
                                        signals_to_dst | PacketSignal::SurbDistress
                                    }
                                    Some(0) => signals_to_dst | PacketSignal::OutOfSurbs,
                                    _ => signals_to_dst - (PacketSignal::OutOfSurbs | PacketSignal::SurbDistress),
                                };
                            } else {
                                // Unset these flags as they make no sense on the forward path.
                                signals_to_dst -= PacketSignal::SurbDistress | PacketSignal::OutOfSurbs;
                            }

                            data.packet_info.get_or_insert_default().signals_to_destination = signals_to_dst;
                            trace!(?resolved, "resolved routing for packet");
                            return Some((resolved, data));
                        }
                        Err(error) if error.is_surb() && std::time::Instant::now() >= deadline => {
                            hopr_transport_session::counters::ROUTING_RESOLUTION_SURB_TIMEOUTS
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            // Warn rather than trace: this is the only externally visible sign that
                            // a counterparty has stopped replenishing, and the outage it replaces
                            // was invisible precisely because nothing on this path said anything.
                            warn!(
                                ?unresolved,
                                ?surb_wait,
                                %error,
                                "dropping an outgoing packet: no SURB for its return path within the wait"
                            );
                            return None;
                        }
                        Err(error) if error.is_surb() => {
                            // No SURB available yet (possibly cache-wrapped); yield briefly so the
                            // pool can refill.
                            futures_timer::Delay::new(SURB_RETRY_INTERVAL).await;
                        }
                        Err(error) => {
                            hopr_transport_session::counters::ROUTING_RESOLUTION_FAILURES
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            error!(%error, "failed to resolve routing");
                            return None;
                        }
                    }
                }
            }
            .in_current_span()
        })
        .buffered(concurrency)
        .filter_map(futures::future::ready)
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use futures_time::future::FutureExt as _;
    use hopr_api::types::{
        crypto::{crypto_traits::Randomizable, prelude::*},
        internal::{
            path::ValidatedPath,
            prelude::HoprPseudonym,
            routing::{RoutingOptions, SurbMatcher},
        },
    };
    use hopr_protocol_app::prelude::{ApplicationData, Tag};

    use super::*;
    use crate::path::errors::PathPlannerError;

    /// Bound on every test in this module. The stall under test is unbounded, so a test that hits
    /// this limit has reproduced it rather than merely run slowly.
    const TEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

    /// The stage's SURB wait, shortened for the tests.
    ///
    /// Well clear of [`SURB_RETRY_INTERVAL`] so a retry-then-succeed case has room, and far enough
    /// below [`TEST_TIMEOUT`] that "the bound fired" and "the test timed out" cannot be confused.
    const TEST_SURB_WAIT: std::time::Duration = std::time::Duration::from_millis(200);

    const TEST_TAG: u64 = 1234;

    /// Concurrency of the stage under test. Any value ≥ 2 exhibits the ordering behaviour; a small
    /// one keeps the failure legible.
    const TEST_CONCURRENCY: usize = 8;

    const TEST_DISTRESS_THRESHOLD: usize = 2;

    /// The stage under test with the test constants applied, so each test differs only in the two
    /// things that matter to it: what goes in, and what resolution does.
    fn stage<St, F, Fut>(
        input: St,
        resolve: F,
    ) -> impl Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>
    where
        St: Stream<Item = (DestinationRouting, ApplicationDataOut)>,
        F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
        Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>)>>,
    {
        resolve_routing_stage(
            input,
            resolve,
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            TEST_SURB_WAIT,
        )
    }

    /// A resolution result the stage accepts. The variant is a forward one because a
    /// [`ResolvedTransportRouting::Return`] needs a real `HoprSurb`, and no test here depends on
    /// which variant came back — only on *whether* the packet was emitted.
    fn resolved() -> ResolvedTransportRouting<HoprSurb> {
        ResolvedTransportRouting::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path: ValidatedPath::direct(
                *OffchainKeypair::random().public(),
                ChainKeypair::random().public().to_address(),
            ),
            return_paths: vec![],
        }
    }

    fn forward_routing() -> DestinationRouting {
        DestinationRouting::forward_only(
            *OffchainKeypair::random().public(),
            RoutingOptions::Hops(1.try_into().expect("1 is a valid hop count")),
        )
    }

    fn return_routing(pseudonym: HoprPseudonym) -> DestinationRouting {
        DestinationRouting::Return(SurbMatcher::Pseudonym(pseudonym))
    }

    /// A packet carrying `marker` as its only payload byte, so emitted packets can be identified.
    fn packet(marker: u8) -> ApplicationDataOut {
        ApplicationDataOut::with_no_packet_info(
            ApplicationData::new(Tag::from(TEST_TAG), &[marker]).expect("a one-byte payload is valid"),
        )
    }

    fn marker_of(data: &ApplicationDataOut) -> u8 {
        data.data.plain_text[0]
    }

    /// The error the planner raises when the SURB store holds nothing for a pseudonym.
    ///
    /// This is a *permanent* condition once the counterparty is gone — it is the same error whether
    /// the pool is momentarily empty or will never be refilled again, which is precisely what makes
    /// retrying on it indefinitely unsafe.
    fn no_surb(routing: &DestinationRouting) -> PathPlannerError {
        let pseudonym = match routing {
            DestinationRouting::Return(matcher) => matcher.pseudonym().to_string(),
            DestinationRouting::Forward { .. } => unreachable!("only return routing can starve for SURBs"),
        };
        PathPlannerError::Surb(format!("no surb for pseudonym {pseudonym}"))
    }

    /// A return packet for a pseudonym whose SURBs will never arrive must not withhold the packets
    /// queued behind it.
    ///
    /// This is the `london-01` outage in miniature. The exit kept a session slot alive after its
    /// initiator had gone, so its keep-alive stream went on emitting return-routed packets for a
    /// pseudonym with no SURBs. Resolution for that packet can never succeed, and because the stage
    /// preserves submission order, every subsequent packet was withheld behind it: the node
    /// originated nothing for 1h44m while forwarding, acking and receiving carried on normally.
    ///
    /// The assertion is that the packets behind the starved one are **emitted** — not merely that
    /// nothing errored. A stalled origination stage is externally indistinguishable from an idle
    /// node, which is what made the live outage cost a day to find.
    #[test_log::test(tokio::test)]
    async fn a_starved_return_packet_should_not_withhold_the_packets_behind_it() -> anyhow::Result<()> {
        let starved = HoprPseudonym::random();
        let dropped_before = hopr_transport_session::counters::routing_resolution_surb_timeout_count();

        let input = futures::stream::iter(vec![
            (return_routing(starved), packet(0)),
            (forward_routing(), packet(1)),
            (forward_routing(), packet(2)),
        ]);

        let emitted = stage(
            input,
            |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match routing {
                    DestinationRouting::Return(_) => Err(no_surb(&routing)),
                    DestinationRouting::Forward { .. } => Ok((resolved(), None)),
                }
            },
        )
        .take(2)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await;

        let emitted = emitted.map_err(|_| {
            anyhow::anyhow!(
                "origination stalled: the two forward packets queued behind a return packet whose pseudonym has no \
                 SURBs were never emitted within {TEST_TIMEOUT:?}. Every packet this node originates passes through \
                 this stage, so this is a node-wide origination outage."
            )
        })?;

        assert_eq!(
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            vec![1, 2],
            "the packets behind the starved one must be emitted, in order"
        );

        // The starved packet is dropped, and that has to be *countable*. A silent drop leaves the
        // operator with the same nothing the unbounded wait did: traffic missing and no signal
        // saying why.
        assert!(
            hopr_transport_session::counters::routing_resolution_surb_timeout_count() > dropped_before,
            "the starved packet was dropped without being counted, so the condition stays invisible"
        );

        Ok(())
    }

    /// A return packet whose SURBs are only momentarily absent must still be emitted once they
    /// arrive.
    ///
    /// This is the behaviour the retry exists for, and it constrains any bound placed on that
    /// retry: dropping a return packet on the first SURB error would lose data on a session that
    /// was merely waiting for its pool to refill.
    #[test_log::test(tokio::test)]
    async fn a_return_packet_should_be_emitted_once_its_surbs_arrive() -> anyhow::Result<()> {
        const FAILURES_BEFORE_SUCCESS: usize = 3;

        let attempts = Arc::new(AtomicUsize::new(0));
        let input = futures::stream::iter(vec![(return_routing(HoprPseudonym::random()), packet(7))]);

        let emitted = {
            let attempts = attempts.clone();
            stage(input, move |_size_hint, _max_surbs, routing: DestinationRouting| {
                let attempts = attempts.clone();
                async move {
                    if attempts.fetch_add(1, Ordering::Relaxed) < FAILURES_BEFORE_SUCCESS {
                        Err(no_surb(&routing))
                    } else {
                        Ok((resolved(), Some(0)))
                    }
                }
            })
            .take(1)
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "a return packet whose SURBs arrived after {FAILURES_BEFORE_SUCCESS} retries was never emitted"
                )
            })?
        };

        assert_eq!(
            emitted.len(),
            1,
            "the return packet must be emitted once its SURBs arrive"
        );
        assert_eq!(marker_of(&emitted[0].1), 7);
        assert!(
            attempts.load(Ordering::Relaxed) > FAILURES_BEFORE_SUCCESS,
            "the stage must retry rather than drop on the first SURB error"
        );

        Ok(())
    }

    /// A hard (non-SURB) resolution failure drops that packet and lets the rest through.
    ///
    /// The contrast with the starvation case is the point: the stage already knows how to drop a
    /// packet it cannot route and carry on. Only the SURB branch retries without a bound.
    #[test_log::test(tokio::test)]
    async fn a_hard_resolution_failure_should_drop_only_its_own_packet() -> anyhow::Result<()> {
        let input = futures::stream::iter(vec![
            (forward_routing(), packet(0)),
            (forward_routing(), packet(1)),
            (forward_routing(), packet(2)),
        ]);

        let emitted = stage(
            input,
            |_size_hint, _max_surbs, _routing: DestinationRouting| async move {
                static SEEN: AtomicUsize = AtomicUsize::new(0);
                if SEEN.fetch_add(1, Ordering::Relaxed) == 0 {
                    Err(PathPlannerError::Api("no path".into()))
                } else {
                    Ok((resolved(), None))
                }
            },
        )
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("a hard resolution failure stalled the stage instead of dropping its packet"))?;

        assert_eq!(
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            vec![1, 2],
            "only the unroutable packet is dropped; the rest keep their order"
        );

        Ok(())
    }
}
