use hopr_api::OffchainPublicKey;
use thiserror::Error;

/// Errors that can occur when manipulating the channel graph.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ChannelGraphError {
    /// Node identified by its public key not found in the graph.
    #[error("node with public key not found: {0}")]
    PublicKeyNodeNotFound(OffchainPublicKey),
}
