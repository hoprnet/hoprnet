use std::sync::Arc;

use futures::{FutureExt, SinkExt, StreamExt};
use futures_concurrency::stream::StreamExt as _;
use hopr_api::{
    ct::{ProbeRouting, ProbingTrafficGeneration},
    graph::{EdgeTransportTelemetry, NetworkGraphError, NetworkGraphUpdate, NetworkGraphView},
    types::{
        crypto::types::OffchainPublicKey, crypto_random::Randomizable, internal::prelude::*,
        primitive::traits::AsUnixTimestamp,
    },
};
use hopr_protocol_app::{
    prelude::{ApplicationDataIn, ApplicationDataOut, OutgoingPacketInfo, ReservedTag, SurbRefund},
    v1::Tag,
};
use hopr_transport_tag_allocator::{AllocatedTag, TagAllocator};
use hopr_utils::{platform::time::native::current_time, runtime::AbortableList};

use crate::{
    HoprProbeProcess,
    config::ProbeConfig,
    content::Message,
    ping::PingQueryReplier,
    types::{NeighborProbe, NeighborTelemetry, PathTelemetry},
};

type CacheNeighborKey = (HoprPseudonym, NeighborProbe);
type CacheNeighborValue = (Box<NodeId>, std::time::Duration, Option<PingQueryReplier>);

/// A loopback probe awaiting its return, with the tag its packet travels under.
///
/// The telemetry id is the probe's identity: a tag value is recycled by the allocator as soon as
/// its entry leaves the cache, so a delayed local-drop callback for one probe must be able to tell
/// its own entry from that of a later probe on the same tag. [`PathTelemetry::id`] already
/// distinguishes them, and is what the graph keys a loopback verdict by.
#[derive(Debug)]
struct ActivePathProbe {
    telemetry: PathTelemetry,
    /// Held only so the tag returns to its allocator when the cache releases the entry.
    #[expect(dead_code, reason = "released by its Drop when the entry leaves the cache")]
    allocated_tag: Arc<AllocatedTag>,
    /// The probe has reached, or can no longer reach, a verdict of its own: it came back, or a drop
    /// site reported it never left the node. A settled probe must not be recorded as a timeout by
    /// the eviction listener, however late that listener fires.
    settled: std::sync::atomic::AtomicBool,
}

impl ActivePathProbe {
    fn settle(&self) {
        self.settled.store(true, std::sync::atomic::Ordering::Release);
    }

    fn is_settled(&self) -> bool {
        self.settled.load(std::sync::atomic::Ordering::Acquire)
    }
}

/// What the sync path-probe cache holds per tag.
type ActivePathProbeValue = Arc<ActivePathProbe>;

/// How long a timeout verdict waits before it is recorded, so a local drop that is reported after the
/// tag's TTL can still suppress it.
///
/// The two clocks disagree on purpose: a loopback tag expires after `ProbeConfig::timeout` (3 s by
/// default), while the encode stage may only report a drop after `PACKET_ENCODING_TIMEOUT` (5 s).
/// Waiting this long past expiry gives every in-flight drop the chance to settle its probe first. A
/// probe that really was lost is still recorded as a timeout, just a few seconds later — noise
/// against the graph's 60 s windows.
const LOCAL_DROP_GRACE: std::time::Duration = std::time::Duration::from_secs(3);

/// Result of classifying one incoming message through the probe layer.
pub enum ProbeDispatch {
    /// The message was a probe message and has been consumed internally.
    Consumed,
    /// The message was not related to probing; caller should route it further.
    Passthrough(HoprPseudonym, ApplicationDataIn),
}

/// Shared state used to classify incoming messages as probe or non-probe.
///
/// Obtained from [`Probe::continuously_scan`]. Use [`filter_stream`](ProbeClassifierState::filter_stream)
/// to wrap an incoming stream so that probe messages are consumed internally and non-probe messages
/// are yielded to the caller.
#[derive(Clone)]
pub struct ProbeClassifierState<G> {
    active_neighbor_probes: moka::future::Cache<CacheNeighborKey, CacheNeighborValue>,
    /// Loopback probes awaiting their return. A *sync* cache, so a local-drop callback running on
    /// the pipeline's own thread can release the tag without an executor; see the eviction listener
    /// in [`Probe::continuously_scan`] for why the timeout verdict is handed to a task.
    active_path_probes: moka::sync::Cache<Tag, ActivePathProbeValue>,
    network_graph: G,
}

impl<G> ProbeClassifierState<G>
where
    G: NetworkGraphUpdate + Clone + Send + Sync + 'static,
{
    /// Classify one incoming `(pseudonym, data)` pair.
    ///
    /// The `push_to_network` sink is used to send pong replies when the message is a Ping.
    /// Returns `Consumed` if the message was a probe, or `Passthrough` for all other messages.
    pub async fn classify<T>(
        &self,
        mut push_to_network: T,
        pseudonym: HoprPseudonym,
        in_data: ApplicationDataIn,
    ) -> ProbeDispatch
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Unpin + Send + 'static,
        T::Error: Send,
    {
        let tag: Tag = in_data.data.application_tag;

        if let Some(probe) = self.active_path_probes.remove(&tag) {
            tracing::debug!(%tag, "loopback probe successfully received");
            // Settle as well as remove: if the TTL expired first, this removal is a no-op on the
            // cache but the probe has still answered, and a deferred verdict must see that.
            probe.settle();
            self.network_graph
                .record_edge::<NeighborTelemetry, PathTelemetry>(hopr_api::graph::MeasurableEdge::Probe(Ok(
                    EdgeTransportTelemetry::Loopback(probe.telemetry),
                )));
        } else if tag == ReservedTag::Ping.into() {
            let message: anyhow::Result<Message> = in_data
                .data
                .try_into()
                .map_err(|e| anyhow::anyhow!("failed to convert data into message: {e}"));

            match message {
                Ok(message) => match message {
                    Message::Telemetry(_) => {
                        tracing::warn!(%pseudonym, "received telemetry on reserved ping tag, ignoring");
                    }
                    Message::Probe(NeighborProbe::Ping(ping)) => {
                        tracing::debug!(%pseudonym, nonce = const_hex::encode(ping), "received ping");
                        tracing::trace!(%pseudonym, nonce = const_hex::encode(ping), "wrapping a pong in the found SURB");

                        let message = Message::Probe(NeighborProbe::Pong(ping));
                        if let Ok(data) = message.try_into() {
                            let routing = DestinationRouting::Return(pseudonym.into());
                            let data = ApplicationDataOut::with_no_packet_info(data);
                            if let Err(_error) = push_to_network.send((routing, data)).await {
                                tracing::error!(%pseudonym, "failed to send back a pong");
                            }
                        } else {
                            tracing::error!(%pseudonym, "failed to convert pong message into data");
                        }
                    }
                    Message::Probe(NeighborProbe::Pong(pong)) => {
                        tracing::debug!(%pseudonym, nonce = const_hex::encode(pong), "received pong");
                        if let Some((peer, start, replier)) = self
                            .active_neighbor_probes
                            .remove(&(pseudonym, NeighborProbe::Ping(pong)))
                            .await
                        {
                            let latency = current_time().as_unix_timestamp().saturating_sub(start);

                            if let NodeId::Offchain(opk) = peer.as_ref() {
                                tracing::debug!(%pseudonym, nonce = const_hex::encode(pong), latency_ms = latency.as_millis(), "probe successful");
                                self.network_graph.record_edge::<NeighborTelemetry, PathTelemetry>(
                                    hopr_api::graph::MeasurableEdge::Probe(Ok(EdgeTransportTelemetry::Neighbor(
                                        NeighborTelemetry {
                                            peer: *opk,
                                            rtt: latency,
                                        },
                                    ))),
                                )
                            } else {
                                tracing::warn!(%pseudonym, nonce = const_hex::encode(pong), latency_ms = latency.as_millis(), "probe successful to non-offchain peer");
                            }

                            if let Some(replier) = replier {
                                replier.notify(Ok(latency));
                            }
                        } else {
                            tracing::warn!(%pseudonym, nonce = const_hex::encode(pong), possible_reasons = "[timeout, adversary]", "received pong for unknown probe");
                        }
                    }
                },
                Err(error) => tracing::error!(%pseudonym, %error, "cannot deserialize message"),
            }
        } else {
            return ProbeDispatch::Passthrough(pseudonym, in_data);
        }

        ProbeDispatch::Consumed
    }

    /// Wraps `stream` as an in-place filter: probe messages are handled internally (telemetry,
    /// pong replies via `push_to_network`), non-probe messages are yielded.
    pub fn filter_stream<T, S>(
        self,
        push_to_network: T,
        stream: S,
    ) -> impl futures::Stream<Item = (HoprPseudonym, ApplicationDataIn)>
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Unpin + Send + Sync + 'static,
        T::Error: Send,
        S: futures::Stream<Item = (HoprPseudonym, ApplicationDataIn)>,
    {
        use futures::StreamExt;
        stream.filter_map(move |(pseudonym, data)| {
            let state = self.clone();
            let push = push_to_network.clone();
            async move {
                match state.classify(push, pseudonym, data).await {
                    ProbeDispatch::Consumed => None,
                    ProbeDispatch::Passthrough(ps, d) => Some((ps, d)),
                }
            }
        })
    }
}

