use std::{sync::Arc, time::Duration};

use anyhow::Context;
use futures::future::join_all;
use hopr_crypto_types::prelude::*;
use hopr_db_node::HoprNodeDb;
use hopr_primitive_types::prelude::*;
use hopr_transport::Hash;
use tokio::time::sleep;

use crate::{Address, ChannelEntry, ChannelStatus, Hopr, PeerId, prelude, testing::TestingConnector};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeSafeConfig {
    pub safe_address: Address,
    pub module_address: Address,
}

pub struct TestedHopr {
    pub instance: Arc<Hopr<TestingConnector, HoprNodeDb>>,
}

impl TestedHopr {
    pub async fn new(
        chain_key: ChainKeypair,
        offchain_key: OffchainKeypair,
        host_port: u16,
        node_db: HoprNodeDb,
        connector: TestingConnector,
        safe: NodeSafeConfig,
        winn_prob: Option<f64>,
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
                protocol: hopr_transport::config::ProtocolConfig {
                    outgoing_ticket_winning_prob: winn_prob,
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

    pub fn inner(&self) -> &Hopr<TestingConnector, HoprNodeDb> {
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
pub struct ChannelGuard {
    pub channels: Vec<(Arc<Hopr<TestingConnector, HoprNodeDb>>, Hash)>,
}

impl ChannelGuard {
    pub fn channel_id(&self, index: usize) -> &Hash {
        &self.channels[index].1
    }

    pub async fn try_open_channels_for_path<I, T>(path: I, funding: HoprBalance) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<Arc<Hopr<TestingConnector, HoprNodeDb>>>,
    {
        let mut channels = vec![];

        let path: Vec<Arc<Hopr<TestingConnector, HoprNodeDb>>> = path.into_iter().map(|item| item.into()).collect();
        let path_len = path.len();

        // no need for a channel to the last node from penultimate
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

    pub async fn try_to_get_all_ticket_counts(&self) -> anyhow::Result<Vec<usize>> {
        let futures = self.channels.iter().map(|(hopr, channel_id)| async move {
            hopr.tickets_in_channel(&channel_id)
                .await
                .context("getting ticket statistics must succeed")
                .into_iter()
                .count()
        });

        let stats = join_all(futures).await;
        Ok(stats)
    }

    pub async fn try_close_channels_all_channels(&self) -> anyhow::Result<()> {
        let futures = self.channels.iter().map(|(hopr, channel_id)| {
            let hopr = hopr.clone();
            let channel_id = channel_id.clone();
            async move {
                hopr.close_channel_by_id(&channel_id)
                    .await
                    .context("closing channel must succeed")
            }
        });

        join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()?;

        sleep(Duration::from_secs(2)).await;

        let futures = self.channels.iter().map(|(hopr, channel_id)| {
            let hopr = hopr.clone();
            let channel_id = channel_id.clone();
            async move {
                hopr.close_channel_by_id(&channel_id)
                    .await
                    .context("closing channel must succeed")
            }
        });

        join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }
}

impl Drop for ChannelGuard {
    fn drop(&mut self) {
        let channels = self.channels.clone();
        tokio::spawn(async move {
            for (hopr, channel_id) in channels {
                let _ = hopr.close_channel_by_id(&channel_id).await;
            }
        });
    }
}
