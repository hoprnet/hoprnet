use tracing::trace;

use hopr_crypto_types::prelude::*;
pub use hopr_db_api::protocol::AckResult;
use hopr_db_api::protocol::HoprDbProtocolOperations;
use hopr_internal_types::prelude::*;
use hopr_transport_identity::PeerId;

use crate::errors::{ProtocolError, Result};

/// Implements protocol acknowledgement logic for acknowledgements
#[derive(Clone)]
pub struct AcknowledgementProcessor<Db: HoprDbProtocolOperations> {
    db: Db,
}

impl<Db: HoprDbProtocolOperations> AcknowledgementProcessor<Db> {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Processes the outgoing acknowledgement.
    #[inline]
    #[tracing::instrument(level = "debug", skip(self, ack))]
    pub async fn send(&self, peer: &PeerId, ack: Acknowledgement) -> Acknowledgement {
        ack
    }

    /// Processes the incoming acknowledgement.
    #[tracing::instrument(level = "debug", skip(self, ack))]
    pub async fn recv(&self, peer: &PeerId, ack: Acknowledgement) -> Result<AckResult> {
        let remote_pk = OffchainPublicKey::try_from(peer)?;
        self.db
            .handle_acknowledgement(ack.validate(&remote_pk)?)
            .await
            .map_err(|e| {
                trace!(error = %e, "Failed to process a received acknowledgement");
                let error: ProtocolError = e.into();
                error
            })
    }
}