/// Probe functionality builder.
///
/// The builder holds information about this node's own addresses and the configuration for the probing process. It is
/// then used to construct the probing process itself.
pub struct Probe {
    /// Probe configuration.
    cfg: ProbeConfig,
    /// Tag allocator for probing telemetry tags.
    tag_allocator: Arc<dyn TagAllocator + Send + Sync>,
}

impl Probe {
    pub fn new(cfg: ProbeConfig, tag_allocator: Arc<dyn TagAllocator + Send + Sync>) -> Self {
        Self { cfg, tag_allocator }
    }

    /// The main function that assembles and starts the probing process.
    ///
    /// Returns the abortable list of background tasks (probe emission) and a
    /// [`ProbeClassifierState`] for inline classification of incoming messages.
    /// Use [`ProbeClassifierState::filter_stream`] to wrap the incoming stream.
    pub async fn continuously_scan<T, V, Tr, G>(
        self,
        api_out: T,       // lower tx channel for sending outgoing probes and pong replies
        manual_events: V, // explicit requests from the API
        probing_traffic_generator: Tr,
        network_graph: G,
    ) -> (AbortableList<HoprProbeProcess>, ProbeClassifierState<G>)
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
        T::Error: Send,
        V: futures::Stream<Item = (OffchainPublicKey, PingQueryReplier)> + Send + 'static,
        Tr: ProbingTrafficGeneration + Send + Sync + 'static,
        G: NetworkGraphView + NetworkGraphUpdate + Clone + Send + Sync + 'static,
    {
        let max_parallel_probes = self.cfg.max_parallel_probes;

        let probing_routes = probing_traffic_generator.build();

        // Currently active probes
        let network_graph_internal_neighbor = network_graph.clone();
        let network_graph_internal_path = network_graph.clone();
        let timeout = self.cfg.timeout;
        let active_neighbor_probes: moka::future::Cache<CacheNeighborKey, CacheNeighborValue> =
            moka::future::Cache::builder()
                .time_to_live(timeout)
                .max_capacity(100_000)
                .async_eviction_listener(
                    move |k: Arc<CacheNeighborKey>,
                          v: CacheNeighborValue,
                          cause|
                          -> moka::notification::ListenerFuture {
                        if matches!(cause, moka::notification::RemovalCause::Expired) {
                            // If the eviction cause is expiration => record as a failed probe
                            let store = network_graph_internal_neighbor.clone();
                            let (peer, _start, notifier) = v;

                            tracing::debug!(%peer, pseudonym = %k.0, probe = %k.1, reason = "timeout", "neighbor probe failed");
                            if let Some(replier) = notifier {
                                if matches!(peer.as_ref(), NodeId::Offchain(_)) {
                                    replier.notify(Err(()));
                                } else {
                                    tracing::warn!(
                                        reason = "non-offchain peer",
                                        "cannot notify timeout for non-offchain peer"
                                    );
                                }
                            };

                            if let NodeId::Offchain(opk) = peer.as_ref() {
                                let opk: OffchainPublicKey = *opk;
                                store
                                    .record_edge::<NeighborTelemetry, PathTelemetry>(
                                        hopr_api::graph::MeasurableEdge::Probe(Err(
                                            NetworkGraphError::ProbeNeighborTimeout(Box::new(opk)),
                                        )),
                                    );
                                futures::FutureExt::boxed(futures::future::ready(()))

                            } else {
                                futures::FutureExt::boxed(futures::future::ready(()))
                            }
                        } else {
                            // If the eviction cause is not expiration, nothing needs to be done
                            futures::FutureExt::boxed(futures::future::ready(()))
                        }
                    },
                )
                .build();

        // A *sync* Moka cache, because the tag also has to be readable and removable from a
        // non-async drop callback (see the loopback emit below). The eviction listener still has to
        // record a graph verdict, which is async, so it hands the work to a dedicated task rather
        // than blocking the cache's maintenance on the graph update.
        //
        // Only `Expired` — a probe whose packet left the node and was never answered — can record a
        // timeout; an explicit removal (a loopback that came back, or one dropped locally before
        // departure) records nothing. Because a drop can be reported later than the tag's TTL — the
        // encode stage waits up to `PACKET_ENCODING_TIMEOUT`, longer than the default probe timeout
        // — expiry only *schedules* the verdict: the task waits `LOCAL_DROP_GRACE` first, and by
        // then any drop site that saw the packet has settled the probe (see `ActivePathProbe::settled`).
        let (path_verdict_tx, path_verdict_rx) =
            futures::channel::mpsc::unbounded::<(Tag, PathTelemetry, ActivePathProbeValue)>();
        let active_path_probes: moka::sync::Cache<Tag, ActivePathProbeValue> = moka::sync::Cache::builder()
            .time_to_live(timeout)
            .max_capacity(100_000)
            .eviction_listener(
                move |tag: Arc<Tag>, probe: ActivePathProbeValue, cause: moka::notification::RemovalCause| {
                    if !matches!(cause, moka::notification::RemovalCause::Expired) {
                        return;
                    }
                    // Unbounded and only written from the eviction listener: an eviction
                    // listener must not block, and this one hand-off is its only work.
                    if path_verdict_tx
                        .unbounded_send((*tag, probe.telemetry, probe.clone()))
                        .is_err()
                    {
                        tracing::warn!(%tag, "path telemetry verdict sink is gone");
                    }
                },
            )
            .build();

        let push_to_network = api_out.clone();

        let mut processes = AbortableList::default();

        // -- Emit probes --
        let direct_neighbors =
            probing_routes
                .map(|peer| (peer, None))
                .merge(manual_events.filter_map(|(peer, notifier)| async move {
                    let routing = DestinationRouting::Forward {
                        destination: Box::new(peer.into()),
                        pseudonym: Some(HoprPseudonym::random()),
                        forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                        return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                    };
                    Some((ProbeRouting::Neighbor(routing), Some(notifier)))
                }));

        // Turns the eviction listener's expiry hand-offs into graph verdicts off the cache's
        // maintenance path. A batch waits `LOCAL_DROP_GRACE` together before recording, so a drop
        // reported after the tags' TTL can settle its probe first. Batched rather than one grace per
        // probe: expiries arrive in bulk whenever a burst of probes is dropped, and a per-probe
        // sleep would delay the N-th verdict by N graces — minutes, against the graph's 60 s
        // windows. The verdicts are independent, so their order within a batch does not matter.
        let network_graph_internal_path_verdicts = network_graph_internal_path;
        let mut path_verdict_rx = path_verdict_rx;
        processes.insert(
            HoprProbeProcess::PathVerdicts,
            hopr_utils::spawn_as_abortable_named!("probe_path_verdicts", async move {
                use futures::StreamExt;
                while let Some(first) = path_verdict_rx.next().await {
                    // Everything already queued joins this batch, at no extra grace.
                    let mut batch = vec![first];
                    while let Ok(next) = path_verdict_rx.try_recv() {
                        batch.push(next);
                    }

                    hopr_utils::runtime::prelude::sleep(LOCAL_DROP_GRACE).await;
                    for (tag, path, probe) in batch {
                        if probe.is_settled() {
                            tracing::debug!(%tag, reason = "settled before expiry was recorded", "loopback probe");
                            continue;
                        }
                        tracing::debug!(%tag, reason = "timeout", "loopback probe failed");
                        network_graph_internal_path_verdicts.record_edge::<NeighborTelemetry, PathTelemetry>(
                            hopr_api::graph::MeasurableEdge::Probe(Err(NetworkGraphError::ProbeLoopbackTimeout(path))),
                        );
                    }
                }
                tracing::warn!(
                    task = "transport (probe - path verdicts)",
                    "long-running background task finished"
                );
            }),
        );

        let tag_allocator = self.tag_allocator.clone();
        let classifier_neighbor_probes = active_neighbor_probes.clone();
        let classifier_path_probes = active_path_probes.clone();
        let emit_diag = hopr_utils::runtime::diagnostics::ConcurrentDiagnostics::new(
            "probe_emit_for_each_concurrent",
            module_path!(),
            file!(),
            line!(),
        );
        processes.insert(
            HoprProbeProcess::Emit,
            hopr_utils::spawn_as_abortable_named!("probe_emit", async move {
                direct_neighbors
                    .for_each_concurrent(max_parallel_probes, move |(peer, notifier)| {
                        let active_neighbor_probes = active_neighbor_probes.clone();
                        let active_path_probes = active_path_probes.clone();
                        let push_to_network = push_to_network.clone();
                        let tag_allocator = tag_allocator.clone();
                        let emit_diag = emit_diag.clone();

                        emit_diag.wrap(|| async move {
                            match peer {
                                ProbeRouting::Neighbor(DestinationRouting::Forward {
                                    destination,
                                    pseudonym,
                                    forward_options,
                                    return_options,
                                }) => {
                                    let nonce = NeighborProbe::random_nonce();

                                    let message = Message::Probe(nonce);

                                    if let Ok(data) = message.try_into() {
                                        let routing = DestinationRouting::Forward {
                                            destination: destination.clone(),
                                            pseudonym,
                                            forward_options,
                                            return_options,
                                        };
                                        // Neighbor probes are sent to a direct neighbor and returned via a zero-hop
                                        // return path: only 1 SURB is ever consumed. See hoprnet/hoprnet#7972.
                                        let data = ApplicationDataOut {
                                            data,
                                            packet_info: Some(OutgoingPacketInfo {
                                                max_surbs_in_packet: 1,
                                                ..Default::default()
                                            }),
                                            surb_refund: Default::default(),
                                        };
                                        let mut push_to_network = push_to_network.clone();

                                        if let Err(_error) = push_to_network.send((routing, data)).await {
                                            tracing::error!("failed to send out a ping");
                                        } else {
                                            active_neighbor_probes
                                                .insert(
                                                    (
                                                        pseudonym
                                                            .expect("the pseudonym must be present in Forward routing"),
                                                        nonce,
                                                    ),
                                                    (destination, current_time().as_unix_timestamp(), notifier),
                                                )
                                                .await;
                                        }
                                    } else {
                                        tracing::error!("failed to convert ping message into data");
                                    }
                                }
                                ProbeRouting::Neighbor(DestinationRouting::Return(_surb_matcher)) => tracing::error!(
                                    error = "logical error",
                                    "resolved transport routing is not forward"
                                ),
                                ProbeRouting::Looping((routing, path_id)) => {
                                    let message = Message::Telemetry(PathTelemetry {
                                        id: hopr_api::types::crypto_random::random_bytes(),
                                        path: std::array::from_fn(|i| path_id[i / 8].to_le_bytes()[i % 8]),
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis(),
                                    });

                                    if let Some(allocated_tag) = tag_allocator.allocate() {
                                        let tag_value = allocated_tag.value();

                                        if let Ok(packet) = hopr_protocol_app::prelude::ApplicationData::new(
                                            tag_value,
                                            message.to_bytes().as_ref(),
                                        ) {
                                            let mut push_to_network = push_to_network.clone();
                                            // the object is constructed above, so will always match
                                            let Message::Telemetry(telemetry) = message else {
                                                unreachable!("the message above is always telemetry")
                                            };

                                            // Register the tag *before* sending, not after: the packet can be
                                            // dropped downstream (routing resolution or encoding) as soon as
                                            // `send` returns, and that drop has to find the probe to settle it.
                                            //
                                            // The drop signal carries the tag and the probe's identity through the
                                            // closure armed below, so the ordering problem becomes a race between
                                            // idempotent operations on one cache key:
                                            // * a downstream drop settles and removes the probe it owns;
                                            // * a failed `send` settles and removes the very entry it inserted.
                                            // Whichever runs first takes the entry; the other finds only its own
                                            // entry (if any) and does nothing. Without the pre-insert, a drop
                                            // landing between send and insert would have found no probe at all.
                                            //
                                            // The drop may also arrive *after* the tag's TTL, in which case the
                                            // entry is already gone and the removal is no longer what silences
                                            // the timeout. That is why the release settles the shared probe state
                                            // before removing: even once the eviction listener has seen the
                                            // entry expire, `settled` keeps its deferred verdict silent.
                                            let tag = Tag::from(tag_value);
                                            let probe = Arc::new(ActivePathProbe {
                                                telemetry,
                                                allocated_tag: Arc::new(allocated_tag),
                                                settled: std::sync::atomic::AtomicBool::new(false),
                                            });
                                            let active_path_probes_for_drop = active_path_probes.clone();
                                            // The probe's telemetry id, captured for the log line: the
                                            // entry itself may already be gone when the drop lands.
                                            let on_local_drop = {
                                                let probe = probe.clone();
                                                move || {
                                                    // Settle first: a deferred verdict reads this flag.
                                                    probe.settle();
                                                    // Remove only *our own* entry, atomically. A tag is recycled
                                                    // as soon as an entry leaves the cache, so by the time this
                                                    // runs the value may belong to a later probe on the same tag —
                                                    // and that one must survive. `and_compute_with` serializes
                                                    // per key, so the compare-and-remove cannot be raced by the
                                                    // expiry-then-reallocation pair a plain get-then-remove could.
                                                    let _ = active_path_probes_for_drop
                                                        .entry_by_ref(&tag)
                                                        .and_compute_with(|current| match current {
                                                            Some(entry)
                                                                if entry.value().telemetry.id == probe.telemetry.id =>
                                                            {
                                                                moka::ops::compute::Op::Remove
                                                            }
                                                            // Gone (already expired), or a later probe's.
                                                            _ => moka::ops::compute::Op::Nop,
                                                        });
                                                    tracing::debug!(
                                                        %tag,
                                                        id = const_hex::encode(probe.telemetry.id),
                                                        reason = "dropped locally before leaving the node",
                                                        "loopback probe released"
                                                    );
                                                    // Dropping the entry's `AllocatedTag` returns the tag to the
                                                    // allocator once the cache no longer holds it.
                                                }
                                            };
                                            active_path_probes.insert(tag, probe);

                                            // Loopback telemetry probes are self-routed and never replied to via
                                            // SURB, so no SURBs should be bundled. See hoprnet/hoprnet#7972.
                                            if let Err(_error) = push_to_network
                                                .send((
                                                    routing,
                                                    ApplicationDataOut {
                                                        data: packet,
                                                        packet_info: Some(OutgoingPacketInfo {
                                                            max_surbs_in_packet: 0,
                                                            ..Default::default()
                                                        }),
                                                        surb_refund: SurbRefund::on_local_drop(on_local_drop),
                                                    },
                                                ))
                                                .await
                                            {
                                                // The packet never entered the sink, so nothing downstream can
                                                // report it: release the tag here, exactly as a downstream drop
                                                // would.
                                                if let Some(probe) = active_path_probes.remove(&tag) {
                                                    probe.settle();
                                                    tracing::debug!(
                                                        %tag,
                                                        id = const_hex::encode(probe.telemetry.id),
                                                        reason = "the outgoing sink rejected the packet",
                                                        "loopback probe released"
                                                    );
                                                }
                                                tracing::error!("failed to send out a ping");
                                            }
                                        } else {
                                            tracing::error!("failed to construct data for path telemetry")
                                        }
                                    } else {
                                        tracing::warn!("probing telemetry tag pool exhausted, skipping loopback probe");
                                    }
                                }
                            }
                        })
                    })
                    .inspect(|_| {
                        tracing::warn!(
                            task = "transport (probe - generate outgoing)",
                            "long-running background task finished"
                        )
                    })
                    .await;
            }),
        );

        let classifier = ProbeClassifierState {
            active_neighbor_probes: classifier_neighbor_probes,
            active_path_probes: classifier_path_probes,
            network_graph,
        };

        (processes, classifier)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::RwLock, time::Duration};

