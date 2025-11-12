use std::{sync::Arc, time::Duration};

use anyhow::Context;
use hopr_api::{
    chain::{HoprBalance, HoprChainApi},
    db::HoprNodeDbApi,
};
use hopr_crypto_types::prelude::*;
use hopr_transport::Hash;

use crate::{Address, ChannelEntry, ChannelStatus, Hopr, PeerId, prelude};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeSafeConfig {
    pub safe_address: Address,
    pub module_address: Address,
}

pub struct TestedHopr<C, Db> {
    pub instance: Arc<Hopr<C, Db>>,
}

impl<C, Db> TestedHopr<C, Db>
where
    C: HoprChainApi + Clone + Send + Sync + 'static,
    Db: HoprNodeDbApi + Clone + Send + Sync + 'static,
{
    pub async fn new(
        chain_key: ChainKeypair,
        offchain_key: OffchainKeypair,
        host_port: u16,
        node_db: Db,
        connector: C,
        safe: NodeSafeConfig,
    ) -> Self {
        let instance = Hopr::new(
            crate::config::HoprLibConfig {
                probe: crate::config::ProbeConfig {
                    timeout: Duration::from_secs(2),
                    max_parallel_probes: 10,
                    recheck_threshold: Duration::from_secs(1),
                    ..Default::default()
                },
                network_options: crate::config::NetworkConfig {
                    ignore_timeframe: Duration::from_secs(0),
                    ..Default::default()
                },
                host: crate::config::HostConfig {
                    address: crate::config::HostType::default(),
                    port: host_port,
                },
                safe_module: crate::config::SafeModule {
                    safe_address: safe.safe_address,
                    module_address: safe.module_address,
                    ..Default::default()
                },
                transport: crate::config::TransportConfig {
                    prefer_local_addresses: true,
                    announce_local_addresses: true,
                },
                session: hopr_transport::config::SessionGlobalConfig {
                    idle_timeout: Duration::from_millis(2500),
                    ..Default::default()
                },
                ..Default::default()
            },
            connector,
            node_db,
            &offchain_key,
            &chain_key,
        )
        .await
        .expect(format!("failed to create hopr instance on port {host_port}").as_str());

        Self {
            instance: std::sync::Arc::new(instance),
        }
    }

    pub fn inner(&self) -> &Hopr<C, Db> {
        &self.instance
    }

    pub fn address(&self) -> Address {
        self.instance.me_onchain()
    }

    pub fn peer_id(&self) -> PeerId {
        self.instance.me_peer_id()
    }

    pub async fn channel_from_hash(&self, channel_hash: &prelude::Hash) -> Option<ChannelEntry> {
        self.instance
            .channel_from_hash(channel_hash)
            .await
            .unwrap_or_else(|_| None)
    }

    pub async fn outgoing_channels_by_status(&self, status: ChannelStatus) -> Option<Vec<ChannelEntry>> {
        match self.instance.channels_from(&self.address()).await {
            Ok(channels) => Some(channels.iter().filter(|c| c.status == status).cloned().collect()),
            Err(_) => None,
        }
    }
}

/// Guard for opening and closing the channels in a HOPR network.
///
/// Cleans up the opened channels on drop.
pub struct ChannelGuard<C, Db> {
    /// Prepared for the implementation of Drop and closing
    #[allow(dead_code)]
    channels: Vec<(Arc<Hopr<C, Db>>, Hash)>,
}

impl<C, Db> ChannelGuard<C, Db>
where
    C: HoprChainApi + Clone + Send + Sync + 'static,
    Db: HoprNodeDbApi + Clone + Send + Sync + 'static,
{
    #[must_use]
    pub async fn try_open_channels_for_path<I, T>(path: I, funding: HoprBalance) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<Arc<Hopr<C, Db>>>,
    {
        let mut channels = vec![];

        let path: Vec<Arc<Hopr<C, Db>>> = path.into_iter().map(|item| item.into()).collect();
        let path_len = path.len();

        // no need for a channel to the last node from previous to last
        for window in path.into_iter().take(path_len - 1).collect::<Vec<_>>().windows(2) {
            let src = &window[0];
            let dst = &window[1];

            let channel = src
                .open_channel(&dst.me_onchain(), funding)
                .await
                .context("opening channel must succeed")?;

            channels.push((src.clone(), channel.channel_id));
        }

        Ok(Self { channels })
    }
}
