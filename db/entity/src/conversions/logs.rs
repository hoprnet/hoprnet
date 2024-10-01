use sea_orm::Set;

use hopr_crypto_types::types::Hash;
use hopr_primitive_types::prelude::*;

use crate::{log, log_status};

impl From<SerializableLog> for log::ActiveModel {
    fn from(value: SerializableLog) -> Self {
        log::ActiveModel {
            address: Set(value.address),
            topics: Set(value.topics.join(",")),
            data: Set(value.data),
            block_number: Set(value.block_number.to_be_bytes().to_vec()),
            transaction_hash: Set(value.tx_hash),
            transaction_index: Set(value.tx_index.to_be_bytes().to_vec()),
            block_hash: Set(value.block_hash),
            log_index: Set(value.log_index.to_be_bytes().to_vec()),
            removed: Set(value.removed),
            ..Default::default()
        }
    }
}

impl From<log::Model> for SerializableLog {
    fn from(value: log::Model) -> Self {
        SerializableLog {
            address: value.address,
            topics: value.topics.split(",").map(|s| s.to_string()).collect(),
            data: value.data,
            block_number: U256::from_be_bytes(value.block_number).as_u64(),
            tx_hash: value.transaction_hash,
            tx_index: U256::from_be_bytes(value.transaction_index).as_u64(),
            block_hash: value.block_hash,
            log_index: U256::from_be_bytes(value.log_index).as_u64(),
            removed: value.removed,
            ..Default::default()
        }
    }
}

impl From<SerializableLog> for log_status::ActiveModel {
    fn from(value: SerializableLog) -> Self {
        let processed = value.processed.map_or(false, |p| p);
        let processed_at = value.processed_at.map_or(None, |p| Some(p.naive_utc()));
        let checksum = value
            .checksum
            .map(|c| Hash::from_hex(c.as_str()).expect("Invalid checksum"))
            .map(|c| c.as_ref().to_vec());

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
