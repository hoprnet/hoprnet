use libp2p_identity::PeerId;
use tracing::trace;

use hopr_crypto_types::prelude::*;
pub use hopr_db_api::protocol::AckResult;
use hopr_db_api::protocol::HoprDbProtocolOperations;
use hopr_internal_types::prelude::*;

use crate::errors::{ProtocolError, Result};

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
        if !ack.validate(&remote_pk) {
            tracing::error!("Failed to verify signature on received acknowledgement");
            return Err(ProtocolError::InvalidSignature);
        };

        self.db.handle_acknowledgement(ack, &self.chain_key).await.map_err(|e| {
            trace!(error = %e, "Failed to process a received acknowledgement");
            let error: ProtocolError = e.into();
            error
        })
    }
}