    use async_trait::async_trait;
    use futures::future::BoxFuture;
    use hopr_api::{
        graph::{
            EdgeLinkObservable, MeasurableEdge, NetworkGraphError,
            traits::{EdgeNetworkObservableRead, EdgeObservableRead, EdgeObservableWrite, EdgeProtocolObservable},
        },
        types::crypto::keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    };
    use hopr_protocol_app::prelude::{ApplicationData, ReservedTag, Tag};

    use super::*;
    use crate::errors::ProbeError;

    lazy_static::lazy_static!(
        static ref OFFCHAIN_KEYPAIR: OffchainKeypair = OffchainKeypair::random();
        static ref ONCHAIN_KEYPAIR: ChainKeypair = ChainKeypair::random();
        static ref NEIGHBOURS: Vec<OffchainPublicKey> = vec![
            *OffchainKeypair::random().public(),
            *OffchainKeypair::random().public(),
            *OffchainKeypair::random().public(),
            *OffchainKeypair::random().public(),
        ];
    );

    /// Test stub implementation of Observable.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct TestEdgeTransportObservations;

    impl EdgeLinkObservable for TestEdgeTransportObservations {
        fn record(&mut self, _latency: std::result::Result<Duration, ()>) {}

        fn average_latency(&self) -> Option<Duration> {
            None
        }

