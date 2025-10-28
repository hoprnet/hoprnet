use std::{str::FromStr, time::Duration};

use crate::{
    Address, Balance, ChannelEntry, ChannelStatus, Currency, Hopr, HoprBalance, HoprSession, PeerId, ProtocolsConfig,
    RoutingOptions, SessionClientConfig, SessionTarget, SurbBalancerConfig, prelude,
};
use anyhow::Context;
use hopr_crypto_types::{
    keypairs::ChainKeypair,
    prelude::{Keypair, OffchainKeypair},
};
use hopr_primitive_types::bounded::BoundedVec;
use hopr_transport::session::{Capabilities, IpOrHost, SealedHost};

use crate::testing::NodeSafeConfig;

pub struct HoprTester {
    instance: Hopr,
    pub safe_config: NodeSafeConfig,
}

impl HoprTester {
    pub fn new(
        chain_keys: ChainKeypair,
        anvil_endpoint: String,
        protocol_config: ProtocolsConfig,
        host_port: u16,
        db_path: String,
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
                chain: crate::config::Chain {
                    protocols: protocol_config,
                    provider: Some(anvil_endpoint.into()),
                    ..Default::default()
                },
                host: crate::config::HostConfig {
                    address: crate::config::HostType::default(),
                    port: host_port,
                },
                db: crate::config::Db {
                    data: db_path.into(),
                    ..Default::default()
                },
                safe_module: crate::config::SafeModule {
                    safe_address: safe.safe_address,
                    module_address: safe.module_address,
                    ..Default::default()
                },
                strategy: hopr_strategy::strategy::MultiStrategyConfig {
                    strategies: vec![hopr_strategy::Strategy::ClosureFinalizer(
                        hopr_strategy::channel_finalizer::ClosureFinalizerStrategyConfig {
                            max_closure_overdue: Duration::from_secs(1),
                        },
                    )],
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
            &OffchainKeypair::random(),
            &chain_keys,
        )
        .expect(format!("failed to create hopr instance on port {host_port}").as_str());

        Self {
            instance,
            safe_config: safe,
        }
    }

    pub fn inner(&self) -> &Hopr {
        &self.instance
    }

    pub async fn run<
        #[cfg(feature = "session-server")] T: crate::traits::session::HoprSessionServer + Clone + Send + 'static,
    >(
        &self,
        #[cfg(feature = "session-server")] server: T,
    ) -> anyhow::Result<()> {
        Ok(self
            .instance
            .run(
                #[cfg(feature = "session-server")]
                server,
            )
            .await
            .map(|_| ())?)
    }

    pub async fn get_balance<C: Currency + Send>(&self) -> Option<Balance<C>> {
        match self.instance.get_balance().await {
            Ok(balance) => Some(balance),
            Err(_) => None,
        }
    }

    pub async fn get_safe_balance<C: Currency + Send>(&self) -> Option<Balance<C>> {
        match self.instance.get_safe_balance().await {
            Ok(balance) => Some(balance),
            Err(_) => None,
        }
    }

    pub async fn safe_allowance(&self) -> Option<HoprBalance> {
        match self.instance.safe_allowance().await {
            Ok(allowance) => Some(allowance),
            Err(_) => None,
        }
    }

    pub fn address(&self) -> Address {
        self.instance.me_onchain()
    }

    pub fn peer_id(&self) -> PeerId {
        self.instance.me_peer_id()
    }

    pub async fn channel_from_hash(&self, channel_hash: &prelude::Hash) -> Option<ChannelEntry> {
        match self.instance.channel_from_hash(channel_hash).await {
            Ok(channel) => channel,
            Err(_) => None,
        }
    }

    pub async fn outgoing_channels_by_status(&self, status: ChannelStatus) -> Option<Vec<ChannelEntry>> {
        match self.instance.channels_from(&self.address()).await {
            Ok(channels) => Some(channels.iter().filter(|c| c.status == status).cloned().collect()),
            Err(_) => None,
        }
    }

    pub async fn create_raw_0_hop_session(&self, dst: &HoprTester) -> anyhow::Result<HoprSession> {
        let ip = IpOrHost::from_str(":0")?;

        let session = self
            .inner()
            .connect_to(
                dst.address(),
                SessionTarget::UdpStream(SealedHost::Plain(ip)),
                SessionClientConfig {
                    forward_path_options: RoutingOptions::Hops(0_u32.try_into()?),
                    return_path_options: RoutingOptions::Hops(0_u32.try_into()?),
                    capabilities: Capabilities::empty(),
                    pseudonym: None,
                    surb_management: None,
                    always_max_out_surbs: false,
                },
            )
            .await
            .expect("creating a session must succeed");

        Ok(session)
    }

    pub async fn create_1_hop_session(
        &self,
        mid: &HoprTester,
        dst: &HoprTester,
        capabilities: Option<Capabilities>,
        surb_management: Option<SurbBalancerConfig>,
    ) -> anyhow::Result<HoprSession> {
        let ip = IpOrHost::from_str(":0")?;
        let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(mid.address().into())));

        let session = self
            .inner()
            .connect_to(
                dst.address(),
                SessionTarget::UdpStream(SealedHost::Plain(ip)),
                SessionClientConfig {
                    forward_path_options: routing.clone(),
                    return_path_options: routing,
                    capabilities: capabilities.unwrap_or_default(),
                    pseudonym: None,
                    surb_management: surb_management,
                    always_max_out_surbs: false,
                },
            )
            .await
            .context("creating a session must succeed")?;

        Ok(session)
    }
}
