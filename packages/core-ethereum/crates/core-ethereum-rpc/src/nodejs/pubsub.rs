use ethers_providers::PubsubClient;
use primitive_types::U256;
use serde_json::value::RawValue;
use crate::nodejs::NodeJsRpcClient;

impl PubsubClient for NodeJsRpcClient {
    type NotificationStream = futures::channel::mpsc::UnboundedReceiver<Box<RawValue>>;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error> {
        todo!()
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
        todo!()
    }
}