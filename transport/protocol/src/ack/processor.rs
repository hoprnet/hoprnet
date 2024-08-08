use libp2p_identity::PeerId;
use tracing::trace;

use hopr_crypto_types::prelude::*;
pub use hopr_db_api::protocol::AckResult;
use hopr_db_api::protocol::HoprDbProtocolOperations;
use hopr_internal_types::prelude::*;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleCounter};

use crate::errors::{ProtocolError, Result};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_RECEIVED_ACKS: MultiCounter = MultiCounter::new(
        "hopr_received_ack_count",
        "Number of received acknowledgements",
        &["valid"]
    )
    .unwrap();
    static ref METRIC_SENT_ACKS: SimpleCounter =
        SimpleCounter::new("hopr_sent_acks_count", "Number of sent message acknowledgements").unwrap();
    static ref METRIC_TICKETS_COUNT: MultiCounter =
        MultiCounter::new("hopr_tickets_count", "Number of winning tickets", &["type"]).unwrap();
}

/// Implements protocol acknowledgement logic for acknowledgements
#[derive(Clone)]
pub struct AcknowledgementProcessor<Db: HoprDbProtocolOperations> {
    db: Db,
    chain_key: ChainKeypair,
}

impl<Db: HoprDbProtocolOperations> AcknowledgementProcessor<Db> {
    pub fn new(db: Db, chain_key: &ChainKeypair) -> Self {
        Self {
            db,
            chain_key: chain_key.clone(),
        }
    }

    /// Processes the outgoing acknowledgement.
    #[inline]
    #[tracing::instrument(level = "debug", skip(self, ack))]
    pub async fn send(&self, peer: &PeerId, ack: Acknowledgement) -> Acknowledgement {
        ack
    }

    /// Processes the incoming acknowledgement.
    #[tracing::instrument(level = "debug", skip(self, ack))]
    pub async fn recv(&self, peer: &PeerId, mut ack: Acknowledgement) -> Result<AckResult> {
        let remote_pk = OffchainPublicKey::try_from(peer)?;
        let ack = ack.validate(&remote_pk).then_some(ack).ok_or_else(|| {
            trace!("Failed to verify signature on received acknowledgement");
            ProtocolError::InvalidSignature
        })?;

        Ok(self
            .db
            .handle_acknowledgement(ack, &self.chain_key)
            .await
            .map(|reply| {
                #[cfg(all(feature = "prometheus", not(test)))]
                match &reply {
                    AckResult::Sender(_) => {
                        METRIC_RECEIVED_ACKS.increment(&["true"]);
                    }
                    AckResult::RelayerWinning(_) => {
                        METRIC_RECEIVED_ACKS.increment(&["true"]);
                        METRIC_TICKETS_COUNT.increment(&["winning"]);
                    }
                    AckResult::RelayerLosing => {
                        METRIC_RECEIVED_ACKS.increment(&["true"]);
                        METRIC_TICKETS_COUNT.increment(&["losing"]);
                    }
                }

                reply
            })
            .map_err(|e| {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_ACKS.increment(&["false"]);

                trace!("Failed to process a received acknowledgement: {e}");
                let error: ProtocolError = e.into();
                error
            })?)
    }
}
