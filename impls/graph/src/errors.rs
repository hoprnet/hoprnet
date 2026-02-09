use hopr_api::{Address, OffchainPublicKey};
use thiserror::Error;

/// Errors that can occur when manipulating the channel graph.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ChannelGraphError {
    /// Source and destination are the same node.
    #[error("channel source and destination must be different")]
    LoopChannel,
    /// Channel already exists in the graph.
    #[error("channel already exists between {0} and {1}")]
    ChannelAlreadyExists(Address, Address),
    /// Channel not found in the graph.
    #[error("channel not found between {0} and {1}")]
    ChannelNotFound(Address, Address),
    /// Node not found in the graph.
    #[error("node not found: {0}")]
    NodeNotFound(Address),
    /// Node identified by its public key not found in the graph.
    #[error("node with public key not found: {0}")]
    PublicKeyNodeNotFound(OffchainPublicKey),
}
