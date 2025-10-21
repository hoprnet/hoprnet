use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_primitive_types::prelude::Address;
use crate::connector::{HoprBlockchainConnector, OnchainDataStorage};
use crate::errors::ConnectorError;

#[async_trait::async_trait]
impl<T: OnchainDataStorage + Send + Sync> hopr_api::chain::ChainKeyOperations for HoprBlockchainConnector<T> {
    type Error = ConnectorError;
    type Mapper = ();

    async fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error> {
        todo!()
    }

    async fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error> {
        todo!()
    }

    fn key_id_mapper_ref(&self) -> &Self::Mapper {
        todo!()
    }
}