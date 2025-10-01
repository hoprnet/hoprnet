use std::time::Duration;

use hopr_crypto_types::{
    keypairs::ChainKeypair,
    prelude::{Keypair, OffchainKeypair},
};
use hopr_lib::{Address, Balance, ChannelEntry, ChannelStatus, Currency, Hopr, HoprBalance, PeerId, prelude};

use crate::common::NodeSafeConfig;

pub struct HoprTester(Hopr);

impl HoprTester {
    pub fn new(
        chain_keys: ChainKeypair,
        anvil_endpoint: String,
        protocol_config: hopr_lib::ProtocolsConfig,
        host_port: u16,
        db_path: String,
        safe: NodeSafeConfig,
    ) -> Self {
        let instance = Hopr::new(
            hopr_lib::config::HoprLibConfig {
                probe: hopr_lib::config::ProbeConfig {
                    timeout: Duration::from_secs(2),
                    max_parallel_probes: 10,
                    recheck_threshold: Duration::from_secs(1),
                    ..Default::default()
                },
                network_options: hopr_lib::config::NetworkConfig {
                    ignore_timeframe: Duration::from_secs(0),
                    ..Default::default()
                },
                chain: hopr_lib::config::Chain {
                    protocols: protocol_config,
                    provider: Some(anvil_endpoint.into()),
                    ..Default::default()
                },
                host: hopr_lib::config::HostConfig {
                    address: hopr_lib::config::HostType::default(),
                    port: host_port,
                },
                db: hopr_lib::config::Db {
                    data: db_path.into(),
                    ..Default::default()
                },
                safe_module: hopr_lib::config::SafeModule {
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
                transport: hopr_lib::config::TransportConfig {
                    prefer_local_addresses: true,
                    announce_local_addresses: true,
                },
                ..Default::default()
            },
            &OffchainKeypair::random(),
            &chain_keys,
        )
        .expect(format!("failed to create hopr instance on port {host_port}").as_str());

        Self(instance)
    }

    pub fn inner(&self) -> &Hopr {
        &self.0
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        match self.0.run().await {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    pub async fn get_balance<C: Currency + Send>(&self) -> Option<Balance<C>> {
        match self.0.get_balance().await {
            Ok(balance) => Some(balance),
            Err(_) => None,
        }
    }

    pub async fn get_safe_balance<C: Currency + Send>(&self) -> Option<Balance<C>> {
        match self.0.get_safe_balance().await {
            Ok(balance) => Some(balance),
            Err(_) => None,
        }
    }

    pub async fn safe_allowance(&self) -> Option<HoprBalance> {
        match self.0.safe_allowance().await {
            Ok(allowance) => Some(allowance),
            Err(_) => None,
        }
    }

    pub fn address(&self) -> Address {
        self.0.me_onchain()
    }

    pub fn peer_id(&self) -> PeerId {
        self.0.me_peer_id()
    }

    pub async fn channel_from_hash(&self, channel_hash: &prelude::Hash) -> Option<ChannelEntry> {
        match self.0.channel_from_hash(channel_hash).await {
            Ok(channel) => channel,
            Err(_) => None,
        }
    }

    pub async fn outgoing_channels_by_status(&self, status: ChannelStatus) -> Option<Vec<ChannelEntry>> {
        match self.0.channels_from(&self.address()).await {
            Ok(channels) => Some(channels.iter().filter(|c| c.status == status).cloned().collect()),
            Err(_) => None,
        }
    }
}
