use std::{fmt::Formatter, sync::Arc, time::Duration};

use anyhow::Context;
use futures::future::join_all;

use crate::{
    api::{
        PeerId,
        node::{HasChainApi, HoprNodeOperations, HoprState, IncentiveChannelOperations},
        types::{
            crypto::prelude::Hash,
            internal::prelude::{ChannelEntry, ChannelStatus},
            primitive::prelude::{Address, HoprBalance},
        },
    },
    config::{HoprLibConfig, SessionGlobalConfig},
    testing::TestingHopr,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeSafeConfig {
    pub safe_address: Address,
    pub module_address: Address,
}

pub fn create_hopr_instance_config(
    host_port: u16,
    safe: NodeSafeConfig,
    winn_prob: f64,
    pix_config: Option<crate::exports::transport::session::IncomingSessionPixConfig>,
    idle_timeout_ms: u64,
    pix_global_config: Option<crate::exports::transport::config::PixGlobalConfig>,
) -> HoprLibConfig {
    HoprLibConfig {
        host: crate::config::HostConfig {
            address: crate::config::HostType::default(),
            port: host_port,
        },
        safe_module: crate::config::SafeModule {
            safe_address: safe.safe_address,
            module_address: safe.module_address,
        },
        protocol: crate::config::HoprProtocolConfig {
            transport: crate::config::TransportConfig {
                prefer_local_addresses: true,
                announce_local_addresses: true,
            },
            session: SessionGlobalConfig {
                idle_timeout: Duration::from_millis(idle_timeout_ms),
                ..Default::default()
            },
            probe: crate::config::ProbeConfig {
                timeout: Duration::from_secs(2),
                max_parallel_probes: 10,
                recheck_threshold: Duration::from_secs(1),
                ..Default::default()
            },
            packet: crate::config::HoprPacketPipelineConfig {
                codec: crate::exports::transport::config::HoprCodecConfig {
                    outgoing_win_prob: Some(winn_prob.try_into().expect("invalid winning probability")),
                    ..Default::default()
                },
                ..Default::default()
            },
            mixer: Default::default(),
            stream: Default::default(),
            path_planner: Default::default(),
            counter_flush_interval: Default::default(),
            pix: pix_global_config.unwrap_or_default(),
            incoming_session_pix_config: pix_config.unwrap_or_default(),
        },
        publish: true,
        ..Default::default()
    }
}

pub struct TestedHopr<TMgr: 'static = crate::testing::wiring::SharedTicketManager> {
    // Tokio runtime in which all long-running tasks of the HOPR node are spawned.
    runtime: Option<tokio::runtime::Runtime>,
    /// HOPR instance that is used for testing.
    pub instance: Arc<
        crate::Hopr<
            crate::testing::TestingConnector,
            crate::testing::TestingGraph,
            hopr_transport_p2p::HoprNetwork,
            TMgr,
        >,
    >,
    pub connector: crate::testing::TestingConnector,
}

impl<TMgr: 'static> std::fmt::Debug for TestedHopr<TMgr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestedHopr")
            .field("instance", &self.instance.identity().node_address)
            .finish()
    }
}

impl<TMgr: 'static + Send + Sync> TestedHopr<TMgr> {
    pub fn new(
        runtime: tokio::runtime::Runtime,
        instance: Arc<
            crate::Hopr<
                crate::testing::TestingConnector,
                crate::testing::TestingGraph,
                hopr_transport_p2p::HoprNetwork,
                TMgr,
            >,
        >,
        connector: crate::testing::TestingConnector,
    ) -> Self {
        assert_eq!(
            HoprState::Running,
            HoprNodeOperations::status(&*instance),
            "hopr instance must be running"
        );
        Self {
            runtime: Some(runtime),
            instance,
            connector,
        }
    }
}

impl<TMgr: 'static> Drop for TestedHopr<TMgr> {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
        tracing::debug!("hopr instance dropped");
    }
}

