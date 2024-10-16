use sea_orm::Set;

use hopr_crypto_types::types::Hash;
use hopr_primitive_types::prelude::*;

use crate::errors::DbEntityError;
use crate::{log, log_status};

impl From<SerializableLog> for log::ActiveModel {
    fn from(value: SerializableLog) -> Self {
        let tx_hash = value.tx_hash;
        let block_hash = value.block_hash;

        log::ActiveModel {
            address: Set(value.address),
            topics: Set(value.topics.join(",")),
            data: Set(value.data),
            block_number: Set(value.block_number.to_be_bytes().to_vec()),
            transaction_hash: Set(Hash::from_hex(tx_hash.clone().as_str())
                .expect(format!("invalid tx_hash {tx_hash}").as_str())
                .as_ref()
                .to_vec()),
            transaction_index: Set(value.tx_index.to_be_bytes().to_vec()),
            block_hash: Set(Hash::from_hex(block_hash.clone().as_str())
                .expect(format!("invalid block_hash {block_hash}").as_str())
                .as_ref()
                .to_vec()),
            log_index: Set(value.log_index.to_be_bytes().to_vec()),
            removed: Set(value.removed),
            ..Default::default()
        }
    }
}

impl TryFrom<log::Model> for SerializableLog {
    type Error = DbEntityError;

    fn try_from(value: log::Model) -> Result<Self, Self::Error> {
        let tx_hash: [u8; 32] = value
            .transaction_hash
            .try_into()
            .map_err(|_| DbEntityError::ConversionError("Invalid tx_hash".into()))?;
        let block_hash: [u8; 32] = value
            .block_hash
            .try_into()
            .map_err(|_| DbEntityError::ConversionError("Invalid tx_hash".into()))?;

        let log = SerializableLog {
            address: value.address,
            topics: value.topics.split(",").map(|s| s.to_string()).collect(),
            data: value.data,
            block_number: U256::from_be_bytes(value.block_number).as_u64(),
            tx_hash: Hash::from(tx_hash).to_hex(),
            tx_index: U256::from_be_bytes(value.transaction_index).as_u64(),
            block_hash: Hash::from(block_hash).to_hex(),
            log_index: U256::from_be_bytes(value.log_index).as_u64(),
            removed: value.removed,
            ..Default::default()
        };

        Ok(log)
    }
}

impl From<SerializableLog> for log_status::ActiveModel {
    fn from(value: SerializableLog) -> Self {
        let processed = value.processed.map_or(false, |p| p);
        let processed_at = value.processed_at.map_or(None, |p| Some(p.naive_utc()));
        let checksum = value
            .checksum
            .map(|c| Hash::from_hex(&c).expect("Invalid checksum").as_ref().to_vec());

        log_status::ActiveModel {
            block_number: Set(value.block_number.to_be_bytes().to_vec()),
            transaction_index: Set(value.tx_index.to_be_bytes().to_vec()),
            log_index: Set(value.log_index.to_be_bytes().to_vec()),
            processed: Set(processed),
            processed_at: Set(processed_at),
            checksum: Set(checksum),
        }
    }
}
