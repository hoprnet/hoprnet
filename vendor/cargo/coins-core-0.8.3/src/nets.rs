//! The `nets` module defines an abstract `Network.` The `Network` trait is a highly-abstracted
//! representation of the relationships between types in a UTXO network. Concrete implementations
//! for various Bitcoin networks are found in the `bitcoin` crate.

use crate::{
    builder::TxBuilder,
    enc::AddressEncoder,
    ser::ByteFormat,
    types::tx::{Input, Output, RecipientIdentifier, Transaction},
};

/// A Network describes a possible UTXO network. It is primarily a collection of types with
/// enforced relationships, but also provides convenient access the the transaction builder,
/// the address encoder, and other network-associated functionality.
///
/// Because we separate some commonly conflated functionality (e.g. output scripts and addresses)
/// we provide Networks to enforce relationships between them. This is why the `Network` trait's
/// associated types are complex. It exists to guarantee consistency of associated types across a
/// large number of disparate elements.
pub trait Network {
    /// A type handling the network's address semantics. This will typically represent some
    /// predicate on the transaction. It is used by both the `Encoder` and the `Builder`.
    type Address;

    /// A type representing the in-protocol recipient. This is usually different from the
    /// Address type.
    type RecipientIdentifier: RecipientIdentifier;

    /// An error type that will be used by the `Encoder`, and returned by the passthrough
    /// `encode_address` and `decode_address` functions
    type Error;

    /// An `Encoder` that uses the `Address` and `Error` types above. This `Encoder` must
    /// implement `AddressEncoder`. It handles translating the `Address` type to the networks
    /// `RecipientIdentifier` type.
    type Encoder: AddressEncoder<
        Address = Self::Address,
        Error = Self::Error,
        RecipientIdentifier = Self::RecipientIdentifier,
    >;

    /// A transaction Input type. This type is used within the `Transaction` and specificies UTXOs
    /// being spent by the transaction.
    type TxIn: Input + ByteFormat;
    /// A transaction Output type. This type is used within the `Transaction` and specificies
    /// UTXOs being consumed by the transaction.
    type TxOut: Output<RecipientIdentifier = Self::RecipientIdentifier> + ByteFormat;

    /// A Transaction type that uses the `TxIn` and `TxOut`.
    type Tx: Transaction<TxIn = Self::TxIn, TxOut = Self::TxOut>;

    /// A transaction Builder that uses the `Encoder` and `Transaction` types defined earlier.
    /// The builder is returned by `Network::tx_builder()`, and provides a convenient interface
    /// for transaction construction.
    type Builder: TxBuilder<Encoder = Self::Encoder, Transaction = Self::Tx>;

    /// Returns a new instance of the associated transaction builder.
    fn tx_builder() -> Self::Builder {
        Self::Builder::new()
    }

    /// Instantiate a builder from a tx object
    fn builder_from_tx_ref(tx: &Self::Tx) -> Self::Builder {
        Self::Builder::from_tx_ref(tx)
    }

    /// Instantiate a builder from a tx object
    fn builder_from_tx(tx: Self::Tx) -> Self::Builder {
        Self::Builder::from_tx(tx)
    }

    /// Instantiate a builder from a hex-serialized transaction
    fn builder_from_hex(hex_tx: &str) -> Result<Self::Builder, <Self::Tx as Transaction>::TxError> {
        Self::Builder::from_hex_tx(hex_tx)
    }

    /// Encode an address using the network's `Address` semantics
    fn encode_address(a: &Self::RecipientIdentifier) -> Result<Self::Address, Self::Error> {
        Self::Encoder::encode_address(a)
    }

    /// Decode an address using the network's `Address` semantics
    fn decode_address(addr: &Self::Address) -> Self::RecipientIdentifier {
        Self::Encoder::decode_address(addr)
    }

    /// Attempt to convert a string into an `Address`.
    fn string_to_address(s: &str) -> Result<Self::Address, Self::Error> {
        Self::Encoder::string_to_address(s)
    }
}
