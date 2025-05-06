use async_lock::RwLock;
use futures::channel::mpsc::Sender;
use futures::stream::FuturesUnordered;
use futures::TryStreamExt;
use std::sync::{Arc, OnceLock};
use tracing::trace;

use hopr_chain_types::chain_events::NetworkRegistryStatus;
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_crypto_types::crypto_traits::Randomizable;
use hopr_db_sql::HoprDbAllOperations;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::{ResolvedTransportRouting, RoutingOptions};
use hopr_network_types::types::DestinationRouting;
use hopr_path::{selectors::PathSelector, ChainPath, PathAddressResolver, ValidatedPath};
use hopr_primitive_types::primitives::Address;
use hopr_transport_protocol::processor::{MsgSender, SendMsgInput};
use hopr_transport_session::{
    errors::{SessionManagerError, TransportSessionError},
    traits::SendMsg,
};

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
    pub unredeemed_value: hopr_primitive_types::primitives::Balance,
    pub redeemed_value: hopr_primitive_types::primitives::Balance,
    pub neglected_value: hopr_primitive_types::primitives::Balance,
    pub rejected_value: hopr_primitive_types::primitives::Balance,
}

#[derive(Clone)]
pub(crate) struct PathPlanner<T, S> {
    db: T,
    channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
    selector: S,
    me: Address,
}

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
        let cg = self.channel_graph.read().await;
        let path = match options {
            RoutingOptions::IntermediatePath(path) => {
                trace!(?path, "resolving a specific path");

                ValidatedPath::new(
                    ChainPath::new(path.into_iter().chain(std::iter::once(destination)))?,
                    &cg,
                    &self.db,
                )
                .await?
            }
            RoutingOptions::Hops(hops) if u32::from(hops) == 0 => {
                trace!(hops = 0, "resolving zero-hop path");

                ValidatedPath::new(ChainPath::direct(destination), &cg, &self.db).await?
            }
            RoutingOptions::Hops(hops) => {
                trace!(%hops, "resolving path using hop count");

                let cp = self
                    .selector
                    .select_path(source, destination, hops.into(), hops.into())
                    .await?;

                ValidatedPath::new(ChainPath::from_channel_path(cp, destination), &cg, &self.db).await?
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

#[derive(Clone)]
pub(crate) struct MessageSender<T, S> {
    pub process_packet_send: Arc<OnceLock<MsgSender<Sender<SendMsgInput>>>>,
    pub resolver: PathPlanner<T, S>,
}

impl<T, S> MessageSender<T, S>
where
    T: HoprDbAllOperations + PathAddressResolver + std::fmt::Debug + Send + Sync + 'static,
    S: PathSelector + Send + Sync,
{
    pub fn new(
        process_packet_send: Arc<OnceLock<MsgSender<Sender<SendMsgInput>>>>,
        resolver: PathPlanner<T, S>,
    ) -> Self {
        Self {
            process_packet_send,
            resolver,
        }
    }
}

#[async_trait::async_trait]
impl<T, S> SendMsg for MessageSender<T, S>
where
    T: HoprDbAllOperations + PathAddressResolver + std::fmt::Debug + Send + Sync + 'static,
    S: PathSelector + Send + Sync,
{
    #[tracing::instrument(level = "debug", skip(self, data))]
    async fn send_message(
        &self,
        data: ApplicationData,
        routing: DestinationRouting,
    ) -> std::result::Result<(), TransportSessionError> {
        let routing = self
            .resolver
            .resolve_routing(data.len(), routing)
            .await
            .map_err(|err| {
                tracing::error!(%err, "failed to resolve routing");
                TransportSessionError::Path
            })?;

        self.process_packet_send
            .get()
            .ok_or_else(|| SessionManagerError::NotStarted)?
            .send_packet(data, routing)
            .await
            .map_err(|_| TransportSessionError::Closed)?
            .consume_and_wait(crate::constants::PACKET_QUEUE_TIMEOUT_MILLISECONDS)
            .await
            .map_err(|_e| TransportSessionError::Timeout)?;

        trace!("packet sent to the outgoing queue");

        Ok(())
    }
}