        // A measured, perfect link — so the probe tests see the same edge they did before the
        // trait gained the presence distinction. `has_observations` derives from this.
        fn average_probe_rate(&self) -> Option<f64> {
            Some(1.0)
        }

        fn score(&self) -> Option<f64> {
            Some(1.0)
        }
    }

    impl EdgeNetworkObservableRead for TestEdgeTransportObservations {
        fn is_connected(&self) -> Option<bool> {
            Some(true)
        }
    }

    impl EdgeProtocolObservable for TestEdgeTransportObservations {
        fn balance(&self) -> Option<hopr_api::graph::traits::Balance> {
            None
        }
    }

    impl hopr_api::graph::EdgeImmediateProtocolObservable for TestEdgeTransportObservations {
        fn ack_rate(&self) -> Option<f64> {
            None
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub struct TestEdgeObservations;

    impl EdgeObservableWrite for TestEdgeObservations {
        fn record(&mut self, _measurement: hopr_api::graph::traits::EdgeWeightType) {}
    }

    impl EdgeObservableRead for TestEdgeObservations {
        type ImmediateMeasurement = TestEdgeTransportObservations;
        type IntermediateMeasurement = TestEdgeTransportObservations;

        fn last_update(&self) -> std::time::Duration {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
        }

        fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement> {
            None
        }

        fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement> {
            None
        }

        fn score(&self) -> Option<f64> {
            Some(1.0)
        }
    }

    /// One recorded loopback verdict as the test store keeps it: whether the probe timed out, and
    /// the telemetry id that identifies the probe.
    type PathVerdict = (bool, Vec<u8>);

    #[derive(Debug, Clone)]
    pub struct PeerStore {
        me: OffchainPublicKey,
        get_peers: Arc<RwLock<VecDeque<Vec<OffchainPublicKey>>>>,
        #[allow(clippy::type_complexity)]
        on_finished: Arc<RwLock<Vec<(OffchainPublicKey, crate::errors::Result<Duration>)>>>,
        /// Loopback verdicts, kept separate from `on_finished` so a test can ask whether a path
        /// timed out without confusing it with a neighbor probe's result. Each entry is
        /// `(timed_out, telemetry id)`; only the measurable-path surface is needed, so the test
        /// store need not name the concrete path type.
        path_verdicts: Arc<RwLock<Vec<PathVerdict>>>,
    }

    impl PeerStore {
        fn new(me: OffchainPublicKey, get_peers: VecDeque<Vec<OffchainPublicKey>>) -> Self {
            Self {
                me,
                get_peers: Arc::new(RwLock::new(get_peers)),
                on_finished: Arc::new(RwLock::new(Vec::new())),
                path_verdicts: Arc::new(RwLock::new(Vec::new())),
            }
        }

        /// How many loopback probes through this store have been recorded as timeouts.
        fn path_timeouts(&self) -> usize {
            self.path_verdicts
                .read()
                .expect("should be lockable")
                .iter()
                .filter(|(timed_out, _)| *timed_out)
                .count()
        }
    }

    impl NetworkGraphUpdate for PeerStore {
        fn record_edge<N, P>(&self, telemetry: MeasurableEdge<N, P>)
        where
            N: hopr_api::graph::MeasurablePeer + Send + Clone,
            P: hopr_api::graph::MeasurablePath + Send + Clone,
        {
            let mut on_finished = self.on_finished.write().unwrap();

            // Borrow first to narrow the path arm without moving the generic telemetry; the
            // prober only produces loopback path verdicts there, and keeping them in a channel of
            // their own lets a test ask for timeouts without disturbing `on_finished`.
            let path_verdict = match &telemetry {
                MeasurableEdge::Probe(Ok(EdgeTransportTelemetry::Loopback(path))) => Some((false, path.id().to_vec())),
                MeasurableEdge::Probe(Err(NetworkGraphError::ProbeLoopbackTimeout(path))) => {
                    Some((true, path.id().to_vec()))
                }
                _ => None,
            };

            match telemetry {
                hopr_api::graph::MeasurableEdge::Probe(Ok(EdgeTransportTelemetry::Neighbor(neighbor_telemetry))) => {
                    let peer: OffchainPublicKey = *neighbor_telemetry.peer();
                    let duration = neighbor_telemetry.rtt();
                    on_finished.push((peer, Ok(duration)));
                }
                hopr_api::graph::MeasurableEdge::Probe(Err(NetworkGraphError::ProbeNeighborTimeout(peer))) => {
                    on_finished.push((
                        *peer.as_ref(),
                        Err(ProbeError::TrafficError(NetworkGraphError::ProbeNeighborTimeout(peer))),
                    ));
                }
                _ => {
                    let verdict = path_verdict.expect("only loopback or neighbor telemetry is recorded");
                    self.path_verdicts.write().unwrap().push(verdict);
                }
            }
        }

        fn record_node<N>(&self, _node: N)
        where
            N: hopr_api::graph::MeasurableNode + Send + Clone,
        {
            unimplemented!()
        }

        fn set_ticket_face_value(&self, _ticket_face_value: hopr_api::graph::traits::Balance) {}
    }

    #[async_trait]
    impl NetworkGraphView for PeerStore {
        type NodeId = OffchainPublicKey;
        type Observed = TestEdgeObservations;

        fn identity(&self) -> &OffchainPublicKey {
            &self.me
        }

        fn ticket_face_value(&self) -> Option<hopr_api::graph::traits::Balance> {
            None
        }

        fn node_count(&self) -> usize {
            self.get_peers.read().unwrap().front().map_or(0, |v| v.len())
        }

        fn contains_node(&self, _key: &OffchainPublicKey) -> bool {
            false
        }

        /// Returns a stream of all known nodes in the network graph.
        fn nodes(&self) -> futures::stream::BoxStream<'static, OffchainPublicKey> {
            let mut get_peers = self.get_peers.write().unwrap();
            Box::pin(futures::stream::iter(get_peers.pop_front().unwrap_or_default()))
        }

        fn edge(&self, _src: &OffchainPublicKey, _dest: &OffchainPublicKey) -> Option<TestEdgeObservations> {
            Some(TestEdgeObservations)
        }

        /// Probing never identifies a path by slot, so nothing here depends on this mapping.
        fn path_slot(&self, _key: &OffchainPublicKey) -> Option<u64> {
            None
        }
    }

    type TestClassifier = ProbeClassifierState<PeerStore>;

