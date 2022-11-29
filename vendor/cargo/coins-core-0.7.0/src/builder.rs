//! The `builder` module defines an abstract `TxBuilder` trait. A concrete implementation for
//! Bitcoin can be found in the `bitcoin` crate

use crate::{
    enc::AddressEncoder,
    ser::ByteFormat,
    types::tx::{Input, Output, Transaction},
};
use std::io::Read;

/// A builder-pattern interface for constructing transactions. Implementations should accumulate
/// inputs, outputs, witnesses, and other TX data, and then `build()` a Transaction object from
/// the accumulated data.
pub trait TxBuilder: std::marker::Sized {
    /// The Transaction type returned by `build()`
    type Transaction: Transaction;

    /// An AddressEncoder that handles encoding and decoding network addresses. This is used in
    /// the `pay` function to decode addresses into associated `RecipientIdentifier`s.
    type Encoder: AddressEncoder;

    /// Instantiate a new builder
    fn new() -> Self;

    /// Instantiate a new builder from a transaction reference by cloning its properties
    fn from_tx_ref(tx: &Self::Transaction) -> Self;

    /// Instantiate a new builder from a transaction by taking ownership of its properties
    fn from_tx(tx: Self::Transaction) -> Self;

    /// Instantiate a new builder from a `std::io::Read` that contains a serialized tx
    fn read_from_tx<R>(reader: &mut R) -> Result<Self, <Self::Transaction as Transaction>::TxError>
    where
        R: Read,
    {
        let tx = Self::Transaction::read_from(reader)?;
        Ok(Self::from_tx(tx))
    }

    /// Instantiate a new builder from transaction hex
    fn from_hex_tx(hex_str: &str) -> Result<Self, <Self::Transaction as Transaction>::TxError> {
        let tx = Self::Transaction::deserialize_hex(hex_str)?;
        Ok(Self::from_tx(tx))
    }

    /// Set or overwrite the transaction version.
    ///
    /// If implementing a network without a version field, feel free to leave this as a NOP
    fn version(self, version: u32) -> Self;

    /// Spend an outpoint. Adds an unsigned input spending the associated outpoint with the
    /// specified sequence number.
    fn spend<I>(self, prevout: I, sequence: u32) -> Self
    where
        I: Into<<<Self::Transaction as Transaction>::TxIn as Input>::TxoIdentifier>;

    /// Pay an Address. Adds an output paying `value` to `address.`
    fn pay(
        self,
        value: <<Self::Transaction as Transaction>::TxOut as Output>::Value,
        address: &<Self::Encoder as AddressEncoder>::Address,
    ) -> Self;

    /// Insert an input at the specified index. Inputs after that are shifted to later indices.
    ///
    /// ## Note
    ///
    /// This may invalidate signatures made using ANYONECANPAY.
    fn insert_input(self, index: usize, input: <Self::Transaction as Transaction>::TxIn) -> Self;

    /// Add a set of inputs to the transaction.
    fn extend_inputs<I>(self, inputs: I) -> Self
    where
        I: IntoIterator<Item = <Self::Transaction as Transaction>::TxIn>;

    /// Insert an output at the specified index. Outputs after that are shifted to later indices.
    ///
    /// ## Note
    ///
    /// This may invalidate signatures made using SINGLE.
    fn insert_output(self, index: usize, output: <Self::Transaction as Transaction>::TxOut)
        -> Self;

    /// Add a set of outputs to the transaction.
    fn extend_outputs<I>(self, outputs: I) -> Self
    where
        I: IntoIterator<Item = <Self::Transaction as Transaction>::TxOut>;

    /// Set or overwrite the transaction locktime.
    ///
    /// If implementing a network without a locktime field, feel free to leave this as a NOP
    fn locktime(self, locktime: u32) -> Self;

    /// Consume the builder and produce a transaction from the builder's current state.
    fn build(self) -> Result<Self::Transaction, <Self::Transaction as Transaction>::TxError>;
}
