use futures::Stream;
use hopr_api::chain::ChainEvent;

use crate::{connector::HoprBlockchainConnector, errors::ConnectorError};

impl<B, C, P, R> hopr_api::chain::ChainEvents for HoprBlockchainConnector<C, B, P, R> {
    type Error = ConnectorError;

    fn subscribe(&self) -> Result<impl Stream<Item = ChainEvent> + Send + 'static, Self::Error> {
        self.check_connection_state()?;

        Ok(self.events.1.activate_cloned())
    }
}
