use std::sync::Arc;

use async_lock::RwLock;
use futures::{TryStreamExt, stream::FuturesUnordered};
use hopr_api::{
    chain::ChainKeyOperations,
    db::{FoundSurb, HoprDbProtocolOperations},
};
use hopr_chain_types::chain_events::NetworkRegistryStatus;
use hopr_crypto_packet::prelude::*;
use hopr_crypto_types::{crypto_traits::Randomizable, prelude::OffchainPublicKey};
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use hopr_path::{ChainPath, PathAddressResolver, ValidatedPath, errors::PathError, selectors::PathSelector};
use hopr_primitive_types::prelude::*;
use tracing::trace;

use crate::errors::HoprTransportError;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PATH_LENGTH: hopr_metrics::SimpleHistogram = hopr_metrics::SimpleHistogram::new(
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
pub(crate) struct PathPlanner<Db, R, S> {
    db: Db,
    resolver: R,
    channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
    selector: S,
    me: Address,
}

struct ChainPathResolver<'c, R>(&'c R);

#[async_trait::async_trait]
impl<'c, R: ChainKeyOperations + Sync> PathAddressResolver for ChainPathResolver<'c, R> {
    async fn resolve_transport_address(&self, address: &Address) -> Result<Option<OffchainPublicKey>, PathError> {
        self.0
            .chain_key_to_packet_key(address)
            .await
            .map_err(|e| PathError::UnknownPeer(format!("{address}: {e}")))
    }

    async fn resolve_chain_address(&self, key: &OffchainPublicKey) -> Result<Option<Address>, PathError> {
        self.0
            .packet_key_to_chain_key(key)
            .await
            .map_err(|e| PathError::UnknownPeer(format!("{key}: {e}")))
    }
}

impl<Db, R, S> PathPlanner<Db, R, S>
where
    Db: HoprDbProtocolOperations + Send + Sync + 'static,
    R: ChainKeyOperations + Send + Sync + 'static,
    S: PathSelector + Send + Sync,
{
    pub(crate) fn new(
        me: Address,
        db: Db,
        resolver: R,
        selector: S,
        channel_graph: Arc<RwLock<hopr_path::channel_graph::ChannelGraph>>,
    ) -> Self {
        Self {
            db,
            resolver,
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
                    &ChainPathResolver(&self.resolver),
                )
                .await?
            }
            RoutingOptions::Hops(hops) if u32::from(hops) == 0 => {
                trace!(hops = 0, "resolving zero-hop path");

                ValidatedPath::new(
                    source,
                    ChainPath::direct(destination),
                    &cg,
                    &ChainPathResolver(&self.resolver),
                )
                .await?
            }
            RoutingOptions::Hops(hops) => {
                trace!(%hops, "resolving path using hop count");

                let cp = self
                    .selector
                    .select_path(source, destination, hops.into(), hops.into())
                    .await?;

                ValidatedPath::new(
                    source,
                    ChainPath::from_channel_path(cp, destination),
                    &cg,
                    &ChainPathResolver(&self.resolver),
                )
                .await?
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
        max_surbs: usize,
        routing: DestinationRouting,
    ) -> crate::errors::Result<(ResolvedTransportRouting, Option<usize>)> {
        match routing {
            DestinationRouting::Forward {
                destination,
                pseudonym,
                forward_options,
                return_options,
            } => {
                let forward_path = self.resolve_path(self.me, destination, forward_options).await?;

                let return_paths = if let Some(return_options) = return_options {
                    // Safeguard for the correct number of SURBs
                    let num_possible_surbs = HoprPacket::max_surbs_with_message(size_hint).min(max_surbs);
                    trace!(%destination, %num_possible_surbs, data_len = size_hint, max_surbs, "resolving packet return paths");

                    (0..num_possible_surbs)
                        .map(|_| self.resolve_path(destination, self.me, return_options.clone()))
                        .collect::<FuturesUnordered<_>>()
                        .try_collect::<Vec<ValidatedPath>>()
                        .await?
                } else {
                    vec![]
                };

                trace!(%destination, num_surbs = return_paths.len(), data_len = size_hint, "resolved packet");

                Ok((
                    ResolvedTransportRouting::Forward {
                        pseudonym: pseudonym.unwrap_or_else(HoprPseudonym::random),
                        forward_path,
                        return_paths,
                    },
                    None,
                ))
            }
            DestinationRouting::Return(matcher) => {
                let FoundSurb {
                    sender_id,
                    surb,
                    remaining,
                } = self
                    .db
                    .find_surb(matcher)
                    .await
                    .map_err(|e| HoprTransportError::Other(e.into()))?;
                Ok((ResolvedTransportRouting::Return(sender_id, surb), Some(remaining)))
            }
        }
    }
}
