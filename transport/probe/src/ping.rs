use std::ops::Div;

use futures::{
    StreamExt,
    channel::mpsc::{Sender, channel},
};
use hopr_api::{OffchainPublicKey, graph::NetworkGraphError};
use hopr_async_runtime::prelude::timeout_fut;
use tracing::{debug, warn};

use crate::errors::{ProbeError, Result};

/// Heartbeat send ping TX type
pub type HeartbeatSendPingTx = Sender<(OffchainPublicKey, PingQueryReplier)>;

/// Configuration for the [`Ping`] mechanism
#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault)]
pub struct PingConfig {
    /// The timeout duration for an indiviual ping
    #[default(std::time::Duration::from_secs(30))]
    pub timeout: std::time::Duration,
}

/// Ping query result type holding data about the ping duration and the string
/// containg an optional version information of the pinged peer, if provided.
pub type PingQueryResult = std::result::Result<std::time::Duration, ()>;

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
    pub async fn ping(&self, peer: &OffchainPublicKey) -> Result<std::time::Duration> {
        let (tx, mut rx) = channel::<PingQueryResult>(1);
        let replier = PingQueryReplier::new(tx);

        if let Err(error) = self.send_ping.clone().try_send((*peer, replier)) {
            warn!(%peer, %error, "Failed to initiate a ping request");
        }

        match timeout_fut(self.config.timeout, rx.next()).await {
            Ok(Some(Ok(latency))) => {
                debug!(latency = latency.as_millis(), %peer, "Ping succeeded",);
                Ok(latency)
            }
            Ok(Some(Err(_))) => {
                debug!(%peer, "Ping failed internally",);
                Err(ProbeError::PingerError(format!("could not successfully ping: {peer}")))
            }
            Ok(None) => {
                debug!(%peer, "Ping canceled");
                Err(ProbeError::PingerError("canceled".into()))
            }
            Err(_) => {
                debug!(%peer, "ping failed due to timeout");
                Err(ProbeError::TrafficError(NetworkGraphError::ProbeNeighborTimeout(*peer)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::anyhow;
    use hex_literal::hex;

    use super::*;
    use crate::ping::Pinger;

    const SECRET_0: [u8; 32] = hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d");

    #[tokio::test]
    async fn ping_query_replier_should_yield_a_failed_probe() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::channel::<PingQueryResult>(256);

        let replier = PingQueryReplier::new(tx);

        replier.notify(Err(()));

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
        let result = result.map_err(|_| anyhow!("should succeed"))?;
        assert_eq!(result, RTT.div(2));

        Ok(())
    }

    #[tokio::test]
    async fn pinger_should_return_an_error_if_the_latency_is_longer_than_the_configure_timeout() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::channel::<(OffchainPublicKey, PingQueryReplier)>(256);

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
        assert!(pinger.ping(&OffchainPublicKey::from_privkey(&SECRET_0)?).await.is_err());

        delaying_channel.abort();

        Ok(())
    }
}
