#[cfg(any(feature = "wasm", test))]
use {crate::errors::Result, core_ethereum_db::traits::HoprCoreEthereumDbActions, utils_types::primitives::Address};

#[cfg(any(feature = "wasm", test))]
pub async fn is_allowed_to_access_network<T>(db: &T, chain_address: &Address) -> Result<bool>
where
    T: HoprCoreEthereumDbActions,
{
    let nr_enabled = db.is_network_registry_enabled().await?;

    if !nr_enabled {
        return Ok(true);
    }

    db.is_allowed_to_access_network(chain_address)
        .await
        .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use hex_literal::hex;
    use std::sync::{Arc, Mutex};
    use utils_db::{db::DB, leveldb::rusty::RustyLevelDbShim};
    use utils_types::{
        primitives::{Address, Snapshot},
        traits::BinarySerializable,
    };

    const ADDR: [u8; Address::SIZE] = hex!("bc6f4c25d5d90906aeb1f2eafcfe90dff79319be");
    const TEST_ADDR: [u8; Address::SIZE] = hex!("43699e2486f10b96ebbd251362ddc166177a06db");
    const TEST_ACCOUNT: [u8; Address::SIZE] = hex!("3a585656b8bbb14e8aebf89256ce4511fa35ac33");

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            Address::from_bytes(&ADDR).unwrap(),
        )
    }

    #[async_std::test]
    async fn test_is_allowed_to_access_network() {
        let mut db = create_mock_db();

        db.set_network_registry(true, &Snapshot::default()).await.unwrap();

        db.set_eligible(&Address::from_bytes(&TEST_ADDR).unwrap(), true, &Snapshot::default())
            .await
            .unwrap();

        db.add_to_network_registry(
            &Address::from_bytes(&TEST_ADDR).unwrap(),
            &Address::from_bytes(&TEST_ACCOUNT).unwrap(),
            &Snapshot::default(),
        )
        .await
        .unwrap();

        let is_allowed = super::is_allowed_to_access_network(&db, &Address::from_bytes(&TEST_ACCOUNT).unwrap()).await;

        assert!(is_allowed.is_ok(), "error while checking access in NR");
        assert!(is_allowed.unwrap(), "should be allowed access");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_ethereum_db::db::wasm::Database;
    use utils_misc::ok_or_jserr;
    use utils_types::primitives::Address;
    use wasm_bindgen::{prelude::*, JsValue};

    #[wasm_bindgen]
    pub async fn is_allowed_to_access_network(db: &Database, chain_address: &Address) -> Result<bool, JsValue> {
        let val = db.as_ref_counted();
        let g = val.read().await;
        ok_or_jserr!(super::is_allowed_to_access_network(&*g, chain_address).await)
    }
}
