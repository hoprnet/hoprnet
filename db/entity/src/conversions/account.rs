use std::str::FromStr;

use hopr_crypto_types::types::OffchainPublicKey;
use hopr_primitive_types::{primitives::Address, traits::ToHex};

impl TryFrom<crate::codegen::sqlite::account::Model> for Address {
    type Error = crate::errors::DbEntityError;

    fn try_from(value: crate::codegen::sqlite::account::Model) -> std::result::Result<Self, Self::Error> {
        Ok(Address::from_str(&value.chain_key).map_err(|e| Self::Error::ConversionError(format!("{e}")))?)
    }
}

impl TryFrom<crate::codegen::sqlite::account::Model> for OffchainPublicKey {
    type Error = crate::errors::DbEntityError;

    fn try_from(value: crate::codegen::sqlite::account::Model) -> std::result::Result<Self, Self::Error> {
        Ok(OffchainPublicKey::from_hex(&value.packet_key).map_err(|e| Self::Error::ConversionError(format!("{e}")))?)
    }
}