    struct TestInterface {
        probe_classifier: TestClassifier,
        from_probing_to_network_rx: futures::channel::mpsc::Receiver<(DestinationRouting, ApplicationDataOut)>,
        from_probing_to_network_tx: futures::channel::mpsc::Sender<(DestinationRouting, ApplicationDataOut)>,
        manual_probe_tx: futures::channel::mpsc::Sender<(OffchainPublicKey, PingQueryReplier)>,
    }

    async fn test_with_probing<F, Fut>(cfg: ProbeConfig, store: PeerStore, test: F) -> anyhow::Result<()>
    where
        Fut: std::future::Future<Output = anyhow::Result<()>>,
        F: Fn(TestInterface) -> Fut + Send + Sync + 'static,
    {
        let tag_allocators = hopr_transport_tag_allocator::create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (hopr_transport_tag_allocator::Usage::Session, 2048),
                (hopr_transport_tag_allocator::Usage::SessionTerminalTelemetry, 4000),
                (hopr_transport_tag_allocator::Usage::ProvingTelemetry, 10000),
            ],
        )
        .expect("tag allocators should be created");
        let probing_allocator = tag_allocators
            .into_iter()
            .find_map(|(u, alloc)| matches!(u, hopr_transport_tag_allocator::Usage::ProvingTelemetry).then_some(alloc))
            .expect("probing allocator should exist");

        let probe = Probe::new(cfg, probing_allocator);

        let (from_probing_to_network_tx, from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(DestinationRouting, ApplicationDataOut)>(100);

        let (manual_probe_tx, manual_probe_rx) =
            futures::channel::mpsc::channel::<(OffchainPublicKey, PingQueryReplier)>(100);

        let (jhs, probe_classifier) = probe
            .continuously_scan(
                from_probing_to_network_tx.clone(),
                manual_probe_rx,
                TestProbeStrategy::ImmediateNeighbor { store: store.clone() },
                store,
            )
            .await;

        let interface = TestInterface {
            probe_classifier,
            from_probing_to_network_rx,
            from_probing_to_network_tx,
            manual_probe_tx,
        };

        let result = test(interface).await;

        jhs.abort_all();

        result
    }

    const NO_PROBE_PASSES: f64 = 0.0;
    const ALL_PROBES_PASS: f64 = 1.0;

    /// Simulates the network: receives outgoing probe packets and feeds pong responses back
    /// through the classifier (mirroring what the remote peer + packet pipeline would do).
    fn concurrent_classify(
        delay: Option<std::time::Duration>,
        pass_rate: f64,
        classifier: TestClassifier,
        push_to_network: futures::channel::mpsc::Sender<(DestinationRouting, ApplicationDataOut)>,
    ) -> impl Fn((DestinationRouting, ApplicationDataOut)) -> BoxFuture<'static, ()> {
        debug_assert!(
            (NO_PROBE_PASSES..=ALL_PROBES_PASS).contains(&pass_rate),
            "Pass rate must be between {NO_PROBE_PASSES} and {ALL_PROBES_PASS}"
        );

        move |(path, data_out): (DestinationRouting, ApplicationDataOut)| -> BoxFuture<'static, ()> {
            let classifier = classifier.clone();
            let push_to_network = push_to_network.clone();

