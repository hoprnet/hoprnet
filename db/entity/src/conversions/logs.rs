use sea_orm::Set;

use hopr_primitive_types::prelude::*;

use crate::errors::DbEntityError;
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

impl TryFrom<&log::Model> for SerializableLog {
    type Error = DbEntityError;

    fn try_from(value: &log::Model) -> Result<Self, Self::Error> {
        let log = SerializableLog {
            address: value.address.clone(),
            topics: value.topics.split(",").map(|s| s.to_string()).collect(),
            data: value.data.clone(),
            block_number: U256::from_be_bytes(&value.block_number).as_u64(),
            tx_hash: value.transaction_hash.clone(),
            tx_index: U256::from_be_bytes(&value.transaction_index).as_u64(),
            block_hash: value.block_hash.clone(),
            log_index: U256::from_be_bytes(&value.log_index).as_u64(),
            removed: value.removed,
        };
        Ok(log)
    }
}

impl TryFrom<log::Model> for SerializableLog {
    type Error = DbEntityError;

    fn try_from(value: log::Model) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl From<SerializableLog> for log_status::ActiveModel {
    fn from(value: SerializableLog) -> Self {
        log_status::ActiveModel {
            block_number: Set(value.block_number.to_be_bytes().to_vec()),
            transaction_index: Set(value.tx_index.to_be_bytes().to_vec()),
            log_index: Set(value.log_index.to_be_bytes().to_vec()),
            ..Default::default()
        }
    }
}
