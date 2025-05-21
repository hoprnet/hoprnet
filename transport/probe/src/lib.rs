//! The crate provides the probing functionality used by the transport layer
//! to identify different attributes of possible transport paths in the network.
//!
//! The goal of probing is to establish a map with weighted properties that will
//! allow the caller to optimize transport, verify transport path properties and
//! lay groundworks for mitigating potential adversarial behavior.
//!
//! There are 2 fundamental types of probing:
//! 1. Immediate hop probing - collects telemetry for direct 0-hop neighbors. Such telemetry
//! can be identified and potentially gamed by an adversary, but it is still useful to identify
//! the basic properties of the most immediate connection to the neighbor, since in the worst
//! case scenario the mitigation strategy can discard unsuitable peers.
//!
//! 2. Multi-hop probing - collects telemetry using a probing mechanism based on looping. A loop
//! is a message sent by this peer to itself through different pre-selected peers. This probing
//! mechanism can be combined together with the cover traffic into a single mechanism improving
//! the network view.

pub mod config;
pub mod errors;
pub mod messaging;
pub mod ping;

pub mod neighbors;
pub mod store;

use std::{collections::HashMap, sync::Arc};

use hopr_internal_types::protocol::HoprPseudonym;
use hopr_network_types::types::ResolvedTransportRouting;
use hopr_transport_network::network::Network;
use hopr_transport_packet::prelude::ApplicationData;
use hopr_transport_protocol::processor::PacketSendFinalizer;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, strum::Display)]
pub enum HoprProbeProcess {
    #[strum(to_string = "heartbeat")]
    Heartbeat,
}

/// TODO
pub async fn probe_network<T, U, DB>(
    api: (T, U),
    network: Arc<Network<DB>>,
    cfg: config::ProbeConfig,
) -> HashMap<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>
where
    T: futures::Sink<(ApplicationData, ResolvedTransportRouting, PacketSendFinalizer)>,
    U: futures::Stream<Item = (HoprPseudonym, ApplicationData)>,
    DB: hopr_transport_network::HoprDbPeersOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    // // manual ping
    // let (ping_tx, ping_rx) = mpsc::unbounded::<(PeerId, PingQueryReplier)>();

    // let ping_cfg = PingConfig {
    //     timeout: self.cfg.protocol.heartbeat.timeout,
    //     max_parallel_pings: self.cfg.heartbeat.max_parallel_probes,
    // };

    // let ping: Pinger<network_notifier::PingExternalInteractions<T>> = Pinger::new(
    //     ping_cfg,
    //     ping_tx.clone(),
    //     network_notifier::PingExternalInteractions::new(
    //         self.network.clone(),
    //         self.db.clone(),
    //         self.path_planner.channel_graph(),
    //         network_events_tx,
    //     ),
    // );

    // self.ping
    //     .clone()
    //     .set(ping)
    //     .expect("must set the ping executor only once");

    // // heartbeat
    // let mut heartbeat = Heartbeat::new(
    //     self.cfg.heartbeat,
    //     self.ping
    //         .get()
    //         .expect("Ping should be initialized at this point")
    //         .clone(),
    //     HeartbeatExternalInteractions::new(self.network.clone()),
    //     Box::new(|dur| Box::pin(sleep(dur))),
    // );

    // let half_the_hearbeat_interval = self.cfg.heartbeat.interval / 4;
    // processes.insert(
    //     HoprTransportProcess::Probing(HoprProbeProcess::Heartbeat),
    //     spawn(async move {
    //         // present to make sure that the heartbeat does not start immediately
    //         hopr_async_runtime::prelude::sleep(half_the_hearbeat_interval).await;
    //         heartbeat.heartbeat_loop().await
    //     }),
    // );

    HashMap::<HoprProbeProcess, hopr_async_runtime::prelude::JoinHandle<()>>::new()
}
