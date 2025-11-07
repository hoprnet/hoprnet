use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::prelude::ChannelId;
use hopr_primitive_types::prelude::*;
use multiaddr::Multiaddr;

/// Represents the action previously parsed from an EIP-2718 transaction.
///
/// This is effectively inverse of a [`PayloadGenerator`](crate::payload::PayloadGenerator).
pub enum ParsedHoprChainAction {
    RegisterSafeAddress(Address),
    Announce {
        packet_key: OffchainPublicKey,
        multiaddress: Option<Multiaddr>,
    },
    Withdraw(Address, XDaiBalance),
    OpenChannel(Address, HoprBalance),
    InitializeChannelClosure(ChannelId),
    FinalizeChannelClosure(ChannelId),
    RedeemTicket(ChannelId, u64), // ChannelId and Ticket Index
}

impl ParsedHoprChainAction {
    /// Attempts to parse a signed EIP-2718 transaction previously generated via a
    /// [`PayloadGenerator`](crate::payload::PayloadGenerator).
    pub fn parse_from_eip2718(signed_tx: &[u8]) -> Result<(Self, Address), crate::errors::ChainTypesError> {
        todo!()
    }
}
