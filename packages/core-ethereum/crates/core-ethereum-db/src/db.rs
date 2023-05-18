use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use utils_db::{db::DB,traits::BinaryAsyncKVStorage};

use crate::errors::Result;
use crate::traits::HoprCoreEthereumDbActions;

pub struct CoreEthereumDb<T>
where
    T: BinaryAsyncKVStorage,
{
    db: DB<T>,
    // me: PublicKey,
}

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
impl<T: BinaryAsyncKVStorage> HoprCoreEthereumDbActions for CoreEthereumDb<T> {
    async fn abc(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn build_core_ethereum_db(_db: utils_db::leveldb::LevelDb) -> JsValue {
        // TODO: build core db
        JsValue::undefined()
    }
}

#[cfg(test)]
mod tests {
    use utils_db::db::serialize_to_bytes;
    use utils_types::primitives::EthereumChallenge;
    use utils_types::traits::BinarySerializable;

    #[test]
    fn test_core_ethereum_db_iterable_type_EhtereumChallenge_must_have_fixed_key_length() {
        let challenge = vec![10u8; EthereumChallenge::SIZE];
        let eth_challenge = EthereumChallenge::new(challenge.as_slice());

        let serialized = serialize_to_bytes(&eth_challenge);

        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().len(), EthereumChallenge::SIZE)
    }
}
