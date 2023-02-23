use std::time::Duration;

use libp2p::PeerId;

use crate::peers::{NetworkPeerOrigin,NetworkPeers};

use utils_metrics::metrics::{MultiGauge, SimpleGauge, SimpleHistogram, SimpleCounter};

// #[cfg(not(wasm))]


const MAX_PARALLEL_HEARTBEATS: u32 = 14;
const HEARTBEAT_ROUND_TIMEOUT: Duration = Duration::from_secs(60);



#[derive(Debug)]
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct HeartbeatPingResult {
    pub destination: PeerId,
    pub last_seen: Option<u64>
}

pub struct HeartbeatConfig {
    pub max_parallel_heartbeats: u32,
    pub heartbeat_variance: f32,
    pub heartbeat_interval: u32,
    pub heartbeat_threshold: u32,
    pub network_quality_threshold: f32,
}


/// The values of Network health indicator are valid prometheus metrics values, they cannot be
/// arbitrarily changed.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NetworkHealthIndicator {
    UNKNOWN = 0,
    /// No connection, default
    RED = 1,
    /// Low quality (<= 0.5) connection to at least 1 public relay
    ORANGE = 2,
    /// High quality (> 0.5) connection to at least 1 public relay
    YELLOW = 3,
    /// High quality (> 0.5) connection to at least 1 public relay and 1 NAT node
    GREEN = 4,
}

impl std::fmt::Display for NetworkHealthIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


struct Heartbeat {
    config: HeartbeatConfig,
    health: NetworkHealthIndicator,
    environment_id: String,
    protocol_heartbeat: [String; 2],
    send_msg_cb: Box<dyn Fn()>,
    is_public_node_cb: Box<dyn Fn(&PeerId) -> bool>,
    close_connections_to_cb: Box<dyn Fn(&PeerId)>,
    metric_time_to_heartbeat: Option<SimpleHistogram>,
    metric_time_to_ping: Option<SimpleHistogram>,
    metric_successful_ping_count: Option<SimpleCounter>,
    metric_failed_ping_count: Option<SimpleCounter>,
}

impl Heartbeat {
    pub fn new(config: HeartbeatConfig, environment_id: &str, normalized_version: &str) -> Heartbeat {
        let config = HeartbeatConfig {
            max_parallel_heartbeats: config.max_parallel_heartbeats.max(MAX_PARALLEL_HEARTBEATS),
            ..config
        };

        Heartbeat {
            config,
            health: NetworkHealthIndicator::UNKNOWN,
            environment_id: environment_id.to_owned(),
            protocol_heartbeat: [
                format!("/hopr/{environment_id}/heartbeat/{normalized_version}"),       // new
                format!("/hopr/{environment_id}/heartbeat")                             // deprecated
            ],
            send_msg_cb: Box::new(|| {}),
            is_public_node_cb: Box::new(|_| { true }),
            close_connections_to_cb: Box::new(|_| {  }),
            metric_time_to_heartbeat: SimpleHistogram::new(
                "core_histogram_heartbeat_time_seconds",
                "Measures total time it takes to probe all other nodes (in seconds)",
                vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0]
            ).ok(),
            metric_time_to_ping: SimpleHistogram::new(
                "core_histogram_ping_time_seconds",
                "Measures total time it takes to ping a single node (seconds)",
                vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0]
            ).ok(),
            metric_successful_ping_count: SimpleCounter::new(
                "core_counter_heartbeat_successful_pings",
                "Total number of successful pings"
            ).ok(),
            metric_failed_ping_count: SimpleCounter::new(
                "core_counter_heartbeat_failed_pings",
                "Total number of failed pings"
            ).ok(),
        }
    }

    pub fn start(&self) {

    }

    pub fn stop(&self) {

    }

    fn check_nodes(&self) {

    }

    pub fn ping_node(&mut self, destination: &PeerId) -> HeartbeatPingResult {
        // log(format!("Pinging the node with peer id {destination}"));
        use rand::{RngCore,SeedableRng};
        let mut challenge: [u8; 16] = [0u8; 16];
        rand::rngs::StdRng::from_entropy().fill_bytes(&mut challenge);

        let _ping_timer = match &self.metric_time_to_ping {
            Some(metric_time_to_ping) => {
                let timer = metric_time_to_ping.start_measure();
                Some(scopeguard::guard((), move |_| { metric_time_to_ping.cancel_measure(timer); }))
            },
            None => None
        };

        // timeout with send message should be here
        let is_successful = true;

        let ping_result = HeartbeatPingResult{
            destination: destination.clone(),
            last_seen: if is_successful { Some(4u64) } else { None },       // TODO: proper timestamp
        };


        match ping_result.last_seen {
            Some(_) => {
                if let Some(metric_successful_ping_count) = &self.metric_successful_ping_count {
                    metric_successful_ping_count.increment(1u64);
                };
            }
            None => {
                if let Some(metric_failed_ping_count) = &self.metric_failed_ping_count {
                    metric_failed_ping_count.increment(1u64);
                };
            }
        }

        ping_result
    }

    /**
     * Recalculates the network health indicator based on the
     * current network state knowledge.
     * @returns Value of the current network health indicator (possibly updated).
     */
    pub fn recalculate_network_health() -> NetworkHealthIndicator {
        NetworkHealthIndicator::UNKNOWN
    }
}


#[cfg(test)]
mod tests {
    use crate::heartbeat;
    use super::*;

    const HEARTBEAT_CONFIG: HeartbeatConfig = HeartbeatConfig {
        max_parallel_heartbeats: 10,
        heartbeat_variance: 10.0,
        heartbeat_interval: 1000,
        heartbeat_threshold: 100,       // time since we want to ping again
        network_quality_threshold: 0.1,
    };

    #[test]
    fn test_entry_should_advance_time_on_ping() {
        let heartbeat = Heartbeat::new(HEARTBEAT_CONFIG, "environment_id", "1.2.3");
        assert_eq!(2828, 2828);
    }
}