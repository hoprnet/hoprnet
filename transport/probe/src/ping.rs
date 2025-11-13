use std::ops::Div;

use futures::{
    StreamExt,
    channel::mpsc::{Sender, channel},
};
use hopr_async_runtime::prelude::timeout_fut;
use libp2p_identity::PeerId;
use tracing::{debug, warn};

use crate::errors::{ProbeError, Result};

/// Heartbeat send ping TX type
pub type HeartbeatSendPingTx = Sender<(PeerId, PingQueryReplier)>;

/// Configuration for the [`Ping`] mechanism
#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault)]
pub struct PingConfig {
    /// The timeout duration for an indiviual ping
    #[default(std::time::Duration::from_secs(30))]
    pub timeout: std::time::Duration,
}

/// Ping query result type holding data about the ping duration and the string
/// containg an optional version information of the pinged peer, if provided.
pub type PingQueryResult = Result<std::time::Duration>;

/// Helper object allowing to send a ping query as a wrapped channel combination
/// that can be filled up on the transport part and awaited locally by the `Pinger`.
#[derive(Debug, Clone)]
pub struct PingQueryReplier {
    /// Back channel for notifications, is [`Clone`] to allow caching
    notifier: Sender<PingQueryResult>,
}

impl PingQueryReplier {
    pub fn new(notifier: Sender<PingQueryResult>) -> Self {
        Self { notifier }
    }

    /// Mechanism to finalize the ping operation by providing a [`ControlMessage`] received by the
    /// transport layer.
    ///
    /// The resulting timing information about the RTT is halved to provide a unidirectional latency.
    pub fn notify(mut self, result: PingQueryResult) {
        let result = result.map(|rtt| rtt.div(2u32));

        if self.notifier.try_send(result).is_err() {
            warn!("Failed to notify the ping query result due to upper layer ping timeout");
        }
    }
}

/// Implementation of the ping mechanism
#[derive(Debug, Clone)]
pub struct Pinger {
    config: PingConfig,
    send_ping: HeartbeatSendPingTx,
}

impl Pinger {
    pub fn new(config: PingConfig, send_ping: HeartbeatSendPingTx) -> Self {
        Self { config, send_ping }
    }

    /// Performs a ping to a single peer.
    #[tracing::instrument(level = "info", skip(self))]
    pub async fn ping(&self, peer: PeerId) -> Result<std::time::Duration> {
        let (tx, mut rx) = channel::<PingQueryResult>(1);
        let replier = PingQueryReplier::new(tx);

        if let Err(error) = self.send_ping.clone().try_send((peer, replier)) {
            warn!(%peer, %error, "Failed to initiate a ping request");
        }

        match timeout_fut(self.config.timeout, rx.next()).await {
            Ok(Some(Ok(latency))) => {
                debug!(latency = latency.as_millis(), %peer, "Ping succeeded",);
                Ok(latency)
            }
            Ok(Some(Err(e))) => {
                let error = if let ProbeError::DecodingError = e {
                    ProbeError::PingerError(peer, "incorrect pong response".into())
                } else {
                    e
                };

                debug!(%peer, %error, "Ping failed internally",);
                Err(error)
            }
            Ok(None) => {
                debug!(%peer, "Ping canceled");
                Err(ProbeError::PingerError(peer, "canceled".into()))
            }
            Err(_) => {
                debug!(%peer, "Ping failed due to timeout");
                Err(ProbeError::ProbeNeighborTimeout(peer))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::anyhow;

    use super::*;
    use crate::ping::Pinger;

    #[tokio::test]
    async fn ping_query_replier_should_yield_a_failed_probe() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::channel::<PingQueryResult>(256);

        let replier = PingQueryReplier::new(tx);

        replier.notify(Err(ProbeError::ProbeNeighborTimeout(PeerId::random())));

        assert!(rx.next().await.is_some_and(|r| r.is_err()));

        Ok(())
    }

    #[tokio::test]
    async fn ping_query_replier_should_yield_a_successful_probe_as_unidirectional_latency() -> anyhow::Result<()> {
        const RTT: Duration = Duration::from_millis(100);

        let (tx, mut rx) = futures::channel::mpsc::channel::<PingQueryResult>(256);

        let replier = PingQueryReplier::new(tx);

        replier.notify(Ok(RTT));

        let result = rx.next().await.ok_or(anyhow!("should contain a value"))?;

        assert!(result.is_ok());
        let result = result?;
        assert_eq!(result, RTT.div(2));

        Ok(())
    }

    #[tokio::test]
    async fn pinger_should_return_an_error_if_the_latency_is_longer_than_the_configure_timeout() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::channel::<(PeerId, PingQueryReplier)>(256);

        let delay = Duration::from_millis(10);
        let delaying_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                tokio::time::sleep(delay).await;

                replier.notify(Ok(delay));
            }
        });

        let pinger = Pinger::new(
            PingConfig {
                timeout: Duration::from_millis(0),
            },
            tx,
        );
        assert!(pinger.ping(PeerId::random()).await.is_err());

        delaying_channel.abort();

        Ok(())
    }
}