            Box::pin(async move {
                if let DestinationRouting::Forward { pseudonym, .. } = path {
                    let message: Message = data_out.data.try_into().expect("failed to convert data into message");
                    if let Message::Probe(NeighborProbe::Ping(ping)) = message {
                        let pong_message = Message::Probe(NeighborProbe::Pong(ping));

                        if let Some(delay) = delay {
                            tokio::time::sleep(delay).await;
                        }

                        if rand::random_range(NO_PROBE_PASSES..=ALL_PROBES_PASS) < pass_rate {
                            let pseudonym = pseudonym.expect("the pseudonym is always known from cache");
                            classifier
                                .classify(
                                    push_to_network,
                                    pseudonym,
                                    ApplicationDataIn {
                                        data: pong_message
                                            .try_into()
                                            .expect("failed to convert pong message into data"),
                                        packet_info: Default::default(),
                                    },
                                )
                                .await;
                        }
                    }
                };
            })
        }
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_value_for_manual_neighbor_probe() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(5),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());

        test_with_probing(cfg, store, move |iface: TestInterface| async move {
            let mut manual_probe_tx = iface.manual_probe_tx;
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let probe_classifier = iface.probe_classifier;

            let (tx, mut rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ()>>(128);
            manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;

            let _jh: hopr_utils::runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_classify(None, ALL_PROBES_PASS, probe_classifier, from_probing_to_network_tx),
                    )
                    .await;
            });

            let _duration = tokio::time::timeout(std::time::Duration::from_secs(1), rx.next())
                .await?
                .ok_or_else(|| anyhow::anyhow!("Probe did not return a result in time"))?
                .map_err(|_| anyhow::anyhow!("Probe failed"))?;

            Ok(())
        })
        .await
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_failure_on_manual_fail() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(5),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());

        test_with_probing(cfg, store, move |iface: TestInterface| async move {
            let mut manual_probe_tx = iface.manual_probe_tx;
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let probe_classifier = iface.probe_classifier;

            let (tx, mut rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ()>>(128);
            manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;

            let _jh: hopr_utils::runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_classify(None, NO_PROBE_PASSES, probe_classifier, from_probing_to_network_tx),
                    )
                    .await;
            });

            assert!(tokio::time::timeout(cfg.timeout * 2, rx.next()).await.is_err());

            Ok(())
        })
        .await
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_results_of_successful_automatically_generated_probes() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(20),
            max_parallel_probes: NEIGHBOURS.len(),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), {
            let mut neighbors = VecDeque::new();
            neighbors.push_back(NEIGHBOURS.clone());
            neighbors
        });

        test_with_probing(cfg, store.clone(), move |iface: TestInterface| async move {
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let probe_classifier = iface.probe_classifier;

            let _jh: hopr_utils::runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_classify(None, ALL_PROBES_PASS, probe_classifier, from_probing_to_network_tx),
                    )
                    .await;
            });

            // Wait for the full probe lifecycle: emit → network round trip → process.
            // Must exceed cfg.timeout (the cache TTL) to avoid probes being evicted
            // as timeouts before the pong response is processed.
            tokio::time::sleep(cfg.timeout * 3).await;

            Ok(())
        })
        .await?;

        assert_eq!(
            store
                .on_finished
                .read()
                .expect("should be lockable")
                .iter()
                .filter(|(_peer, result)| result.is_ok())
                .count(),
            NEIGHBOURS.len()
        );

        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_record_results_of_timed_out_automatically_generated_probes() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(10),
            max_parallel_probes: NEIGHBOURS.len(),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), {
            let mut neighbors = VecDeque::new();
            neighbors.push_back(NEIGHBOURS.clone());
            neighbors
        });

        let timeout = cfg.timeout * 2;

        test_with_probing(cfg, store.clone(), move |iface: TestInterface| async move {
            let from_probing_to_network_rx = iface.from_probing_to_network_rx;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let probe_classifier = iface.probe_classifier;

            let _jh: hopr_utils::runtime::prelude::JoinHandle<()> = tokio::spawn(async move {
                from_probing_to_network_rx
                    .for_each_concurrent(
                        cfg.max_parallel_probes + 1,
                        concurrent_classify(
                            Some(timeout),
                            ALL_PROBES_PASS,
                            probe_classifier,
                            from_probing_to_network_tx,
                        ),
                    )
                    .await;
            });

            // wait for the probes to start and finish
            tokio::time::sleep(timeout * 2).await;

            Ok(())
        })
        .await?;

        assert_eq!(
            store
                .on_finished
                .read()
                .expect("should be lockable")
                .iter()
                .filter(|(_peer, result)| result.is_err())
                .count(),
            NEIGHBOURS.len()
        );

        Ok(())
    }

    #[tokio::test]
    async fn probe_should_reply_with_pong_when_receiving_ping() -> anyhow::Result<()> {
        use anyhow::Context;

        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(10),
            ..Default::default()
        };

        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());

        test_with_probing(cfg, store, move |iface: TestInterface| async move {
            let probe_classifier = iface.probe_classifier;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;
            let mut from_probing_to_network_rx = iface.from_probing_to_network_rx;

            // Build a Ping message with the reserved Ping tag
            let ping = NeighborProbe::random_nonce();
            let ping_nonce = match ping {
                NeighborProbe::Ping(n) => n,
                _ => unreachable!(),
            };
            let ping_msg = Message::Probe(ping);
            let app_data: ApplicationData = ping_msg.try_into().context("converting ping to ApplicationData")?;

            // Classify the ping directly — the classifier sends the pong reply to push_to_network
            let result = probe_classifier
                .classify(
                    from_probing_to_network_tx,
                    HoprPseudonym::random(),
                    ApplicationDataIn {
                        data: app_data,
                        packet_info: Default::default(),
                    },
                )
                .await;
            anyhow::ensure!(matches!(result, ProbeDispatch::Consumed), "ping should be consumed");

            // The probe should reply with a Pong on the network channel
            let (routing, data_out) = tokio::time::timeout(Duration::from_secs(2), from_probing_to_network_rx.next())
                .await
                .context("timeout waiting for pong")?
                .context("probe should send pong reply")?;

            // Verify it's a Return routing (SURB-based reply)
            anyhow::ensure!(
                matches!(routing, DestinationRouting::Return(_)),
                "pong should use Return routing, got: {routing:?}"
            );

            // Verify the payload is a Pong with the same nonce
            let response_msg: Message = data_out.data.try_into().context("converting response to Message")?;
            anyhow::ensure!(
                matches!(response_msg, Message::Probe(NeighborProbe::Pong(n)) if n == ping_nonce),
                "response should be Pong with matching nonce"
            );

            Ok(())
        })
        .await
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn probe_should_pass_through_non_associated_tags() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), {
            let mut neighbors = VecDeque::new();
            neighbors.push_back(NEIGHBOURS.clone());
            neighbors
        });

        test_with_probing(cfg, store.clone(), move |iface: TestInterface| async move {
            let probe_classifier = iface.probe_classifier;
            let from_probing_to_network_tx = iface.from_probing_to_network_tx;

            let expected_data = ApplicationData::new(Tag::MAX, b"Hello, this is a test message!")?;

            let result = probe_classifier
                .classify(
                    from_probing_to_network_tx,
                    HoprPseudonym::random(),
                    ApplicationDataIn {
                        data: expected_data.clone(),
                        packet_info: Default::default(),
                    },
                )
                .await;

            match result {
                ProbeDispatch::Passthrough(_, actual) => assert_eq!(actual.data, expected_data),
                ProbeDispatch::Consumed => anyhow::bail!("expected Passthrough, got Consumed"),
            }

            Ok(())
        })
        .await
    }

    /// How a probe gets triggered inside the test: either via the manual ping channel
    /// (for neighbor probes) or by injecting a single [`ProbeRouting::Looping`] into the
    /// probing traffic generator (for loopback probes).
    #[derive(Clone)]
    enum TestProbeStrategy {
        ManualNeighbor,
        ImmediateNeighbor {
            store: PeerStore,
        },
        OneShotLoopback {
            routing: DestinationRouting,
            path_id: hopr_api::types::internal::routing::PathId,
        },
        /// A loopback emitted every `period`, with a distinct telemetry id each time.
        ///
        /// Needed wherever the test depends on the tag cache expiring an entry: Moka drives
        /// expiration from cache activity (there is no background upkeep thread), so a cache left
        /// without traffic never evicts.
        PeriodicLoopback {
            routing: DestinationRouting,
            path_id: hopr_api::types::internal::routing::PathId,
            period: std::time::Duration,
        },
    }

    impl hopr_api::ct::ProbingTrafficGeneration for TestProbeStrategy {
        fn build(&self) -> futures::stream::BoxStream<'static, hopr_api::ct::ProbeRouting> {
            match self {
                Self::ManualNeighbor => Box::pin(futures::stream::pending()),
                Self::ImmediateNeighbor { store } => {
                    let peers: Vec<OffchainPublicKey> =
                        store.get_peers.write().unwrap().pop_front().unwrap_or_default();
                    Box::pin(futures::StreamExt::chain(
                        futures::stream::iter(peers.into_iter().map(|peer| {
                            ProbeRouting::Neighbor(DestinationRouting::Forward {
                                destination: Box::new(peer.into()),
                                pseudonym: Some(HoprPseudonym::random()),
                                forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
                                return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
                            })
                        })),
                        futures::stream::pending(),
                    ))
                }
                Self::OneShotLoopback { routing, path_id } => {
                    let probe = hopr_api::ct::ProbeRouting::Looping((routing.clone(), *path_id));
                    Box::pin(futures::StreamExt::chain(
                        futures::stream::iter(std::iter::once(probe)),
                        futures::stream::pending(),
                    ))
                }
                Self::PeriodicLoopback {
                    routing,
                    path_id,
                    period,
                } => {
                    let routing = routing.clone();
                    let path_id = *path_id;
                    let period = *period;
                    // One probe per iteration, the next yielded only after `period`.
                    Box::pin(futures::stream::unfold((), move |()| {
                        let routing = routing.clone();
                        async move {
                            tokio::time::sleep(period).await;
                            Some((hopr_api::ct::ProbeRouting::Looping((routing, path_id)), ()))
                        }
                    }))
                }
            }
        }
    }

    /// Regression test for hoprnet/hoprnet#7972: each probe type must be emitted with the
    /// exact SURB count it actually consumes — no more, no less — so the path planner does
    /// not resolve (and the SURB store does not accumulate) unused return paths.
    ///
    /// - Neighbor probe: 1 SURB (zero-hop pong return).
    /// - Loopback probe: 0 SURBs (self-routed, never replied to).
    #[rstest::rstest]
    #[case::neighbor_probe_requests_one_surb(TestProbeStrategy::ManualNeighbor, 1)]
    #[case::loopback_probe_requests_zero_surbs(
        TestProbeStrategy::OneShotLoopback {
            routing: DestinationRouting::Forward {
                destination: Box::new((*OFFCHAIN_KEYPAIR.public()).into()),
                pseudonym: Some(HoprPseudonym::random()),
                forward_options: RoutingOptions::Hops(1.try_into().expect("1 is a valid u8")),
                return_options: None,
            },
            path_id: [1, 2, 3, 4, 5],
        },
        0,
    )]
    #[tokio::test]
    async fn probe_should_emit_with_expected_surb_count(
        #[case] strategy: TestProbeStrategy,
        #[case] expected_max_surbs: usize,
    ) -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_secs(1),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };

        // Wire up a full probing harness with the parameterized strategy injected as the
        // traffic generator. Mirrors `test_with_probing` but allows arbitrary `ProbingTrafficGeneration`.
        let tag_allocators = hopr_transport_tag_allocator::create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (hopr_transport_tag_allocator::Usage::Session, 2048),
                (hopr_transport_tag_allocator::Usage::SessionTerminalTelemetry, 4000),
                (hopr_transport_tag_allocator::Usage::ProvingTelemetry, 10000),
            ],
        )
        .expect("tag allocators should be created");
        let probing_allocator = tag_allocators
            .into_iter()
            .find_map(|(u, alloc)| matches!(u, hopr_transport_tag_allocator::Usage::ProvingTelemetry).then_some(alloc))
            .expect("probing allocator should exist");

        let probe = Probe::new(cfg, probing_allocator);

        let (from_probing_to_network_tx, mut from_probing_to_network_rx) =
            futures::channel::mpsc::channel::<(DestinationRouting, ApplicationDataOut)>(100);
        let (mut manual_probe_tx, manual_probe_rx) =
            futures::channel::mpsc::channel::<(OffchainPublicKey, PingQueryReplier)>(100);

        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());

        // Kick off the probing process before triggering — the strategy's stream drives
        // the loopback case, and we push to `manual_probe_tx` below for the neighbor case.
        let is_manual = matches!(strategy, TestProbeStrategy::ManualNeighbor);
        let (jhs, _probe_classifier) = probe
            .continuously_scan(from_probing_to_network_tx, manual_probe_rx, strategy, store)
            .await;

        if is_manual {
            let (tx, _rx) = futures::channel::mpsc::channel::<std::result::Result<Duration, ()>>(128);
            manual_probe_tx.send((NEIGHBOURS[0], PingQueryReplier::new(tx))).await?;
        }

        let (_routing, data_out) =
            tokio::time::timeout(std::time::Duration::from_secs(1), from_probing_to_network_rx.next())
                .await?
                .ok_or_else(|| anyhow::anyhow!("no probe emitted"))?;

        jhs.abort_all();

        let packet_info = data_out
            .packet_info
            .ok_or_else(|| anyhow::anyhow!("probe must carry explicit OutgoingPacketInfo"))?;
        assert_eq!(
            packet_info.max_surbs_in_packet, expected_max_surbs,
            "probe must request exactly {expected_max_surbs} SURB(s)"
        );

        Ok(())
    }

    /// The routing a single test loopback probe is sent on.
    fn loopback_routing() -> DestinationRouting {
        DestinationRouting::Forward {
            destination: Box::new((*OFFCHAIN_KEYPAIR.public()).into()),
            pseudonym: Some(HoprPseudonym::random()),
            forward_options: RoutingOptions::Hops(1.try_into().expect("1 is a valid u8")),
            return_options: None,
        }
    }

    /// A prober sized like the one in [`test_with_probing`], but with the strategy injected and the
    /// output channel handed back so a test can choose to answer, drop or ignore each probe. The
    /// classifier is returned with it so a test can deliver a returned loopback back to the prober.
    async fn loopback_prober(
        cfg: ProbeConfig,
        strategy: TestProbeStrategy,
        store: PeerStore,
    ) -> (
        hopr_utils::runtime::AbortableList<HoprProbeProcess>,
        ProbeClassifierState<PeerStore>,
        futures::channel::mpsc::Receiver<(DestinationRouting, ApplicationDataOut)>,
    ) {
        let (tx, rx) = futures::channel::mpsc::channel::<(DestinationRouting, ApplicationDataOut)>(100);
        let (jhs, classifier) = prober_with_sink(cfg, strategy, store, tx).await;
        (jhs, classifier, rx)
    }

    /// [`loopback_prober`] with the outgoing sink injected, so a test can make the send itself fail.
    async fn prober_with_sink<T>(
        cfg: ProbeConfig,
        strategy: TestProbeStrategy,
        store: PeerStore,
        out: T,
    ) -> (
        hopr_utils::runtime::AbortableList<HoprProbeProcess>,
        ProbeClassifierState<PeerStore>,
    )
    where
        T: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
        T::Error: Send,
    {
        let tag_allocators = hopr_transport_tag_allocator::create_allocators(
            ReservedTag::range().end..u16::MAX as u64 + 1,
            [
                (hopr_transport_tag_allocator::Usage::Session, 2048),
                (hopr_transport_tag_allocator::Usage::SessionTerminalTelemetry, 4000),
                (hopr_transport_tag_allocator::Usage::ProvingTelemetry, 10000),
            ],
        )
        .expect("tag allocators should be created");
        let probing_allocator = tag_allocators
            .into_iter()
            .find_map(|(u, alloc)| matches!(u, hopr_transport_tag_allocator::Usage::ProvingTelemetry).then_some(alloc))
            .expect("probing allocator should exist");
        let probe = Probe::new(cfg, probing_allocator);

        let (_manual_tx, manual_rx) = futures::channel::mpsc::channel::<(OffchainPublicKey, PingQueryReplier)>(100);

        probe.continuously_scan(out, manual_rx, strategy, store).await
    }

    /// A probe whose packet is dropped locally before it leaves the node must release its tag and
    /// record no [`NetworkGraphError::ProbeLoopbackTimeout`], so a local drop cannot lower the score
    /// of a relay that never failed.
    ///
    /// This is the in-repo half of hoprnet/hoprnet#8661: the prober registers the tag before
    /// sending, so the drop sites (`resolve_routing_stage` and the encode stage) can find it and
    /// fire the packet's local-drop signal. The prober's own `remove` is explicit, so even after the
    /// tag's TTL elapses the eviction listener records no timeout.
    #[tokio::test]
    async fn a_locally_dropped_loopback_probe_should_release_its_tag_and_record_no_timeout() -> anyhow::Result<()> {
        use anyhow::Context;

        // The tag TTL must dwarf any scheduling jitter between insert and the test's release: a
        // TTL the test occasionally overshoots would turn a correct release into an `Expired`
        // removal and fail on a loaded runner. The emission period is independent — Moka expires
        // entries only while the cache is active, so the prober, not a timer, drives maintenance.
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };
        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());
        let strategy = TestProbeStrategy::PeriodicLoopback {
            routing: loopback_routing(),
            path_id: [1, 2, 3, 4, 5],
            period: std::time::Duration::from_millis(1),
        };

        let (jhs, _classifier, mut rx) = loopback_prober(cfg, strategy, store.clone()).await;

        // One probe is emitted, then dropped locally as a downstream site would: stand in for
        // `resolve_routing_stage` / the encode stage by firing the packet's own signal.
        let (_routing, data_out) = tokio::time::timeout(std::time::Duration::from_secs(1), rx.next())
            .await
            .context("timeout waiting for the loopback probe")?
            .context("the prober must emit the loopback probe")?;

        anyhow::ensure!(
            data_out.surb_refund.has_local_drop_listener(),
            "a loopback probe must arm the local-drop signal so a drop site can release its tag"
        );
        let tag: Tag = data_out.data.application_tag;
        // The tag cache keys the verdict by the telemetry id, which is what identifies *this*
        // probe among the ones emitted after it.
        let dropped_telemetry = PathTelemetry::try_from(&data_out.data.plain_text[2..])
            .context("decoding the loopback probe's path telemetry")?;
        data_out.surb_refund.give_back();

        // Keep consuming emissions past the tag's TTL, then past the deferred-verdict grace, so the
        // cache runs maintenance and every expiry that could fire a verdict has fired.
        let _ = tokio::time::timeout(std::time::Duration::from_millis(600), async {
            while rx.next().await.is_some() {}
        })
        .await;
        jhs.abort_all();

        assert!(
            !store
                .path_verdicts
                .read()
                .expect("should be lockable")
                .iter()
                .any(|(timed_out, id)| *timed_out && *id == dropped_telemetry.id),
            "the locally dropped loopback probe (tag {tag}) must not be recorded as a relay timeout; the probes \
             emitted after it may expire, but not this one"
        );
        Ok(())
    }

    /// A drop reported *after* its tag's TTL must also be silent. The pipeline's clocks disagree:
    /// an encode drop can fire at `PACKET_ENCODING_TIMEOUT` (5 s), long after a loopback tag's TTL
    /// (`ProbeConfig::timeout`, 3 s by default), so by then the entry is gone and the removal is
    /// `Expired`. The release must still stop the verdict — this is the hole a removal-only release
    /// left open.
    #[tokio::test]
    async fn a_drop_reported_after_the_tag_ttl_should_still_record_no_timeout() -> anyhow::Result<()> {
        use anyhow::Context;

        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };
        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());
        let strategy = TestProbeStrategy::PeriodicLoopback {
            routing: loopback_routing(),
            path_id: [7, 7, 7, 7, 7],
            period: std::time::Duration::from_millis(1),
        };

        let (jhs, _classifier, mut rx) = loopback_prober(cfg, strategy, store.clone()).await;

        let (_routing, data_out) = tokio::time::timeout(std::time::Duration::from_secs(1), rx.next())
            .await
            .context("timeout waiting for the loopback probe")?
            .context("the prober must emit the loopback probe")?;
        let tag: Tag = data_out.data.application_tag;
        let dropped = PathTelemetry::try_from(&data_out.data.plain_text[2..])
            .context("decoding the loopback probe's path telemetry")?;

        // Let the tag TTL elapse, and keep the cache active so its expiry is actually processed,
        // before the drop site reports the packet as lost. That reproduces the real ordering: the
        // entry is gone by the time the drop arrives.
        let _ = tokio::time::timeout(std::time::Duration::from_millis(150), async {
            while rx.next().await.is_some() {}
        })
        .await;
        data_out.surb_refund.give_back();

        // Past the deferred-verdict grace, so any timeout queued for this probe would be recorded.
        let _ = tokio::time::timeout(std::time::Duration::from_millis(3_500), async {
            while rx.next().await.is_some() {}
        })
        .await;
        jhs.abort_all();

        assert!(
            !store
                .path_verdicts
                .read()
                .expect("should be lockable")
                .iter()
                .any(|(timed_out, id)| *timed_out && *id == dropped.id),
            "a drop reported after its TTL (tag {tag}) must still record no timeout for that probe"
        );
        Ok(())
    }

    /// A `send` that fails never lets the packet reach any drop site, so the prober releases the
    /// tag itself. Without that, the probe would sit until its TTL and blame the path for a sink
    /// failure that never involved a relay.
    #[tokio::test]
    async fn a_loopback_probe_whose_send_fails_should_record_no_timeout() -> anyhow::Result<()> {
        /// A sink that accepts nothing — `send` returns `Err` without yielding the item.
        #[derive(Clone)]
        struct FailingSink;

        impl futures::Sink<(DestinationRouting, ApplicationDataOut)> for FailingSink {
            type Error = std::io::Error;

            fn poll_ready(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn start_send(
                self: std::pin::Pin<&mut Self>,
                _item: (DestinationRouting, ApplicationDataOut),
            ) -> Result<(), Self::Error> {
                Err(std::io::Error::other("the outgoing sink is closed"))
            }

            fn poll_flush(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn poll_close(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }
        }

        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };
        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());
        let strategy = TestProbeStrategy::PeriodicLoopback {
            routing: loopback_routing(),
            path_id: [3, 3, 3, 3, 3],
            period: std::time::Duration::from_millis(1),
        };

        let (jhs, _classifier) = prober_with_sink(cfg, strategy, store.clone(), FailingSink).await;

        // Every emission fails to send and is released; well past the TTL and the verdict grace,
        // nothing may be recorded as a timeout.
        tokio::time::sleep(std::time::Duration::from_millis(3_500)).await;
        jhs.abort_all();

        assert_eq!(
            store.path_timeouts(),
            0,
            "a loopback whose send failed must be released, not left to expire as a timeout"
        );
        Ok(())
    }

    /// A loopback that returns must record a success, not a timeout: the tag is removed on the way
    /// back, before its TTL can expire. This is the peer-delivery half of the same mechanism — it
    /// pins that a probe which came home is never counted against the path.
    #[tokio::test]
    async fn a_returned_loopback_probe_should_not_be_recorded_as_a_timeout() -> anyhow::Result<()> {
        use anyhow::Context;

        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };
        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());
        let strategy = TestProbeStrategy::PeriodicLoopback {
            routing: loopback_routing(),
            path_id: [4, 4, 4, 4, 4],
            period: std::time::Duration::from_millis(1),
        };

        let (jhs, classifier, mut rx) = loopback_prober(cfg, strategy, store.clone()).await;

        // Deliver the first emission back, as the loop's last hop would.
        let (_routing, data_out) = tokio::time::timeout(std::time::Duration::from_secs(1), rx.next())
            .await
            .context("timeout waiting for the loopback probe")?
            .context("the prober must emit the loopback probe")?;
        let tag: Tag = data_out.data.application_tag;
        // The classifier's sink is only used for pong replies, which a telemetry loop never sends.
        let (sink, _sink_rx) = futures::channel::mpsc::channel(1);
        assert!(matches!(
            classifier
                .classify(
                    sink,
                    HoprPseudonym::random(),
                    ApplicationDataIn {
                        data: data_out.data.clone(),
                        packet_info: Default::default(),
                    },
                )
                .await,
            ProbeDispatch::Consumed
        ));

        let _ = tokio::time::timeout(std::time::Duration::from_millis(600), async {
            while rx.next().await.is_some() {}
        })
        .await;
        jhs.abort_all();

        // The probes emitted after it may expire; the one that came home must be a success only.
        let delivered = PathTelemetry::try_from(&data_out.data.plain_text[2..])
            .context("decoding the loopback probe's path telemetry")?;
        let verdicts = store.path_verdicts.read().expect("should be lockable");
        assert!(
            verdicts
                .iter()
                .any(|(timed_out, id)| !*timed_out && *id == delivered.id),
            "the returned loopback (tag {tag}) must be recorded as a success"
        );
        assert!(
            !verdicts.iter().any(|(timed_out, id)| *timed_out && *id == delivered.id),
            "the returned loopback (tag {tag}) must not also be recorded as a timeout"
        );
        Ok(())
    }

    /// The control for the test above: a probe whose packet genuinely left the node and was never
    /// answered still records a timeout when its tag's TTL expires.
    #[tokio::test]
    async fn an_unanswered_loopback_probe_should_record_a_timeout_when_its_tag_expires() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };
        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());
        let strategy = TestProbeStrategy::PeriodicLoopback {
            routing: loopback_routing(),
            path_id: [9, 8, 7, 6, 5],
            period: std::time::Duration::from_millis(1),
        };

        let (jhs, _classifier, mut rx) = loopback_prober(cfg, strategy, store.clone()).await;

        // Leave every emission unanswered and in flight. Consuming them is what keeps the cache
        // active, which is what drives the expiry under test; the wait covers the TTL plus the
        // deferred-verdict grace.
        let _ = tokio::time::timeout(std::time::Duration::from_millis(3_500), async {
            while rx.next().await.is_some() {}
        })
        .await;

        jhs.abort_all();

        assert!(
            store.path_timeouts() > 0,
            "an unanswered loopback probe must still be recorded as a timeout"
        );
        Ok(())
    }

    /// Many probes expiring at once must all be recorded within roughly one grace, not one grace
    /// each.
    ///
    /// The condition that produced hoprnet/hoprnet#8661 — encode timeouts dropping loopback probes
    /// in bulk — expires many tags together. A per-probe sleep in the verdict task would delay the
    /// last verdict by N graces and push it out of the graph's windows; the task therefore batches
    /// whatever is queued at once. This drives several probes to expiry inside the same window and
    /// bounds the time until the last verdict lands, which is exactly what a serialized grace would
    /// blow past.
    #[tokio::test]
    async fn a_batch_of_expired_loopback_probes_should_be_recorded_within_one_grace() -> anyhow::Result<()> {
        let cfg = ProbeConfig {
            timeout: std::time::Duration::from_millis(100),
            interval: std::time::Duration::from_secs(0),
            ..Default::default()
        };
        let store = PeerStore::new(*OFFCHAIN_KEYPAIR.public(), VecDeque::new());
        let strategy = TestProbeStrategy::PeriodicLoopback {
            routing: loopback_routing(),
            path_id: [2, 4, 6, 8, 0],
            period: std::time::Duration::from_millis(1),
        };

        let (jhs, _classifier, rx) = loopback_prober(cfg, strategy, store.clone()).await;

        // Keep the cache active past the TTL so a burst of entries expires together, then keep it
        // active through one grace so the batched verdict task can run.
        let drain = |rx: futures::channel::mpsc::Receiver<(DestinationRouting, ApplicationDataOut)>, window| async move {
            let _ = tokio::time::timeout(window, async {
                let mut rx = rx;
                while rx.next().await.is_some() {}
            })
            .await;
        };
        let started = std::time::Instant::now();
        drain(rx, std::time::Duration::from_millis(200)).await;
        let expired_at = started.elapsed();

        // All of them fall due at essentially the same moment; querying after one grace (+ slack)
        // must already see the whole batch.
        tokio::time::sleep(LOCAL_DROP_GRACE + std::time::Duration::from_millis(400)).await;
        jhs.abort_all();

        let recorded = store.path_timeouts();
        let elapsed_after_grace = started.elapsed();
        assert!(
            recorded >= 2,
            "the batch must contain several expired probes, saw {recorded} (expiry took {expired_at:?})"
        );
        // A serialized grace would need `recorded` graces; allow one grace plus generous slack for
        // the TTL drain and the verdict task.
        let per_probe_grace_total = LOCAL_DROP_GRACE * recorded as u32;
        assert!(
            elapsed_after_grace < per_probe_grace_total,
            "the batch took {elapsed_after_grace:?} for {recorded} verdicts, which is one grace each \
             ({per_probe_grace_total:?}) rather than a single batched grace"
        );
        Ok(())
    }
}
