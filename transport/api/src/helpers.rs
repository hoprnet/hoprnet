use futures::{TryStreamExt, stream::FuturesUnordered};
use hopr_api::chain::{ChainKeyOperations, ChainPathResolver, ChainReadChannelOperations};
use hopr_crypto_packet::prelude::*;
use hopr_crypto_types::crypto_traits::Randomizable;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use hopr_primitive_types::prelude::*;
use hopr_protocol_hopr::{FoundSurb, SurbStore};
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
pub(crate) struct PathPlanner<Surb, R, S> {
    pub(crate) surb_store: Surb,
    resolver: R,
    selector: S,
    me: Address,
}

impl<Surb, R, S> PathPlanner<Surb, R, S>
where
    Surb: SurbStore + Send + Sync + 'static,
    R: ChainKeyOperations + ChainReadChannelOperations + Send + Sync + 'static,
    S: PathSelector + Send + Sync,
{
    pub(crate) fn new(me: Address, surb_store: Surb, resolver: R, selector: S) -> Self {
        Self {
            surb_store,
            resolver,
            selector,
            me,
        }
    }

    async fn resolve_node_id_to_addr(&self, node_id: &NodeId) -> crate::errors::Result<Address> {
        match node_id {
            NodeId::Chain(addr) => Ok(*addr),
            NodeId::Offchain(key) => self
                .resolver
                .packet_key_to_chain_key(key)
                .await
                .map_err(|e| {
                    HoprTransportError::Other(anyhow::anyhow!("failed to resolve offchain key to chain key: {e}"))
                })?
                .ok_or(HoprTransportError::Other(anyhow::anyhow!(
                    "failed to resolve offchain key to chain key: no chain key found"
                ))),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn resolve_path(
        &self,
        source: NodeId,
        destination: NodeId,
        options: RoutingOptions,
    ) -> crate::errors::Result<ValidatedPath> {
        let resolver = ChainPathResolver::from(&self.resolver);
        let path = match options {
            RoutingOptions::IntermediatePath(path) => {
                trace!(?path, "resolving a specific path");

                ValidatedPath::new(
                    source,
                    path.into_iter().chain(std::iter::once(destination)).collect::<Vec<_>>(),
                    &resolver,
                )
                .await?
            }
            RoutingOptions::Hops(hops) if u32::from(hops) == 0 => {
                trace!(hops = 0, "resolving zero-hop path");

                ValidatedPath::new(source, vec![destination], &resolver).await?
            }
            RoutingOptions::Hops(hops) => {
                trace!(%hops, "resolving path using hop count");

                let dst = self.resolve_node_id_to_addr(&destination).await?;

                let cp = self
                    .selector
                    .select_path(
                        self.resolve_node_id_to_addr(&source).await?,
                        dst,
                        hops.into(),
                        hops.into(),
                    )
                    .await?;

                ValidatedPath::new(source, ChainPath::from_channel_path(cp, dst), &resolver).await?
            }
        };

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            use hopr_internal_types::path::Path;
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
                let forward_path = self.resolve_path(self.me.into(), *destination, forward_options).await?;

                let return_paths = if let Some(return_options) = return_options {
                    // Safeguard for the correct number of SURBs
                    let num_possible_surbs = HoprPacket::max_surbs_with_message(size_hint).min(max_surbs);
                    trace!(%destination, %num_possible_surbs, data_len = size_hint, max_surbs, "resolving packet return paths");

                    (0..num_possible_surbs)
                        .map(|_| self.resolve_path(*destination, self.me.into(), return_options.clone()))
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
                    .surb_store
                    .find_surb(matcher)
                    .await
                    .ok_or(HoprTransportError::Api("no surb".into()))?;
                Ok((ResolvedTransportRouting::Return(sender_id, surb), Some(remaining)))
            }
        }
    }
}
