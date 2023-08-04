use std::io::Write;

use digest::Digest;

use crate::{
    hashes::{DigestOutput, MarkedDigest, MarkedDigestOutput},
    ser::{ByteFormat, SerError},
};

/// A `TxoIdentifier` represents the network's unique identifier an output. In Bitcoin this is an
/// outpoint.
pub trait TxoIdentifier {}

/// An `Input` spends a specific TXO, and typically contains a `TxoIdentifier` for that TXO.
pub trait Input {
    /// An input must define what type contains the TXO ID it is spending.
    type TxoIdentifier: TxoIdentifier;
}

/// A RecipientIdentifier represents the network's identifier for a recipient. In Bitcoin this is
/// a `ScriptPubkey`.
pub trait RecipientIdentifier {}

/// An Output represents a new TXO being created. It has an associated `RecipientIdentifier`.
pub trait Output {
    /// How is value represented in this Output? For Bitcoin this is a u64.
    type Value;
    /// The associated `RecipientIdentifier` type that describes to whom the output is paid.
    /// For Bitcoin, this is a `ScriptPubkey`
    type RecipientIdentifier: RecipientIdentifier;
}

/// Basic functionality for a Transaction
///
/// This trait has been generalized to support transactions from Non-Bitcoin networks. The
/// transaction specificies which types it considers to be inputs and outputs, and a struct that
/// contains its Sighash arguments. This allows others to define custom transaction types with
/// unique functionality.
pub trait Transaction: ByteFormat {
    /// An associated error type, used in Results returned by the Transaction.
    type TxError: From<SerError> + From<<Self as ByteFormat>::Error>;
    /// The Input type for the transaction
    type TxIn: Input;
    /// The Output type for the transaction
    type TxOut: Output;
    /// A type describing arguments for the sighash function for this transaction.
    type SighashArgs;
    /// A marked hash (see crate::hashes::marked) to be used as the transaction ID type.
    type TXID: MarkedDigestOutput;
    /// A type that implements `HashWriter`. Used to generate the `TXID` and `Sighash`.
    type HashWriter: MarkedDigest<Self::TXID>;

    /// Instantiate a new Transaction by specifying inputs and outputs.
    fn new<I, O>(version: u32, vin: I, vout: O, locktime: u32) -> Result<Self, Self::TxError>
    where
        I: Into<Vec<Self::TxIn>>,
        O: Into<Vec<Self::TxOut>>,
        Self: Sized;

    /// Returns the transaction version number
    fn version(&self) -> u32;

    /// Returns a reference to the transaction input vector
    fn inputs(&self) -> &[Self::TxIn];

    /// Returns a reference the the transaction output vector
    fn outputs(&self) -> &[Self::TxOut];

    /// Returns the transaction's nLocktime field
    fn locktime(&self) -> u32;

    /// Calculates and returns the transaction's ID. The default TXID is simply the digest of the
    /// serialized transaction.
    fn txid(&self) -> Self::TXID {
        let mut w = Self::HashWriter::default();
        self.write_to(&mut w)
            .expect("No IOError from hash functions");
        w.finalize_marked()
    }

    /// Generate the digest that must be signed to authorize inputs. For Bitcoin transactions
    /// this is a function of the transaction, and the input's prevout.
    ///
    /// # Note:
    ///
    /// For Bitcoin, this will write the DEFAULT sighash for the current transaction type. For
    /// witness transactions, that is the BIP143 sighash. When signing Legacy inputs included in a
    /// witness transaction, use `write_legacy_sighash_preimage` instead.
    fn write_sighash_preimage<W: Write>(
        &self,
        writer: &mut W,
        _args: &Self::SighashArgs,
    ) -> Result<(), Self::TxError>;

    /// Calls `write_sighash_preimage` with the provided arguments and a new HashWriter.
    /// Returns the sighash digest which should be signed.
    fn sighash(
        &self,
        args: &Self::SighashArgs,
    ) -> Result<DigestOutput<Self::HashWriter>, Self::TxError> {
        let mut w = Self::HashWriter::default();
        self.write_sighash_preimage(&mut w, args)?;
        Ok(w.finalize())
    }
}
