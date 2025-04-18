use hopr_crypto_types::prelude::SimplePseudonym;
use hopr_internal_types::protocol::ApplicationData;
use hopr_network_types::prelude::RoutingOptions;
use hopr_primitive_types::prelude::Address;

use crate::errors::TransportSessionError;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SendMsg {
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: Address,
        forward_options: RoutingOptions,
        return_options: Option<RoutingOptions>,
        pseudonym: Option<SimplePseudonym>,
    ) -> std::result::Result<(), TransportSessionError>;
}
