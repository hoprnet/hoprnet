use std::ops::Div;

use futures::{
    StreamExt,
    channel::mpsc::{UnboundedSender, unbounded},
};
use hopr_async_runtime::prelude::timeout_fut;
use hopr_platform::time::native::current_time;
use hopr_primitive_types::prelude::AsUnixTimestamp;
use libp2p_identity::PeerId;
use tracing::{debug, warn};

use crate::{
    content::NeighborProbe,
    errors::{ProbeError, Result},
};

/// Heartbeat send ping TX type
///
/// NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
/// in case of faster input than output the memory might run out.
///
/// The unboundedness relies on the fact that a back pressure mechanism exists on a
/// higher level of the business logic making sure that only a fixed maximum count
/// of pings ever enter the queues at any given time.
pub type HeartbeatSendPingTx = UnboundedSender<(PeerId, PingQueryReplier)>;

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
    notifier: UnboundedSender<PingQueryResult>,
    challenge: (u64, NeighborProbe),
}

impl PingQueryReplier {
    pub fn new(notifier: UnboundedSender<PingQueryResult>) -> Self {
        Self {
            notifier,
            challenge: (
                current_time().as_unix_timestamp().as_millis() as u64,
                NeighborProbe::random_nonce(),
            ),
        }
    }

    /// Return a copy of the challenge for which the reply is expected
    pub fn challenge(&self) -> NeighborProbe {
        self.challenge.1
    }

    /// Mechanism to finalize the ping operation by providing a [`ControlMessage`] received by the
    /// transport layer.
    ///
    /// The resulting timing information about the RTT is halved to provide a unidirectional latency.
    pub fn notify(self, pong: NeighborProbe) {
        let timed_result = if self.challenge.1.is_complement_to(pong) {
            let unidirectional_latency = current_time()
                .as_unix_timestamp()
                .saturating_sub(std::time::Duration::from_millis(self.challenge.0))
                .div(2u32); // RTT -> unidirectional latency
            Ok(unidirectional_latency)
        } else {
            Err(ProbeError::DecodingError)
        };

        if self.notifier.unbounded_send(timed_result).is_err() {
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
        let (tx, mut rx) = unbounded::<PingQueryResult>();
        let replier = PingQueryReplier::new(tx);

        if let Err(error) = self.send_ping.clone().unbounded_send((peer, replier)) {
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
                Err(ProbeError::Timeout(self.config.timeout.as_secs()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{content::NeighborProbe, ping::Pinger};

    #[tokio::test]
    async fn ping_query_replier_should_return_ok_result_when_the_pong_is_correct_for_the_challenge()
    -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<PingQueryResult>();

        let replier = PingQueryReplier::new(tx);
        let challenge = replier.challenge.1;

        replier.notify(NeighborProbe::Pong(challenge.into()));

        assert!(rx.next().await.is_some_and(|r| r.is_ok()));

        Ok(())
    }

    #[tokio::test]
    async fn ping_query_replier_should_return_err_result_when_the_reply_is_incorrect_for_the_challenge()
    -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<PingQueryResult>();

        let replier = PingQueryReplier::new(tx);
        let challenge = replier.challenge.1;

        let wrong: u128 = challenge.into();
        let wrong = wrong + 1u128;

        replier.notify(NeighborProbe::Pong(wrong));

        assert!(rx.next().await.is_some_and(|r| r.is_err()));

        Ok(())
    }

    #[tokio::test]
    async fn ping_query_replier_should_return_the_latency_on_notification() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<PingQueryResult>();

        let replier = PingQueryReplier::new(tx);
        let challenge = replier.challenge.1;

        let delay = std::time::Duration::from_millis(10);
        tokio::time::sleep(delay).await;

        replier.notify(NeighborProbe::Pong(challenge.into()));

        let actual_latency = rx
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("should contain a result value"))?
            .map_err(|_e| anyhow::anyhow!("should contain a result value"))?;

        assert!(actual_latency > delay / 2);
        assert!(actual_latency < delay);

        Ok(())
    }

    #[tokio::test]
    async fn pinger_should_return_the_measeured_latency_when_no_issues_occur() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let ideal_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                let challenge = replier.challenge.1;

                replier.notify(NeighborProbe::Pong(challenge.into()));
            }
        });

        let pinger = Pinger::new(Default::default(), tx);
        pinger.ping(PeerId::random()).await?;

        ideal_channel.abort();

        Ok(())
    }

    #[tokio::test]
    async fn pinger_should_return_an_error_if_an_incorrect_nonce_is_replied() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let failing_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                replier.notify(NeighborProbe::Pong(NeighborProbe::random_nonce().into()));
            }
        });

        let pinger = Pinger::new(Default::default(), tx);
        assert!(pinger.ping(PeerId::random()).await.is_err());

        failing_channel.abort();

        Ok(())
    }

    #[tokio::test]
    async fn pinger_should_return_an_error_if_the_latency_is_longer_than_the_configure_timeout() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let delay = std::time::Duration::from_millis(10);
        let delaying_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                let challenge = replier.challenge.1;

                tokio::time::sleep(delay).await;

                replier.notify(NeighborProbe::Pong(challenge.into()));
            }
        });

        let pinger = Pinger::new(
            PingConfig {
                timeout: std::time::Duration::from_millis(0),
            },
            tx,
        );
        assert!(pinger.ping(PeerId::random()).await.is_err());

        delaying_channel.abort();

        Ok(())
    }
}
