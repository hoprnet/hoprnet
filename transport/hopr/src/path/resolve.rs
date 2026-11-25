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

use std::{
    collections::{HashMap, VecDeque},
    task::Poll,
};

use futures::{FutureExt, Stream, StreamExt, stream::FuturesUnordered};
use hopr_api::types::internal::{
    prelude::HoprPseudonym,
    routing::{DestinationRouting, ResolvedTransportRouting},
};
use hopr_crypto_packet::prelude::{HoprSurb, PacketSignal};
use hopr_protocol_app::prelude::{ApplicationDataOut, SurbRefund};
use tracing::{Instrument, error, trace, warn};

use super::errors::Result as PathResult;

/// How often a return route whose SURBs have not arrived is retried.
const SURB_RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(5);

/// The most return packets of one pseudonym that may queue behind the one waiting for a SURB.
///
/// A packet beyond it is dropped at once, as if its wait had run out.
const MAX_WAITING_PER_PSEUDONYM: usize = 256;

/// The most return packets that may wait for a SURB at the same time, over all pseudonyms.
///
/// Bounds the memory and the retries however many counterparties stop sending SURBs. With no room
/// left, a return packet whose pseudonym has none waiting gets one attempt and no wait, and one
/// whose pseudonym has a packet waiting is dropped at once.
const MAX_WAITING: usize = 1024;

/// Default for [`PacketPipelineConfig::surb_resolution_wait`][cfg], used when it is unset.
///
/// A momentary gap is normal — the SURB pool refills asynchronously, and dropping a return packet
/// on the first miss loses data on a session that was only waiting. That is why the retry exists.
///
/// What it must not do is wait indefinitely. When the counterparty is gone no SURB is ever coming,
/// and the packet would hold the later packets of its pseudonym that queue behind it, and its room
/// among the waiting packets, for the life of the process.
///
/// Six seconds is deliberately generous, because this is a *library* default and the useful ceiling
/// belongs to the caller. A packet held past the session's frame timeout — 3 s for the sessions
/// hoprd runs — cannot be used by the receiver anyway, so a deployment that knows its own timeout
/// should configure something tighter, as hoprd does. What a library default must not do is drop a
/// packet a slower or reliable session would still have made use of, so it errs long and leaves the
/// tightening to whoever knows the traffic.
///
/// [cfg]: crate::protocol::PacketPipelineConfig::surb_resolution_wait
pub(crate) const DEFAULT_SURB_RESOLUTION_WAIT: std::time::Duration = std::time::Duration::from_secs(6);

/// The wait a node should use, given what its configuration says.
///
/// Unset means the default; zero is honoured as "do not wait", which drops a return packet the
/// first time its SURBs are missing. Kept as one function so both meanings live in one place
/// rather than being re-derived at the call site.
pub(crate) fn surb_resolution_wait(configured: Option<std::time::Duration>) -> std::time::Duration {
    configured.unwrap_or(DEFAULT_SURB_RESOLUTION_WAIT)
}