impl<TMgr: 'static + Send + Sync> TestedHopr<TMgr> {
    pub fn inner(
        &self,
    ) -> &crate::Hopr<
        crate::testing::TestingConnector,
        crate::testing::TestingGraph,
        hopr_transport_p2p::HoprNetwork,
        TMgr,
    > {
        &self.instance
    }

    pub fn address(&self) -> Address {
        self.instance.identity().node_address
    }

    pub fn peer_id(&self) -> PeerId {
        (*self.instance.graph().me()).into()
    }

    pub fn connector(&self) -> &crate::testing::TestingConnector {
        &self.connector
    }

    pub fn config(&self) -> &HoprLibConfig {
        self.instance.config()
    }

    pub async fn channel_from_hash(&self, channel_hash: &Hash) -> Option<ChannelEntry> {
        IncentiveChannelOperations::channel_by_id(&*self.instance, channel_hash).unwrap_or(None)
    }

    pub async fn outgoing_channels_by_status(&self, status: ChannelStatus) -> Option<Vec<ChannelEntry>> {
        match IncentiveChannelOperations::channels_from(&*self.instance, self.address()).await {
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

            let channel = IncentiveChannelOperations::open_channel(&**src, dst.identity().node_address, funding)
                .await
                .context("opening channel must succeed")?;

            channels.push((
                src.clone(),
                *channel.output().expect("open_channel must return a channel ID"),
            ));
        }

        Ok(Self { channels })
    }

    pub async fn open_channel_between_nodes(
        src: Arc<TestingHopr>,
        dst: Arc<TestingHopr>,
        funding: HoprBalance,
    ) -> anyhow::Result<Self> {
        let channel = IncentiveChannelOperations::open_channel(&*src, dst.identity().node_address, funding)
            .await
            .context("failed to open channel")?;

        Ok(Self {
            channels: vec![(
                src.clone(),
                *channel.output().expect("open_channel must return a channel ID"),
            )],
        })
    }

    pub async fn try_close_channels_all_channels(&self) -> anyhow::Result<()> {
        // First pass: initiate closure on Open channels → PendingToClose
        let futures = self.channels.iter().map(|(hopr, channel_id)| {
            let hopr = hopr.clone();
            let channel_id = *channel_id;
            async move {
                if IncentiveChannelOperations::channel_by_id(&*hopr, &channel_id)?
                    .is_some_and(|c| matches!(c.status, ChannelStatus::Open))
                {
                    IncentiveChannelOperations::close_channel_by_id(&*hopr, &channel_id)
                        .await
                        .map(|_| ())
                        .context("channel closure initiation must succeed")
                } else {
                    Ok(())
                }
            }
        });

        join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()?;

        // Poll each channel until it reaches Closed state, attempting finalization
        // when the grace period has elapsed (PendingToClose → Closed).
        for (hopr, channel_id) in &self.channels {
            let hopr = hopr.clone();
            let channel_id = *channel_id;
            super::wait_until(
                || async {
                    let ch = IncentiveChannelOperations::channel_by_id(&*hopr, &channel_id)
                        .ok()
                        .flatten();
                    match ch.as_ref().map(|c| &c.status) {
                        None | Some(ChannelStatus::Closed) => return Ok(true),
                        Some(ChannelStatus::PendingToClose(_)) => {
                            // Attempt finalization; ignore errors (grace period may not have elapsed)
                            let _ = IncentiveChannelOperations::close_channel_by_id(&*hopr, &channel_id).await;
                        }
                        _ => {}
                    }
                    Ok::<_, crate::errors::HoprLibError>(false)
                },
                Duration::from_secs(5),
            )
            .await
            .context("channel did not reach Closed state")?;
        }

        Ok(())
    }
}

impl Drop for ChannelGuard {
    fn drop(&mut self) {
        let channels = self.channels.clone();
        let cleanup = std::thread::Builder::new().spawn(move || {
            if let Ok(runtime) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                runtime.block_on(async move {
                    for (hopr, channel_id) in &channels {
                        let _ = IncentiveChannelOperations::close_channel_by_id(&**hopr, channel_id).await;
                    }

                    // Poll each channel until Closed, attempting finalization when possible
                    for (hopr, channel_id) in &channels {
                        let _ = super::wait_until(
                            || async {
                                let ch = IncentiveChannelOperations::channel_by_id(&**hopr, channel_id)
                                    .ok()
                                    .flatten();
                                match ch.as_ref().map(|c| &c.status) {
                                    None | Some(ChannelStatus::Closed) => return Ok(true),
                                    Some(ChannelStatus::PendingToClose(_)) => {
                                        let _ =
                                            IncentiveChannelOperations::close_channel_by_id(&**hopr, channel_id).await;
                                    }
                                    _ => {}
                                }
                                Ok::<_, crate::errors::HoprLibError>(false)
                            },
                            Duration::from_secs(5),
                        )
                        .await;
                    }
                });
            }
        });

        if let Ok(handle) = cleanup {
            let _ = handle.join();
        }
    }
}
