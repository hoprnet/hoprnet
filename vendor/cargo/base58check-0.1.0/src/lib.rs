//! Base58Check-to-text encoding

extern crate base58;
extern crate sha2;

use std::iter;
use base58::{FromBase58, ToBase58};
use sha2::{Digest, Sha256};

pub use base58::FromBase58Error;

/// Errors that can occur when decoding base58check encoded string.
#[derive(Debug, PartialEq)]
pub enum FromBase58CheckError {
    /// Base58 error.
    InvalidBase58(FromBase58Error),
    /// The input had invalid checksum.
    InvalidChecksum,
}

/// A trait for converting a value to base58 encoded string.
pub trait ToBase58Check {
    /// Converts a value of `self` to a base58 value, returning the owned string.
    fn to_base58check(&self, version: u8) -> String;
}

/// A trait for converting base58check encoded values.
pub trait FromBase58Check {
    /// Convert a value of `self`, interpreted as base58check encoded data, into the tuple with version and payload as bytes vector.
    fn from_base58check(&self) -> Result<(u8, Vec<u8>), FromBase58CheckError>;
}

impl ToBase58Check for [u8] {
    fn to_base58check(&self, version: u8) -> String {
        let mut payload: Vec<u8> = iter::once(version).chain(self.iter().map(|x| *x)).collect();
        let mut checksum = double_sha256(&payload);
        payload.append(&mut checksum[..4].to_vec());
        payload.to_base58()
    }
}

impl FromBase58Check for str {
    fn from_base58check(&self) -> Result<(u8, Vec<u8>), FromBase58CheckError> {
        let mut payload: Vec<u8> = match self.from_base58() {
            Ok(payload) => payload,
            Err(error) => return Err(FromBase58CheckError::InvalidBase58(error)),
        };
        if payload.len() < 5 {
            return Err(FromBase58CheckError::InvalidChecksum)
        }
        let checksum_index = payload.len() - 4;
        let provided_checksum = payload.split_off(checksum_index);
        let checksum = double_sha256(&payload)[..4].to_vec();
        if checksum != provided_checksum {
            return Err(FromBase58CheckError::InvalidChecksum)
        }
        Ok((payload[0], payload[1..].to_vec()))
    }
}

fn double_sha256(payload: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(&payload);
    let output: Vec<_> = hasher.result().into_iter().collect();

    let mut hasher = Sha256::new();
    hasher.input(&output);
    hasher.result().into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::{ToBase58Check, FromBase58Check, FromBase58CheckError};

    #[test]
    fn to_base58check() {
        assert_eq!(b"".to_base58check(0), "1Wh4bh");
        assert_eq!(b"hello".to_base58check(0), "12L5B5yqsf7vwb");
        assert_eq!(b"hello".to_base58check(1), "5b4vP1wunz2H5");
    }

    #[test]
    fn from_base58check() {
        assert_eq!("1Wh4bh".from_base58check().unwrap(), (0u8, vec![]));
        assert_eq!("12L5B5yqsf7vwb".from_base58check().unwrap(), (0u8, b"hello".to_vec()));
        assert_eq!("5b4vP1wunz2H5".from_base58check().unwrap(), (1u8, b"hello".to_vec()));
    }

    #[test]
    fn from_base58check_with_invalid_checksum() {
        assert_eq!("1Wh4bc".from_base58check(), Err(FromBase58CheckError::InvalidChecksum));
    }

    #[test]
    #[should_panic]
    fn from_base58check_with_invalid_length() {
        "Wh4bh".from_base58check().unwrap();
    }
}
