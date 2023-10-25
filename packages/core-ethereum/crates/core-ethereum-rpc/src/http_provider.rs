

#[cfg(feature = "wasm")]
pub mod wasm {
    use std::fmt::Debug;
    use async_trait::async_trait;
    use ethers_providers::JsonRpcClient;
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    #[derive(Debug)]
    pub struct NodeJsRpcClient {

    }

    #[async_trait(? Send)]
    impl JsonRpcClient for NodeJsRpcClient {
        type Error = ();

        async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(&self, method: &str, params: T) -> Result<R, Self::Error> {
            todo!()
        }
    }
}