/// Resolves the routing of every outgoing packet.
///
/// `resolve` is the resolution step itself — in production
/// [`PathPlanner::resolve_routing`](super::PathPlanner::resolve_routing), taking the payload size
/// hint, the maximum number of SURBs the packet can carry, and the unresolved routing.
///
/// A packet whose routing cannot be resolved is dropped and counted. A packet whose *return* routing
/// finds no SURB is retried for up to `surb_wait` before being dropped the same way; see
/// [`DEFAULT_SURB_RESOLUTION_WAIT`] for why that bound has to exist.
///
/// Ordering is deliberate: out-of-order delivery to the entry's reassembler makes the sequencer
/// discard frames that arrive after `frame_timeout`. Forward routes are resolved concurrently in a
/// [`buffered`](futures::StreamExt::buffered) stage that keeps submission order. A return route may
/// wait for a SURB, and `buffered` withholds completed futures behind an unfinished one, so return
/// routes are resolved after that stage, one at a time per pseudonym (see [`resolve_per_pseudonym`]):
/// a packet that waits is overtaken by packets of other pseudonyms, never by one of its own.
pub(crate) fn resolve_routing_stage_with_context<St, F, Fut>(
    input: St,
    resolve: F,
    distress_threshold: usize,
    concurrency: usize,
    surb_wait: std::time::Duration,
) -> impl Stream<Item = crate::trace_context::WithContext<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>>
where
    St: Stream<Item = crate::trace_context::WithContext<(DestinationRouting, ApplicationDataOut)>>,
    F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
    Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>>,
{
    let staged = input
        .map(move |message| {
            let ((unresolved, mut data), span) = message.into_parts();
            let request_span = tracing::debug_span!(
                parent: &span,
                "transport.send",
                packet_id = crate::trace_context::next_packet_id()
            );
            let span = if request_span.is_disabled() { span } else { request_span };
            let output_span = span.clone();
            let resolve = resolve.clone();
            // Only a return route can wait for a SURB, and that wait must not run in the ordered stage.
            let returns_to = match &unresolved {
                DestinationRouting::Return(matcher) => Some((matcher.pseudonym(), data.surb_refund.clone())),
                DestinationRouting::Forward { .. } => None,
            };
            // Taken at submission: a packet that queues behind another of its pseudonym waits no longer
            // for that.
            let deadline = std::time::Instant::now() + surb_wait;
            let resolution = async move {
                // Retry on SURB starvation: the SURB pool on the exit side refills asynchronously
                // (target 600, ~300/sec via keep-alive). Silently dropping return-path packets when
                // the pool is momentarily empty causes irreversible data loss; instead we yield
                // briefly so the pool can replenish before retrying — but only for `surb_wait`,
                // after which no SURB is coming.
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
                        Ok((resolved, rem_surbs, generation)) => {
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

                            let info = data.packet_info.get_or_insert_default();
                            info.signals_to_destination = signals_to_dst;
                            // Carry the generation captured with the return-path plan alongside the
                            // packet, so the encode stage stamps the plan's generation rather than
                            // re-reading a possibly-newer one after a concurrent re-plan.
                            info.surb_generation = generation;
                            trace!(?resolved, "resolved routing for packet");
                            return Some(crate::trace_context::WithContext::new((resolved, data), output_span));
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
                            // Its sender may have counted the SURB as consumed already, and the packet
                            // leaves without one.
                            data.surb_refund.give_back();
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
            .instrument(tracing::debug_span!(parent: &span, "routing.resolve"))
            .instrument(span);
            async move {
                match returns_to {
                    Some((pseudonym, refund)) => Staged::Return(pseudonym, refund, resolution),
                    None => Staged::Done(resolution.await),
                }
            }
        })
        .buffered(concurrency);
    resolve_per_pseudonym(staged)
}

/// A packet whose routing is resolved.
type Resolved = crate::trace_context::WithContext<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>;

/// A packet as the ordered stage hands it on.
#[allow(
    clippy::large_enum_variant,
    reason = "a transient item, and boxing it would cost every forward packet an allocation"
)]
enum Staged<Fut> {
    /// Its resolution ran in the ordered stage: the resolved packet, or nothing if it was dropped.
    Done(Option<Resolved>),
    /// A return route with the refund of its packet: its resolution has not started.
    Return(HoprPseudonym, SurbRefund, Fut),
}

