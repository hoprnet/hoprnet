use std::time::Duration;

use futures::{
    future::{
        Either,
        FutureExt,      // .fuse()
        select
    },
    pin_mut,
    stream::{FuturesUnordered, StreamExt}
};
use libp2p::PeerId;

use utils_metrics::metrics::{SimpleHistogram, SimpleCounter};
use utils_misc::time::current_timestamp;

#[cfg(not(wasm))]
use async_std::task::sleep as sleep;

#[cfg(wasm)]
use gloo_timers::future::sleep as sleep;

const PINGS_MAX_PARALLEL: usize = 14;


#[cfg_attr(test, mockall::automock)]
pub(crate) trait PingCallable {
    fn send_msg(&self, peer: PeerId, protocols: &[String], msg: Box<[u8]>, include_reply: bool) -> Result<Box<[u8]>, String>;

    fn close_connection(&self, peer: &PeerId);

    fn on_finished_ping(&self, peer: &PeerId, result: crate::types::Result);
}


type PingMeasurement = (PeerId, crate::types::Result);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PingConfig {
    pub max_parallel_pings: usize,
    pub environment_id: &'static str,
    pub normalized_version: &'static str,
    pub timeout: Duration,
}

struct Ping {
    config: PingConfig,
    protocol_heartbeat: [String; 2],
    external_api: Box<dyn PingCallable>,
    metric_time_to_heartbeat: Option<SimpleHistogram>,
    metric_time_to_ping: Option<SimpleHistogram>,
    metric_successful_ping_count: Option<SimpleCounter>,
    metric_failed_ping_count: Option<SimpleCounter>,
}

impl Ping {
    pub fn new(config: PingConfig, external_api: Box<dyn PingCallable>) -> Ping {
        let config = PingConfig {
            max_parallel_pings: config.max_parallel_pings.min(PINGS_MAX_PARALLEL),
            ..config
        };

        Ping {
            config,
            protocol_heartbeat: [
                // new
                format!("/hopr/{}/heartbeat/{}", config.environment_id, config.normalized_version),
                // deprecated
                format!("/hopr/{}/heartbeat", config.environment_id)
            ],
            external_api,
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

    pub async fn ping_peers(&self, mut peers: Vec<PeerId>) {
        if peers.is_empty() {
            // TODO: log here
            ()
        }

        let _ping_peers_timer = match &self.metric_time_to_heartbeat {
            Some(metric_time_to_heartbeat) => {
                let timer = metric_time_to_heartbeat.start_measure();
                Some(scopeguard::guard((), move |_| { metric_time_to_heartbeat.cancel_measure(timer); }))
            },
            None => None
        };

        let remainder = peers.split_off(self.config.max_parallel_pings.min(peers.len()));

        let start = current_timestamp();
        let mut futs = to_futures_unordered(
            peers.iter()
                .map(|x| { self.ping_peer(x.clone(), self.config.timeout)})
                .collect::<Vec<_>>()
        );

        let mut waiting = remainder.iter();
        while let Some(heartbeat) = futs.next().await {
            if let Some(v) = waiting.next() {
                let remaining_time = current_timestamp() - start;
                if (remaining_time as u128) < self.config.timeout.as_millis() {
                    futs.push(self.ping_peer(v.clone(), Duration::from_millis(remaining_time)));
                }
            }

            self.external_api.on_finished_ping(&heartbeat.0, heartbeat.1);
        }
    }

    async fn ping_peer(&self, destination: PeerId, timeout_duration: Duration) -> PingMeasurement {
        // TODO: log(format!("Pinging the peer '{}'", destination));
        use rand::{RngCore,SeedableRng};
        let mut challenge = Box::new([0u8; 16]);
        rand::rngs::StdRng::from_entropy().fill_bytes(&mut challenge.as_mut_slice());

        let ping_result: PingMeasurement = {
            let _ping_peer_timer = match &self.metric_time_to_ping {
                Some(metric_time_to_ping) => {
                    let timer = metric_time_to_ping.start_measure();
                    Some(scopeguard::guard((), move |_| { metric_time_to_ping.cancel_measure(timer); }))
                },
                None => None
            };

            let timeout = sleep(std::cmp::min(timeout_duration, self.config.timeout)).fuse();
            let ping = async {
                let result = self.external_api.send_msg(destination.clone(), &self.protocol_heartbeat, challenge.clone(), true);
                result
            }.fuse();

            pin_mut!(timeout, ping);

            let ping_result: Result<(), String> = match select(
                timeout, ping).await
            {
                Either::Left(_) => Err(format!("The ping timed out {}s", timeout_duration.as_secs())),
                Either::Right((v, _)) => match v {
                    Ok(received) => {
                        let expected = crate::heartbeat::generate_ping_response(challenge.clone());
                        let r = match compare(expected.as_ref(), received.as_ref()) {
                            std::cmp::Ordering::Equal => Ok(()),
                            _ => Err(format!("Received incorrect reply for challenge, expected '{:x?}', but received: {:x?}", expected.as_ref(), received.as_ref()))
                        };

                        self.external_api.close_connection(&destination);

                        r
                    }
                    Err(description) => {
                        Err(format!("Error during ping to peer '{}': {}", destination.to_string(), description))
                    }
                },
            };

            // TODO: log(format!("The ping result for peer {} in {}", destination, ping_result));

            (
                destination,
                if ping_result.is_ok() { Ok(current_timestamp()) } else { Err(()) }
            )
        };

        match ping_result.1 {
            Ok(_) => {
                if let Some(metric_successful_ping_count) = &self.metric_successful_ping_count {
                    metric_successful_ping_count.increment(1u64);
                };
            }
            Err(_) => {
                if let Some(metric_failed_ping_count) = &self.metric_failed_ping_count {
                    metric_failed_ping_count.increment(1u64);
                };
            }
        }

        ping_result
    }
}

fn to_futures_unordered<F>(mut fs: Vec<F>) -> FuturesUnordered<F> {
    let futures: FuturesUnordered<F> = FuturesUnordered::new();
    for f in fs.drain(..) {
        futures.push(f);
    }

    futures
}

fn compare(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }

    for (ai, bi) in a.iter().zip(b.iter()) {
        match ai.cmp(&bi) {
            std::cmp::Ordering::Equal => continue,
            ord => return ord
        }
    }

    a.len().cmp(&b.len())
}


#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;
    use more_asserts::*;

