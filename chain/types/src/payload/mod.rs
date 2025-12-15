//! Module defining various Ethereum transaction payload generators for the actions.
//!
//! This module defines the basic [`PayloadGenerator`] trait that describes how an action
//! is translated into a [`TransactionRequest`] that can be submitted on-chain.
//!
//! There are two main implementations:
//! - [`BasicPayloadGenerator`] which implements generation of a direct EIP1559 transaction payload. This is currently
//!   not used by a HOPR node.
//! - [`SafePayloadGenerator`] which implements generation of a payload that embeds the transaction data into the SAFE
//!   transaction. This is currently the main mode of HOPR node operation.
//!
//! These are currently based on the `hopr-bindings` crate.

#[cfg(feature = "use-bindings")]
mod bindings_based;

#[cfg(feature = "use-bindings")]
pub(crate) use bindings_based::KeyBindAndAnnouncePayload;
#[cfg(feature = "use-bindings")]
pub use bindings_based::{BasicPayloadGenerator, SafePayloadGenerator, TransactionRequest};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

type Result<T> = std::result::Result<T, crate::errors::ChainTypesError>;

/// Estimated gas parameters for a transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GasEstimation {
    /// Gas limit for the transaction.
    ///
    /// Defaults to 27 000 000.
    pub gas_limit: u64,
    /// Maximal fee per gas for the transaction.
    ///
    /// Defaults to 0.01 Gwei
    pub max_fee_per_gas: u128,
    /// Maximal priority fee per gas for the transaction.
    ///
    /// Defaults to 0.002 Gwei
    pub max_priority_fee_per_gas: u128,
}

impl Default for GasEstimation {
    fn default() -> Self {
        Self {
            gas_limit: 27_000_000,
            max_fee_per_gas: 10_000_000,         // 0.01 Gwei
            max_priority_fee_per_gas: 2_000_000, // 0.002 Gwei
        }
    }
}

/// Trait for transaction payloads that can be signed and encoded to EIP2718 format.
#[async_trait::async_trait]
pub trait SignableTransaction {
    /// Sign the transaction using the given chain keypair and encode it to EIP2718 format.
    async fn sign_and_encode_to_eip2718(
        self,
        nonce: u64,
        max_gas: Option<GasEstimation>,
        chain_keypair: &ChainKeypair,
    ) -> Result<Box<[u8]>>;
}

/// Trait for various implementations of common on-chain transaction payloads generators.
pub trait PayloadGenerator {
    type TxRequest: SignableTransaction + Send;

    /// Create an ERC20 approve transaction payload. Pre-requisite to open payment channels.
    /// The `spender` address is typically the HOPR Channels contract address.
    fn approve(&self, spender: Address, amount: HoprBalance) -> Result<Self::TxRequest>;

    /// Create a ERC20 transfer transaction payload
    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> Result<Self::TxRequest>;

    /// Creates the transaction payload to announce a node on-chain.
    fn announce(&self, announcement: AnnouncementData, key_binding_fee: HoprBalance) -> Result<Self::TxRequest>;

    /// Creates the transaction payload to open a payment channel
    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> Result<Self::TxRequest>;

    /// Creates the transaction payload to immediately close an incoming payment channel
    fn close_incoming_channel(&self, source: Address) -> Result<Self::TxRequest>;

    /// Creates the transaction payload that initiates the closure of a payment channel.
    /// Once the notice period is due, the funds can be withdrawn using a
    /// finalizeChannelClosure transaction.
    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<Self::TxRequest>;

    /// Creates a transaction payload that withdraws funds from
    /// an outgoing payment channel. This will succeed once the closure
    /// notice period is due.
    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<Self::TxRequest>;

    /// Used to create the payload to claim incentives for relaying a mixnet packet.
    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> Result<Self::TxRequest>;

    /// Creates a transaction payload to register a Safe instance which is used
    /// to manage the node's funds
    fn register_safe_by_node(&self, safe_addr: Address) -> Result<Self::TxRequest>;

    /// Creates a transaction payload to remove the Safe instance. Once succeeded,
    /// the node no longer manages the funds.
    fn deregister_node_by_safe(&self) -> Result<Self::TxRequest>;
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::ContractAddresses;

    lazy_static::lazy_static! {
        pub static ref CONTRACT_ADDRS: ContractAddresses = serde_json::from_str(r#"{
            "announcements": "0xf1c143B1bA20C7606d56aA2FA94502D25744b982",
            "channels": "0x77C9414043d27fdC98A6A2d73fc77b9b383092a7",
            "module_implementation": "0x32863c4974fBb6253E338a0cb70C382DCeD2eFCb",
            "network_registry": "0x15a315E1320cFF0de84671c0139042EE320CE38d",
            "network_registry_proxy": "0x20559cbD3C2eDcD0b396431226C00D2Cd102eB3F",
            "node_safe_registry": "0x4F7C7dE3BA2B29ED8B2448dF2213cA43f94E45c0",
            "node_safe_migration": "0x222222222222890352Ed9Ca694EdeAC49528D8F3",
            "node_stake_factory": "0x791d190b2c95397F4BcE7bD8032FD67dCEA7a5F2",
            "token": "0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1",
            "ticket_price_oracle": "0x442df1d946303fB088C9377eefdaeA84146DA0A6",
            "winning_probability_oracle": "0xC15675d4CCa538D91a91a8D3EcFBB8499C3B0471"
        }"#).unwrap();
    }
}