/// Runs the resolutions of the return routes, one at a time per pseudonym.
///
/// This is where a packet waits for a SURB. It holds up the later packets of its own pseudonym, which
/// queue behind it in submission order, and nothing else (hoprnet#8753).
fn resolve_per_pseudonym<St, Fut>(staged: St) -> impl Stream<Item = Resolved>
where
    St: Stream<Item = Staged<Fut>>,
    Fut: Future<Output = Option<Resolved>>,
{
    let mut staged = Box::pin(staged.fuse());
    // The packets queued behind the running resolution of their pseudonym.
    let mut queues = HashMap::<HoprPseudonym, VecDeque<(SurbRefund, Fut)>>::new();
    let mut queued = 0;
    let mut running = FuturesUnordered::new();
    let run = |pseudonym, resolution: Fut| resolution.map(move |resolved| (pseudonym, resolved));
    // A packet with no room to wait in is dropped as the wait running out drops one.
    let drop_unresolved = |pseudonym: HoprPseudonym, refund: SurbRefund| {
        hopr_transport_session::counters::ROUTING_RESOLUTION_SURB_TIMEOUTS
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        tracing::debug!(%pseudonym, "dropping an outgoing packet: too many packets are waiting for a SURB");
        refund.give_back();
    };
    futures::stream::poll_fn(move |cx| {
        loop {
            // A finished resolution lets the next packet of its pseudonym start.
            if let Poll::Ready(Some((pseudonym, resolved))) = running.poll_next_unpin(cx) {
                match queues.get_mut(&pseudonym).and_then(VecDeque::pop_front) {
                    Some((_, next)) => {
                        queued -= 1;
                        running.push(run(pseudonym, next));
                    }
                    None => {
                        queues.remove(&pseudonym);
                    }
                }
                match resolved {
                    Some(resolved) => return Poll::Ready(Some(resolved)),
                    None => continue,
                }
            }
            let (pseudonym, refund, resolution) = match staged.poll_next_unpin(cx) {
                Poll::Ready(Some(Staged::Done(Some(resolved)))) => return Poll::Ready(Some(resolved)),
                Poll::Ready(Some(Staged::Done(None))) => continue,
                Poll::Ready(Some(Staged::Return(pseudonym, refund, resolution))) => (pseudonym, refund, resolution),
                Poll::Ready(None) if running.is_empty() => return Poll::Ready(None),
                Poll::Ready(None) | Poll::Pending => return Poll::Pending,
            };
            let room = running.len() + queued < MAX_WAITING;
            match queues.get_mut(&pseudonym) {
                // Behind an earlier packet of its pseudonym, with no attempt of its own: it neither
                // overtakes that packet nor takes the SURB it is waiting for.
                Some(queue) if room && queue.len() < MAX_WAITING_PER_PSEUDONYM => {
                    queue.push_back((refund, resolution));
                    queued += 1;
                }
                Some(_) => drop_unresolved(pseudonym, refund),
                None if room => {
                    queues.insert(pseudonym, VecDeque::new());
                    running.push(run(pseudonym, resolution));
                }
                // No room to wait in: one attempt.
                None => {
                    let mut resolution = std::pin::pin!(resolution);
                    match resolution.as_mut().poll(cx) {
                        Poll::Ready(Some(resolved)) => return Poll::Ready(Some(resolved)),
                        Poll::Ready(None) => {}
                        Poll::Pending => drop_unresolved(pseudonym, refund),
                    }
                }
            }
        }
    })
}

