use std::{fmt::Formatter, sync::Arc, time::Duration};

use anyhow::Context;
use futures::future::join_all;
use hopr_lib::{
    Address, ChannelEntry, ChannelStatus, Hash, HoprBalance, HoprNodeChainOperations, HoprNodeNetworkOperations,
    HoprNodeOperations, HoprTransportIO, PeerId,
    api::node::state::HoprState,
    config::{HoprLibConfig, SessionGlobalConfig},
    prelude,
};
use tokio::time::sleep;

use crate::testing::{TestingConnector, TestingHopr};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeSafeConfig {
    pub safe_address: Address,
    pub module_address: Address,
}

pub fn create_hopr_instance_config(host_port: u16, safe: NodeSafeConfig, winn_prob: f64) -> HoprLibConfig {
    HoprLibConfig {
        host: hopr_lib::config::HostConfig {
            address: hopr_lib::config::HostType::default(),
            port: host_port,
        },
        safe_module: hopr_lib::config::SafeModule {
            safe_address: safe.safe_address,
            module_address: safe.module_address,
            ..Default::default()
        },
        protocol: hopr_lib::config::HoprProtocolConfig {
            transport: hopr_lib::config::TransportConfig {
                prefer_local_addresses: true,
                announce_local_addresses: true,
            },
            session: SessionGlobalConfig {
                idle_timeout: Duration::from_millis(2500),
                ..Default::default()
            },
            probe: hopr_lib::config::ProbeConfig {
                timeout: Duration::from_secs(2),
                max_parallel_probes: 10,
                recheck_threshold: Duration::from_secs(1),
                ..Default::default()
            },
            packet: hopr_lib::config::HoprPacketPipelineConfig {
                codec: hopr_lib::exports::transport::config::HoprCodecConfig {
                    outgoing_win_prob: Some(winn_prob.try_into().expect("invalid winning probability")),
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        publish: true,
        ..Default::default()
    }
}

pub struct TestedHopr {
    // Tokio runtime in which all long-running tasks of the HOPR node are spawned.
    runtime: Option<tokio::runtime::Runtime>,
    /// HOPR instance that is used for testing.
    pub instance: Arc<TestingHopr>,
    /// Transport socket that can be used to send and receive data via the HOPR node.
    pub socket: HoprTransportIO,
    pub connector: TestingConnector,
}

impl std::fmt::Debug for TestedHopr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestedHopr")
            .field("instance", &self.instance.me_onchain())
            .finish()
    }
}

impl TestedHopr {
    pub fn new(
        runtime: tokio::runtime::Runtime,
        instance: Arc<TestingHopr>,
        socket: HoprTransportIO,
        connector: TestingConnector,
    ) -> Self {
        assert_eq!(HoprState::Running, instance.status(), "hopr instance must be running");
        Self {
            runtime: Some(runtime),
            instance: instance,
            socket,
            connector,
        }
    }
}

impl Drop for TestedHopr {
    fn drop(&mut self) {
        let _ = self.instance.shutdown();
        std::thread::sleep(Duration::from_secs(1));
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
        tracing::debug!("hopr instance dropped");
    }
}

impl TestedHopr {
    pub fn inner(&self) -> &TestingHopr {
        &self.instance
    }

    pub fn address(&self) -> Address {
        self.instance.me_onchain()
    }

    pub fn peer_id(&self) -> PeerId {
        self.instance.me_peer_id()
    }

    pub fn connector(&self) -> &TestingConnector {
        &self.connector
    }

    pub fn config(&self) -> &HoprLibConfig {
        self.instance.config()
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
    pub channels: Vec<(Arc<TestingHopr>, Hash)>,
}

impl ChannelGuard {
    pub fn channel_id(&self, index: usize) -> &Hash {
        &self.channels[index].1
    }

    pub async fn try_open_channels_for_path<I, T>(path: I, funding: HoprBalance) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<Arc<TestingHopr>>,
    {
        let mut channels = vec![];

        let path: Vec<Arc<TestingHopr>> = path.into_iter().map(|item| item.into()).collect();
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

    pub async fn open_channel_between_nodes(
        src: Arc<TestingHopr>,
        dst: Arc<TestingHopr>,
        funding: HoprBalance,
    ) -> anyhow::Result<Self> {
        let channel = src
            .open_channel(&dst.me_onchain(), funding)
            .await
            .context("failed to open channel")?;

        Ok(Self {
            channels: vec![(src.clone(), channel.channel_id)],
        })
    }

    pub async fn try_close_channels_all_channels(&self) -> anyhow::Result<()> {
        let futures = self.channels.iter().map(|(hopr, channel_id)| {
            let hopr = hopr.clone();
            let channel_id = channel_id.clone();
            async move {
                if hopr
                    .channel_from_hash(&channel_id)
                    .await?
                    .is_some_and(|c| matches!(c.status, ChannelStatus::Open))
                {
                    hopr.close_channel_by_id(&channel_id)
                        .await
                        .map(|_| ())
                        .context("channel closure initiation must succeed")
                } else {
                    Ok(())
                }
            }
        });

        join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()?;

        sleep(Duration::from_secs(2)).await;

        let futures = self.channels.iter().map(|(hopr, channel_id)| {
            let hopr = hopr.clone();
            let channel_id = channel_id.clone();
            async move {
                if hopr
                    .channel_from_hash(&channel_id)
                    .await?
                    .is_some_and(|c| matches!(c.status, ChannelStatus::PendingToClose(_)))
                {
                    hopr.close_channel_by_id(&channel_id)
                        .await
                        .map(|_| ())
                        .context("closing channel must succeed")
                } else {
                    Ok(())
                }
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
