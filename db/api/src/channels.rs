use hopr_crypto_packet::{HoprSphinxHeaderSpec, HoprSphinxSuite, KeyIdMapper};
use hopr_internal_types::prelude::ChannelEntry;
use hopr_primitive_types::prelude::Address;
use crate::errors::DbError;

#[async_trait::async_trait]
pub trait HoprDbSimpleChannelOperations {
    async fn get_channel_by_parties(&self, source: &Address, destination: &Address) -> Result<Option<ChannelEntry>, DbError>;

    fn key_id_mapper(&self) -> &impl KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>;
}