    fn simple_ping_config() -> PingConfig {
        PingConfig {
            max_parallel_pings: 2,
            environment_id: "test",
            normalized_version: "1.0a",
            timeout: Duration::from_millis(150),
        }
    }

    #[async_std::test]
    async fn test_ping_peers_with_happy_path_should_trigger_the_desired_external_api_calls() {
        let peer = PeerId::random();

        let mut mock = MockPingCallable::new();
        mock.expect_send_msg()
            .returning(|_,_,msg,_| Ok(crate::heartbeat::generate_ping_response(msg)));
        mock.expect_on_finished_ping()
            .with(predicate::eq(peer), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());
        mock.expect_close_connection()
            .with(predicate::eq(peer))
            .return_const(());

        let pinger = Ping::new(simple_ping_config(), Box::new(mock) );
        pinger.ping_peers(vec![peer.clone()]).await;
    }

    #[async_std::test]
    async fn test_ping_should_invoke_a_failed_ping_reply_for_an_incorrect_reply() {
        let peer = PeerId::random();

        let mut mock = MockPingCallable::new();
        mock.expect_send_msg()
            .returning(|_,_,msg,_| {
                let mut reply = crate::heartbeat::generate_ping_response(msg);
                // let's damage the reply bytes
                for x in reply.iter_mut() {
                    *x = *x + 1;
                }
                Ok(reply)
            });
        mock.expect_on_finished_ping()
            .with(predicate::eq(peer), predicate::function(|x: &crate::types::Result| x.is_err()))
            .return_const(());
        mock.expect_close_connection()
            .with(predicate::eq(peer))
            .return_const(());

        let pinger = Ping::new(simple_ping_config(), Box::new(mock) );
        pinger.ping_peers(vec![peer.clone()]).await;
    }

