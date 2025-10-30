use futures::Stream;
use hopr_api::chain::ChainEvent;
use crate::connector::{HoprBlockchainConnector};
use crate::errors::ConnectorError;

impl<B, C, P> hopr_api::chain::ChainEvents for  HoprBlockchainConnector<B, C, P> {
    type Error = ConnectorError;

    fn subscribe(&self) -> Result<impl Stream<Item=ChainEvent> + Send + 'static, Self::Error> {
        Ok(self.events.as_ref().1.activate_cloned())
    }
}