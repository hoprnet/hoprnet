use async_lock::RwLock;
use futures::channel::mpsc::Sender;
use std::sync::{Arc, OnceLock};
use tracing::trace;

use chain_types::chain_events::NetworkRegistryStatus;
use core_path::{
    path::TransportPath,
    selectors::dfs::{DfsPathSelector, DfsPathSelectorConfig, RandomizedEdgeWeighting},
    selectors::PathSelector,
};
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_sql::HoprDbAllOperations;
use hopr_internal_types::protocol::ApplicationData;
use hopr_network_types::prelude::RoutingOptions;
use hopr_primitive_types::primitives::Address;
use hopr_transport_identity::PeerId;
use hopr_transport_protocol::msg::processor::{MsgSender, SendMsgInput};
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

use crate::{constants::RESERVED_SESSION_TAG_UPPER_LIMIT, errors::HoprTransportError};

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
pub(crate) struct PathPlanner<T> {
    db: T,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    selector: DfsPathSelector<RandomizedEdgeWeighting>,
}

impl<T> PathPlanner<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Send + Sync + 'static,
{
    pub(crate) fn new(
        db: T,
        path_selector_cfg: DfsPathSelectorConfig,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    ) -> Self {
        Self {
            db,
            channel_graph,
            selector: DfsPathSelector::new(path_selector_cfg),
        }
    }

    pub(crate) fn channel_graph(&self) -> Arc<RwLock<core_path::channel_graph::ChannelGraph>> {
        self.channel_graph.clone()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) async fn resolve_path(
        &self,
        destination: PeerId,
        options: RoutingOptions,
    ) -> crate::errors::Result<TransportPath> {
        let path = match options {
            RoutingOptions::IntermediatePath(path) => {
                let complete_path = Vec::from_iter(path.into_iter().chain([destination]));
                trace!(full_path = ?complete_path, "resolved a specific path");

                let cg = self.channel_graph.read().await;

                TransportPath::resolve(complete_path, &self.db, &cg)
                    .await
                    .map(|(p, _)| p)?
            }
            RoutingOptions::Hops(hops) if u32::from(hops) == 0 => {
                trace!(hops = 0, %destination, "resolved zero-hop path");
                TransportPath::direct(destination)
            }
            RoutingOptions::Hops(hops) => {
                trace!(%hops, "resolved path using hop count");

                let pk = OffchainPublicKey::try_from(destination)?;

                if let Some(chain_key) = self
                    .db
                    .translate_key(None, pk)
                    .await
                    .map_err(hopr_db_sql::api::errors::DbError::from)?
                {
                    let target_chain_key: Address = chain_key.try_into()?;
                    let cp = {
                        let cg = self.channel_graph.read().await;
                        self.selector
                            .select_path(&cg, cg.my_address(), target_chain_key, hops.into(), hops.into())?
                    };

                    let full_path = cp.into_path(&self.db, target_chain_key).await?;
                    trace!(%full_path, "resolved automatic path");

                    full_path
                } else {
                    return Err(HoprTransportError::Api(
                        "send msg: unknown destination peer id encountered".to_owned(),
                    ));
                }
            }
        };

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            use core_path::path::Path;
            hopr_metrics::SimpleHistogram::observe(&METRIC_PATH_LENGTH, (path.hops().len() - 1) as f64);
        }

        Ok(path)
    }
}

#[derive(Clone)]
pub(crate) struct MessageSender<T> {
    pub process_packet_send: Arc<OnceLock<MsgSender<Sender<SendMsgInput>>>>,
    pub resolver: PathPlanner<T>,
}

impl<T> MessageSender<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Send + Sync + 'static,
{
    pub fn new(process_packet_send: Arc<OnceLock<MsgSender<Sender<SendMsgInput>>>>, resolver: PathPlanner<T>) -> Self {
        Self {
            process_packet_send,
            resolver,
        }
    }
}

#[async_trait::async_trait]
impl<T> SendMsg for MessageSender<T>
where
    T: HoprDbAllOperations + std::fmt::Debug + Send + Sync + 'static,
{
    #[tracing::instrument(level = "debug", skip(self, data))]
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: PeerId,
        options: RoutingOptions,
    ) -> std::result::Result<(), TransportSessionError> {
        data.application_tag
            .is_some_and(|application_tag| application_tag < RESERVED_SESSION_TAG_UPPER_LIMIT)
            .then_some(())
            .ok_or(TransportSessionError::Tag)?;

        let path = self
            .resolver
            .resolve_path(destination, options)
            .await
            .map_err(|_| TransportSessionError::Path)?;

        self.process_packet_send
            .get()
            .ok_or_else(|| SessionManagerError::NotStarted)?
            .send_packet(data, path)
            .await
            .map_err(|_| TransportSessionError::Closed)?
            .consume_and_wait(crate::constants::PACKET_QUEUE_TIMEOUT_MILLISECONDS)
            .await
            .map_err(|_e| TransportSessionError::Timeout)?;

        trace!("Packet sent to the outgoing queue");

        Ok(())
    }
}
