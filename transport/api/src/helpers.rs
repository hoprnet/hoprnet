use std::sync::Arc;

use async_lock::RwLock;
use futures::{StreamExt, TryStreamExt, channel::mpsc::Sender, stream::FuturesUnordered};
use hopr_chain_types::chain_events::NetworkRegistryStatus;
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_crypto_types::crypto_traits::Randomizable;
use hopr_db_sql::HoprDbAllOperations;
use hopr_internal_types::prelude::*;
use hopr_network_types::{
    prelude::{ResolvedTransportRouting, RoutingOptions},
    types::DestinationRouting,
};
use hopr_path::{ChainPath, PathAddressResolver, ValidatedPath, selectors::PathSelector};
use hopr_primitive_types::{prelude::HoprBalance, primitives::Address};
use hopr_protocol_app::v1::ApplicationData;
use hopr_transport_protocol::processor::{MsgSender, SendMsgInput};
use tracing::trace;

use crate::constants::MAXIMUM_MSG_OUTGOING_BUFFER_SIZE;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PATH_LENGTH: hopr_metrics::metrics::SimpleHistogram = hopr_metrics::metrics::SimpleHistogram::new(
        "hopr_path_length",
        "Distribution of number of hops of sent messages",
        vec![0.0, 1.0, 2.0, 3.0, 4.0]
    ).unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PeerEligibility {
    Eligible,
    Ineligible,
}

impl From<NetworkRegistryStatus> for PeerEligibility {
    fn from(value: NetworkRegistryStatus) -> Self {
        match value {
            NetworkRegistryStatus::Allowed => Self::Eligible,
            NetworkRegistryStatus::Denied => Self::Ineligible,
        }
    }
}

/// Ticket statistics data exposed by the ticket mechanism.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TicketStatistics {
    pub winning_count: u128,
    pub unredeemed_value: HoprBalance,
    pub redeemed_value: HoprBalance,
    pub neglected_value: HoprBalance,
    pub rejected_value: HoprBalance,
}

#[derive(Clone)]
pub(crate) struct PathPlanner<T, S> {
    db: T,
    channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
    selector: S,
    me: Address,
}

const DEFAULT_PACKET_PLANNER_CONCURRENCY: usize = 10;

impl<T, S> PathPlanner<T, S>
where
    T: HoprDbAllOperations + PathAddressResolver + std::fmt::Debug + Send + Sync + 'static,
    S: PathSelector + Send + Sync,
{
    pub(crate) fn new(
        me: Address,
        db: T,
        selector: S,
        channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
    ) -> Self {
        Self {
            db,
            channel_graph,
            selector,
            me,
        }
    }

    pub(crate) fn channel_graph(&self) -> Arc<RwLock<hopr_path::channel_graph::ChannelGraph>> {
        self.channel_graph.clone()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn resolve_path(
        &self,
        source: Address,
        destination: Address,
        options: RoutingOptions,
    ) -> crate::errors::Result<ValidatedPath> {
        let cg = self.channel_graph.read_arc().await;
        let path = match options {
            RoutingOptions::IntermediatePath(path) => {
                trace!(?path, "resolving a specific path");

                ValidatedPath::new(
                    source,
                    ChainPath::new(path.into_iter().chain(std::iter::once(destination)))?,
                    &cg,
                    &self.db,
                )
                .await?
            }
            RoutingOptions::Hops(hops) if u32::from(hops) == 0 => {
                trace!(hops = 0, "resolving zero-hop path");

                ValidatedPath::new(source, ChainPath::direct(destination), &cg, &self.db).await?
            }
            RoutingOptions::Hops(hops) => {
                trace!(%hops, "resolving path using hop count");

                let cp = self
                    .selector
                    .select_path(source, destination, hops.into(), hops.into())
                    .await?;

                ValidatedPath::new(source, ChainPath::from_channel_path(cp, destination), &cg, &self.db).await?
            }
        };

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            use hopr_path::Path;
            hopr_metrics::SimpleHistogram::observe(&METRIC_PATH_LENGTH, (path.num_hops() - 1) as f64);
        }

        trace!(%path, "validated resolved path");

        Ok(path)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) async fn resolve_routing(
        &self,
        size_hint: usize,
        routing: DestinationRouting,
    ) -> crate::errors::Result<ResolvedTransportRouting> {
        match routing {
            DestinationRouting::Forward {
                destination,
                pseudonym,
                forward_options,
                return_options,
            } => {
                let forward_path = self.resolve_path(self.me, destination, forward_options).await?;

                let return_paths = if let Some(return_options) = return_options {
                    let num_possible_surbs = HoprPacket::max_surbs_with_message(size_hint);
                    trace!(%destination, %num_possible_surbs, data_len = size_hint, "resolving packet return paths");

                    (0..num_possible_surbs)
                        .map(|_| self.resolve_path(destination, self.me, return_options.clone()))
                        .collect::<FuturesUnordered<_>>()
                        .try_collect::<Vec<ValidatedPath>>()
                        .await?
                } else {
                    vec![]
                };

                trace!(%destination, num_surbs = return_paths.len(), data_len = size_hint, "resolved packet");

                Ok(ResolvedTransportRouting::Forward {
                    pseudonym: pseudonym.unwrap_or_else(HoprPseudonym::random),
                    forward_path,
                    return_paths,
                })
            }
            DestinationRouting::Return(matcher) => {
                let (sender_id, surb) = self.db.find_surb(matcher).await?;
                Ok(ResolvedTransportRouting::Return(sender_id, surb))
            }
        }
    }
}

// TODO: consider making this a `with` decorator for `MsgSender`
// ^^ This requires
//   a) `MsgSender` to be a `Sink`
//   b) Dropping the `Clone` requirement on `Sink` that is given into `SessionManager`
// However the DestinationRouting resolution concurrency (from for_each_concurrent) would be lost.
pub(crate) fn run_packet_planner<T, S>(
    planner: PathPlanner<T, S>,
    packet_sender: MsgSender<Sender<SendMsgInput>>,
) -> Sender<(DestinationRouting, ApplicationData)>
where
    T: HoprDbAllOperations + PathAddressResolver + std::fmt::Debug + Clone + Send + Sync + 'static,
    S: PathSelector + Clone + Send + Sync + 'static,
{
    let (tx, rx) =
        futures::channel::mpsc::channel::<(DestinationRouting, ApplicationData)>(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE);

    let planner_concurrency = std::env::var("HOPR_PACKET_PLANNER_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PACKET_PLANNER_CONCURRENCY);

    hopr_async_runtime::prelude::spawn(rx.for_each_concurrent(planner_concurrency, move |(routing, data)| {
        let planner = planner.clone();
        let packet_sender = packet_sender.clone();
        async move {
            match planner.resolve_routing(data.len(), routing).await {
                Ok(resolved) => {
                    // The awaiter here is intentionally dropped,
                    // since we do not intend to be notified about packet delivery to the first hop
                    if let Err(error) = packet_sender.send_packet(data, resolved).await {
                        tracing::error!(%error, "failed to enqueue packet for sending");
                    }
                }
                Err(error) => tracing::error!(%error, "failed to resolve path for routing"),
            }
        }
    }));

    tx
}