    #[async_std::test]
    async fn test_ping_peer_invokes_send_message_but_the_time_runs_out() {
        let peer = PeerId::random();
        let mut ping_config = simple_ping_config();
        ping_config.timeout = Duration::from_millis(0);

        let mut mock = MockPingCallable::new();
        mock.expect_send_msg()
            .returning(move |_,_,msg,_| Ok(crate::heartbeat::generate_ping_response(msg)));
        mock.expect_on_finished_ping()
            .with(predicate::eq(peer), predicate::function(|x: &crate::types::Result| x.is_err()))
            .return_const(());
        mock.expect_close_connection()
            .with(predicate::eq(peer))
            .return_const(());

        let pinger = Ping::new(ping_config, Box::new(mock) );
        pinger.ping_peers(vec![peer.clone()]).await;
    }

    #[async_std::test]
    async fn test_ping_peers_empty_list_will_not_trigger_any_pinging() {
        let mock = MockPingCallable::new();
        let pinger = Ping::new(simple_ping_config(), Box::new(mock) );

        pinger.ping_peers(vec![]).await;
    }

    #[async_std::test]
    async fn test_ping_peers_multiple_peers_are_pinged_in_parallel() {
        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingCallable::new();
        mock.expect_send_msg()
            .returning(move |_,_,msg,_| {
                std::thread::sleep(Duration::from_millis(1));
                Ok(crate::heartbeat::generate_ping_response(msg))
            });
        mock.expect_send_msg()
            .returning(move |_,_,msg,_| {
                std::thread::sleep(Duration::from_millis(2));
                Ok(crate::heartbeat::generate_ping_response(msg))
            });
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[0].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[1].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());
        mock.expect_close_connection()
            .with(predicate::eq(peers[0].clone()))
            .return_const(());
        mock.expect_close_connection()
            .with(predicate::eq(peers[1].clone()))
            .return_const(());

        let pinger = Ping::new(simple_ping_config(), Box::new(mock) );
        pinger.ping_peers(peers).await;
    }

    #[async_std::test]
    async fn test_ping_peers_should_ping_parallel_only_a_limited_number_of_peers() {
        let mut config = simple_ping_config();
        config.max_parallel_pings = 1;

        let ping_delay_first = 10u64;
        let ping_delay_second = 10u64;

        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingCallable::new();
        mock.expect_send_msg()
            .returning(move |_,_,msg,_| {
                std::thread::sleep(Duration::from_millis(ping_delay_first));
                Ok(crate::heartbeat::generate_ping_response(msg))
            });
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[0].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());
        mock.expect_close_connection()
            .with(predicate::eq(peers[0].clone()))
            .return_const(());
        mock.expect_send_msg()
            .returning(move |_,_,msg,_| {
                std::thread::sleep(Duration::from_millis(ping_delay_second));
                Ok(crate::heartbeat::generate_ping_response(msg))
            });
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[1].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());
        mock.expect_close_connection()
            .with(predicate::eq(peers[1].clone()))
            .return_const(());

        let pinger = Ping::new(config, Box::new(mock) );

        let start = current_timestamp();
        pinger.ping_peers(peers).await;
        let end = current_timestamp();

        assert_ge!(end - start, ping_delay_first + ping_delay_second)
    }

    #[async_std::test]
    async fn test_ping_peer_should_not_ping_all_peers_if_the_max_timeout_is_reached() {
        let mut config = simple_ping_config();
        config.timeout = Duration::from_millis(1);
        config.max_parallel_pings = 1;

        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingCallable::new();
        mock.expect_send_msg()
            .returning(move |_,_,msg,_| {
                std::thread::sleep(Duration::from_millis(10));
                Ok(crate::heartbeat::generate_ping_response(msg))
            });
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[0].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());
        mock.expect_close_connection()
            .with(predicate::eq(peers[0].clone()))
            .return_const(());

        let pinger = Ping::new(config, Box::new(mock) );

        pinger.ping_peers(peers).await;
    }
}