#[cfg(test)]
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
    Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>>,
{
    resolve_routing_stage_with_context(
        input.map(Into::into),
        resolve,
        distress_threshold,
        concurrency,
        surb_wait,
    )
    .map(|message| message.value)
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
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
    use hopr_protocol_app::prelude::{ApplicationData, SurbRefund, Tag};

    use super::*;
    use crate::path::errors::PathPlannerError;

    #[rstest::rstest]
    #[case(hopr_transport_session::Capabilities::default())]
    #[case(hopr_transport_session::Capability::Segmentation.into())]
    #[case(hopr_transport_session::Capability::Segmentation | hopr_transport_session::Capability::RetransmissionAck)]
    #[cfg(feature = "tracing-context")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial_test::serial(routing_logs)]
    async fn queued_sessions_should_keep_their_context_through_routing_and_encoding(
        #[case] capabilities: hopr_transport_session::Capabilities,
    ) -> anyhow::Result<()> {
        use anyhow::Context as _;
        use futures::AsyncWriteExt;

        use crate::{tests::support::Logs, trace_context::ContextSender};

        let logs = Logs::default();
        let subscriber = logs.subscriber(tracing::Level::DEBUG);
        logs.with_cpu_subscriber(subscriber, async {
            let (tx, rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel(4);
            let tx = ContextSender::new(tx);
            let ids = [HoprPseudonym::random(), HoprPseudonym::random()];
            for (index, id) in ids.iter().enumerate() {
                // Construct under one context, then poll outside it, as the session manager does.
                let mut session = hopr_transport_session::HoprSession::new(
                    *id,
                    forward_routing(),
                    hopr_transport_session::HoprSessionConfig {
                        capabilities,
                        ..Default::default()
                    },
                    (tx.clone(), futures::stream::pending()),
                    None,
                )?;
                session.write_all(&[index as u8]).await?;
                session.flush().await?;
            }
            drop(tx);
            let resolved_messages = resolve_routing_stage_with_context(
                rx.take(2),
                |_, _, _| async {
                    // Yield so buffered futures for different sessions overlap.
                    tokio::task::yield_now().await;
                    tracing::debug!("session route resolved");
                    Ok((resolved(), None, None))
                },
                TEST_DISTRESS_THRESHOLD,
                2,
                TEST_SURB_WAIT,
            )
            .collect::<Vec<_>>()
            .await;
            assert_eq!(resolved_messages.len(), 2);
            // The encoder receives these envelopes after routing's instrumented future exits.
            // Exercise the same CPU boundary used by the packet encoder.
            for message in resolved_messages {
                let marker = *message
                    .value
                    .1
                    .data
                    .plain_text
                    .last()
                    .context("missing session payload")?;
                let dispatch = tracing::dispatcher::get_default(Clone::clone);
                async move {
                    hopr_utils::parallelize::cpu::spawn_encode_blocking(
                        move || {
                            // Scoped test subscribers must also be installed on the Rayon thread.
                            tracing::dispatcher::with_default(&dispatch, || {
                                tracing::debug!(marker, "session CPU encoding");
                            });
                        },
                        "session_context_test",
                    )
                    .await?;
                    anyhow::Ok(())
                }
                .instrument(message.into_parts().1)
                .await?;
            }
            for event in logs
                .records("session route resolved")
                .into_iter()
                .chain(logs.records("session CPU encoding"))
            {
                assert!(event.ancestry.contains(&"session"), "missing session ancestor");
                assert!(event.ancestry.contains(&"transport.send"), "missing packet ancestor");
                assert!(!event.fields.contains_key("session_id"));
                assert!(!event.fields.contains_key("packet_id"));
            }
            assert_eq!(logs.records("session route resolved").len(), 2);
            assert_eq!(logs.records("session CPU encoding").len(), 2);
            let output = logs.output();
            for (marker, id) in ids.iter().enumerate() {
                let route = output
                    .lines()
                    .find(|line| line.contains("session route resolved") && line.contains(&id.to_string()))
                    .context("missing routed session")?;
                let packet_id = route
                    .split("packet_id=")
                    .nth(1)
                    .context("routing must expose its packet identifier")?
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>();
                let line = output
                    .lines()
                    .find(|line| line.contains("session CPU encoding") && line.contains(&format!("marker={marker}")))
                    .context("missing encoded session")?;
                assert!(line.contains(&id.to_string()), "encoder lost its session: {line}");
                assert!(
                    line.contains(&format!("packet_id={packet_id}}}")),
                    "encoder changed its packet: {line}"
                );
                assert!(
                    !line.contains(&ids[1 - marker].to_string()),
                    "session context leaked: {line}"
                );
            }
            anyhow::Ok(())
        })?
    }

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
        Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>>,
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
                    DestinationRouting::Forward { .. } => Ok((resolved(), None, None)),
                }
            },
        )
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

    /// The SURB wait of one pseudonym must not delay the packets of other pseudonyms, nor forward
    /// packets: they are emitted well before the starved packet's wait is over (hoprnet#8753).
    /// `a_starved_return_packet_should_not_withhold_the_packets_behind_it` only checks that they come
    /// out eventually.
    #[rstest::rstest]
    #[case::test_wait(TEST_SURB_WAIT)]
    #[case::hoprd_wait(std::time::Duration::from_secs(1))]
    #[test_log::test(tokio::test)]
    async fn a_starved_return_packet_should_not_delay_the_packets_of_other_pseudonyms(
        #[case] surb_wait: std::time::Duration,
    ) -> anyhow::Result<()> {
        let (starved, served) = (HoprPseudonym::random(), HoprPseudonym::random());
        let input = futures::stream::iter(vec![
            (return_routing(starved), packet(0)),
            (return_routing(served), packet(1)),
            (forward_routing(), packet(2)),
        ]);

        let started = std::time::Instant::now();
        let emitted = resolve_routing_stage(
            input,
            move |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match &routing {
                    DestinationRouting::Return(matcher) if matcher.pseudonym() == starved => Err(no_surb(&routing)),
                    DestinationRouting::Return(_) => Ok((resolved(), Some(5), None)),
                    DestinationRouting::Forward { .. } => Ok((resolved(), None, None)),
                }
            },
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            surb_wait,
        )
        .map(|(_, data)| (marker_of(&data), started.elapsed()))
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(surb_wait + TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("the stage did not end once the starved packet was dropped"))?;

        assert_eq!(
            vec![1, 2],
            emitted.iter().map(|(marker, _)| *marker).collect::<Vec<_>>(),
            "the packets behind the starved one must be emitted, in order"
        );
        for (marker, after) in emitted {
            assert!(
                after < surb_wait / 4,
                "packet {marker} was withheld for {after:?} behind a packet of another pseudonym that waited \
                 {surb_wait:?} for a SURB"
            );
        }
        Ok(())
    }

    #[test]
    fn an_unset_wait_should_fall_back_to_the_default() {
        assert_eq!(DEFAULT_SURB_RESOLUTION_WAIT, surb_resolution_wait(None));
    }

    #[test]
    fn a_configured_wait_should_be_honoured() {
        let configured = std::time::Duration::from_millis(2_500);
        assert_eq!(configured, surb_resolution_wait(Some(configured)));
    }

    /// Zero is honoured rather than treated as "unset".
    ///
    /// The sibling concurrency knobs in the same config read `Some(0)` as "use the default", so the
    /// difference is worth pinning: for a wait, zero has a meaning of its own — do not wait at all.
    #[test]
    fn a_zero_wait_should_mean_no_wait_rather_than_the_default() {
        assert_eq!(
            std::time::Duration::ZERO,
            surb_resolution_wait(Some(std::time::Duration::ZERO))
        );
    }

    /// With the wait disabled, a starved return packet is dropped on its first miss.
    ///
    /// This is the behaviour a zero wait buys, and the reason it is a footgun rather than a
    /// tuning: any momentary SURB gap becomes loss. Asserting the attempt count is what separates
    /// "did not wait" from "waited briefly".
    #[test_log::test(tokio::test)]
    async fn a_zero_wait_should_drop_a_starved_return_packet_without_retrying() -> anyhow::Result<()> {
        let attempts = Arc::new(AtomicUsize::new(0));
        let input = futures::stream::iter(vec![(return_routing(HoprPseudonym::random()), packet(0))]);

        let emitted = {
            let attempts = attempts.clone();
            resolve_routing_stage(
                input,
                move |_size_hint, _max_surbs, routing: DestinationRouting| {
                    let attempts = attempts.clone();
                    async move {
                        attempts.fetch_add(1, Ordering::Relaxed);
                        Err(no_surb(&routing))
                    }
                },
                TEST_DISTRESS_THRESHOLD,
                TEST_CONCURRENCY,
                surb_resolution_wait(Some(std::time::Duration::ZERO)),
            )
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
            .await
            .map_err(|_| anyhow::anyhow!("a zero wait still stalled instead of dropping immediately"))?
        };

        assert!(emitted.is_empty(), "the starved packet must not be emitted");
        assert_eq!(
            1,
            attempts.load(Ordering::Relaxed),
            "a zero wait must resolve once and give up, not retry"
        );

        Ok(())
    }

    /// A starved return packet gives back the SURB its sender counted for it. One whose sender counted
    /// nothing gives nothing back, and a packet that finds its SURB keeps the count.
    #[test_log::test(tokio::test)]
    async fn a_starved_return_packet_should_give_back_the_surb_its_sender_counted() -> anyhow::Result<()> {
        let refunds = Arc::new(AtomicU64::new(0));
        let counted = |marker| {
            let mut data = packet(marker);
            data.surb_refund = SurbRefund::to(refunds.clone());
            data
        };
        let (starved, served) = (HoprPseudonym::random(), HoprPseudonym::random());
        let input = futures::stream::iter(vec![
            (return_routing(starved), counted(0)),
            (return_routing(starved), packet(1)),
            (return_routing(served), counted(2)),
        ]);

        let emitted = resolve_routing_stage(
            input,
            move |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match &routing {
                    DestinationRouting::Return(matcher) if matcher.pseudonym() == served => {
                        Ok((resolved(), Some(5), None))
                    }
                    _ => Err(no_surb(&routing)),
                }
            },
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            surb_resolution_wait(Some(std::time::Duration::ZERO)),
        )
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("the stage stalled instead of dropping the starved packets"))?;

        assert_eq!(
            vec![2],
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            "only the packet that found its SURB goes on"
        );
        assert_eq!(
            1,
            refunds.load(Ordering::Relaxed),
            "only the counted packet that starved gives its SURB back"
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
                        Ok((resolved(), Some(0), None))
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
                    Ok((resolved(), None, None))
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

    /// The SURB-batch generation captured with the return-path plan (the third element of the
    /// resolve result) must ride on the emitted packet's `packet_info`, so the encode stage stamps
    /// that value instead of re-reading a possibly-newer one after a concurrent re-plan.
    #[test_log::test(tokio::test)]
    async fn resolved_packet_should_carry_the_captured_surb_generation() -> anyhow::Result<()> {
        const CAPTURED: u8 = 7;

        let input = futures::stream::iter(vec![(forward_routing(), packet(0))]);

        let emitted = stage(input, |_size_hint, _max_surbs, _routing| async move {
            Ok((resolved(), None, Some(CAPTURED)))
        })
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("the stage did not emit the packet within {TEST_TIMEOUT:?}"))?;

        let (_, data) = emitted
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no packet emitted"))?;
        assert_eq!(
            data.packet_info.and_then(|info| info.surb_generation),
            Some(CAPTURED),
            "the generation captured at resolution must ride on packet_info to the encode stage"
        );

        Ok(())
    }

    type Resolution = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>;

    /// A resolver backed by a SURB pool: a return routing of `pseudonym` takes one SURB from `pool` or
    /// finds none; every other routing resolves.
    fn pool_resolver(
        pool: Arc<AtomicUsize>,
        pseudonym: HoprPseudonym,
    ) -> impl Fn(usize, usize, DestinationRouting) -> std::future::Ready<Resolution> + Clone {
        move |_size_hint, _max_surbs, routing: DestinationRouting| {
            std::future::ready(match &routing {
                DestinationRouting::Return(matcher) if matcher.pseudonym() == pseudonym => {
                    match pool.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1)) {
                        Ok(before) => Ok((resolved(), Some(before - 1), None)),
                        Err(_) => Err(no_surb(&routing)),
                    }
                }
                _ => Ok((resolved(), Some(5), None)),
            })
        }
    }

    /// A packet whose sender counted a SURB for it, refunded to `refunds` if it is dropped.
    fn counted(marker: u8, refunds: &Arc<AtomicU64>) -> ApplicationDataOut {
        let mut data = packet(marker);
        data.surb_refund = SurbRefund::to(refunds.clone());
        data
    }

    async fn sleep(duration: std::time::Duration) {
        futures_timer::Delay::new(duration).await;
    }

    /// A packet submitted behind a waiting packet of its pseudonym must not take the SURB that the
    /// earlier packet is waiting for, even when that SURB arrives just before it is submitted.
    #[test_log::test(tokio::test)]
    async fn a_later_packet_should_not_take_the_surb_an_earlier_packet_of_its_pseudonym_waits_for() -> anyhow::Result<()>
    {
        let pseudonym = HoprPseudonym::random();
        let pool = Arc::new(AtomicUsize::new(0));
        let (tx, rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let mut stream = std::pin::pin!(resolve_routing_stage(
            rx,
            pool_resolver(pool.clone(), pseudonym),
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            std::time::Duration::from_secs(2),
        ));
        let limit = futures_time::time::Duration::from(TEST_TIMEOUT);

        tx.unbounded_send((return_routing(pseudonym), packet(0)))?;
        // Let the stage attempt A0 and park it.
        assert!(
            stream
                .next()
                .timeout(futures_time::time::Duration::from(std::time::Duration::from_millis(20)))
                .await
                .is_err(),
            "A0 has no SURB and must not be emitted"
        );
        // No await between these two: one SURB appears, then A1 is submitted.
        pool.fetch_add(1, Ordering::SeqCst);
        tx.unbounded_send((return_routing(pseudonym), packet(1)))?;

        let first = stream
            .next()
            .timeout(limit)
            .await
            .map_err(|_| anyhow::anyhow!("A0 was not emitted"))?
            .map(|(_, data)| marker_of(&data));
        assert_eq!(Some(0), first, "the earlier packet must get the SURB");
        assert!(
            stream
                .next()
                .timeout(futures_time::time::Duration::from(std::time::Duration::from_millis(40)))
                .await
                .is_err(),
            "A1 must still be waiting: its SURB has not arrived"
        );
        pool.fetch_add(1, Ordering::SeqCst);
        let second = stream
            .next()
            .timeout(limit)
            .await
            .map_err(|_| anyhow::anyhow!("A1 was not emitted"))?
            .map(|(_, data)| marker_of(&data));
        assert_eq!(Some(1), second);
        Ok(())
    }

    /// Each waiting packet is dropped at its own deadline, not together with the packet before it.
    #[test_log::test(tokio::test)]
    async fn each_waiting_packet_should_keep_its_own_deadline() -> anyhow::Result<()> {
        let wait = std::time::Duration::from_millis(400);
        let pseudonym = HoprPseudonym::random();
        let refunds = Arc::new(AtomicU64::new(0));
        let (tx, rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let feeder = {
            let refunds = refunds.clone();
            tokio::spawn(async move {
                let _ = tx.unbounded_send((return_routing(pseudonym), counted(0, &refunds)));
                sleep(wait / 2).await;
                let sent = std::time::Instant::now();
                let _ = tx.unbounded_send((return_routing(pseudonym), counted(1, &refunds)));
                sent
            })
        };

        let emitted = resolve_routing_stage(
            rx,
            pool_resolver(Arc::new(AtomicUsize::new(0)), pseudonym),
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            wait,
        )
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("the stage did not end"))?;
        let ended = std::time::Instant::now();
        let sent = feeder.await?;
        let waited = ended.duration_since(sent);

        assert!(emitted.is_empty());
        assert_eq!(2, refunds.load(Ordering::SeqCst));
        // Dropped with the first packet it would have waited 200 ms, with a wait restarted behind it 600 ms.
        assert!(
            waited >= wait - std::time::Duration::from_millis(20),
            "the second packet waited {waited:?}: it was dropped before its own wait was over"
        );
        assert!(
            waited < wait + std::time::Duration::from_millis(120),
            "the second packet waited {waited:?}: queueing behind the first lengthened its wait"
        );
        Ok(())
    }

    /// Packets beyond the per-pseudonym cap, which counts those queued behind the waiting one, are
    /// dropped at once rather than waiting, and do not hold up a pseudonym that is served.
    #[test_log::test(tokio::test)]
    async fn packets_beyond_the_per_pseudonym_cap_should_be_dropped_without_waiting() -> anyhow::Result<()> {
        const OVERFLOW: usize = 5;
        const SERVED_MARKER: u8 = 1;
        let (starved, served) = (HoprPseudonym::random(), HoprPseudonym::random());
        let refunds = Arc::new(AtomicU64::new(0));
        let mut packets = (0..1 + MAX_WAITING_PER_PSEUDONYM + OVERFLOW)
            .map(|_| (return_routing(starved), counted(0, &refunds)))
            .collect::<Vec<_>>();
        packets.push((return_routing(served), packet(SERVED_MARKER)));

        let mut stream = std::pin::pin!(resolve_routing_stage(
            futures::stream::iter(packets),
            pool_resolver(Arc::new(AtomicUsize::new(0)), starved),
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            std::time::Duration::from_secs(2),
        ));

        let first = stream
            .next()
            .timeout(futures_time::time::Duration::from(std::time::Duration::from_millis(
                500,
            )))
            .await
            .map_err(|_| anyhow::anyhow!("the served packet was withheld behind the starved ones"))?
            .map(|(_, data)| marker_of(&data));
        assert_eq!(Some(SERVED_MARKER), first);
        assert_eq!(
            OVERFLOW as u64,
            refunds.load(Ordering::SeqCst),
            "the overflow must be dropped at once"
        );

        let rest = stream
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(
                std::time::Duration::from_secs(2) + TEST_TIMEOUT,
            ))
            .await
            .map_err(|_| anyhow::anyhow!("the stage did not end"))?;
        assert!(rest.is_empty());
        assert_eq!(
            (1 + MAX_WAITING_PER_PSEUDONYM + OVERFLOW) as u64,
            refunds.load(Ordering::SeqCst)
        );
        Ok(())
    }

    /// A return packet finding no room among the waiting packets gets one attempt and is dropped.
    #[test_log::test(tokio::test)]
    async fn a_return_packet_should_get_one_attempt_when_no_room_is_left_to_wait_in() -> anyhow::Result<()> {
        let served = HoprPseudonym::random();
        let refunds = Arc::new(AtomicU64::new(0));
        let mut packets = (0..MAX_WAITING + 1)
            .map(|_| (return_routing(HoprPseudonym::random()), counted(0, &refunds)))
            .collect::<Vec<_>>();
        packets.push((return_routing(served), packet(1)));

        let mut stream = std::pin::pin!(resolve_routing_stage(
            futures::stream::iter(packets),
            move |_size_hint, _max_surbs, routing: DestinationRouting| {
                std::future::ready(match &routing {
                    DestinationRouting::Return(matcher) if matcher.pseudonym() == served => {
                        Ok((resolved(), Some(5), None))
                    }
                    _ => Err(no_surb(&routing)),
                })
            },
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            std::time::Duration::from_secs(2),
        ));

        let first = stream
            .next()
            .timeout(futures_time::time::Duration::from(std::time::Duration::from_millis(
                500,
            )))
            .await
            .map_err(|_| anyhow::anyhow!("the served packet was withheld behind the waiting ones"))?
            .map(|(_, data)| marker_of(&data));
        assert_eq!(Some(1), first);
        assert_eq!(
            1,
            refunds.load(Ordering::SeqCst),
            "only the packet with no room is dropped"
        );

        let rest = stream
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(
                std::time::Duration::from_secs(2) + TEST_TIMEOUT,
            ))
            .await
            .map_err(|_| anyhow::anyhow!("the stage did not end"))?;
        assert!(rest.is_empty());
        assert_eq!((MAX_WAITING + 1) as u64, refunds.load(Ordering::SeqCst));
        Ok(())
    }

    /// A slow path lookup delays the forward packets of a session, whose packets all share one
    /// pseudonym, only by the lookup: they are resolved concurrently, not dropped and not reordered.
    #[test_log::test(tokio::test)]
    async fn a_slow_path_lookup_should_neither_hold_nor_drop_the_forward_packets_of_its_session() -> anyhow::Result<()>
    {
        const PACKETS: u16 = 300;
        let pseudonym = HoprPseudonym::random();
        let input = futures::stream::iter((0..PACKETS).map(move |i| {
            let routing = DestinationRouting::Forward {
                destination: Box::new((*OffchainKeypair::random().public()).into()),
                pseudonym: Some(pseudonym),
                forward_options: RoutingOptions::Hops(1.try_into().expect("1 is a valid hop count")),
                return_options: None,
            };
            let data = ApplicationDataOut::with_no_packet_info(
                ApplicationData::new(Tag::from(TEST_TAG), &i.to_be_bytes()).expect("two bytes are a valid payload"),
            );
            (routing, data)
        }));

        let emitted = resolve_routing_stage(
            input,
            |_size_hint, _max_surbs, _routing: DestinationRouting| async {
                sleep(std::time::Duration::from_millis(20)).await;
                Ok((resolved(), None, None))
            },
            TEST_DISTRESS_THRESHOLD,
            128,
            TEST_SURB_WAIT,
        )
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from_secs(2))
        .await
        .map_err(|_| anyhow::anyhow!("the forward packets were not all emitted within 2 s"))?;

        assert_eq!(
            (0..PACKETS).collect::<Vec<_>>(),
            emitted
                .iter()
                .map(|(_, data)| u16::from_be_bytes([data.data.plain_text[0], data.data.plain_text[1]]))
                .collect::<Vec<_>>(),
            "every forward packet must be emitted, in order"
        );
        Ok(())
    }
}
