use sea_orm::Set;

use hopr_crypto_types::types::Hash;
use hopr_primitive_types::prelude::*;

use crate::errors::DbEntityError;
use crate::{log, log_status};

impl From<SerializableLog> for log::ActiveModel {
    fn from(value: SerializableLog) -> Self {
        log::ActiveModel {
            address: Set(value.address.as_ref().to_vec()),
            topics: Set(value.topics.into_iter().flatten().collect()),
            data: Set(value.data),
            block_number: Set(value.block_number.to_be_bytes().to_vec()),
            transaction_hash: Set(value.tx_hash.to_vec()),
            transaction_index: Set(value.tx_index.to_be_bytes().to_vec()),
            block_hash: Set(value.block_hash.to_vec()),
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
        let address = Address::new(value.address.as_ref());

        let mut topics: Vec<[u8; 32]> = Vec::new();
        for chunk in value.topics.chunks_exact(32) {
            let mut topic = [0u8; 32];
            topic.copy_from_slice(chunk);
            topics.push(topic);
        }

        let log = SerializableLog {
            address,
            topics,
            data: value.data,
            block_number: U256::from_be_bytes(value.block_number).as_u64(),
            tx_hash,
            tx_index: U256::from_be_bytes(value.transaction_index).as_u64(),
            block_hash,
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
