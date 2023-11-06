use core_transport::Hash;
use utils_db::{db::DB, traits::AsyncKVStorage};
use utils_types::primitives::AuthorizationToken;

pub const API_AUTHORIZATION_TOKEN_KEY_PREFIX: &str = "api:authenticationTokens";

#[derive(Clone)]
pub struct HoprdPersistentDb<T>
where
    T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>> + Clone,
{
    pub db: DB<T>,
}

impl<T: AsyncKVStorage<Key = Box<[u8]>, Value = Box<[u8]>> + Clone> HoprdPersistentDb<T> {
    pub fn new(db: DB<T>) -> Self {
        Self { db }
    }

    async fn store_authorization(&mut self, token: AuthorizationToken) -> crate::errors::Result<()> {
        let tid = Hash::create(&[token.id().as_bytes()]);
        let key = utils_db::db::Key::new_with_prefix(&tid, API_AUTHORIZATION_TOKEN_KEY_PREFIX)?;
        let _ = self.db.set(key, &token).await?;
        Ok(())
    }

    async fn retrieve_authorization(&self, id: String) -> crate::errors::Result<Option<AuthorizationToken>> {
        let tid = Hash::create(&[id.as_bytes()]);
        let key = utils_db::db::Key::new_with_prefix(&tid, API_AUTHORIZATION_TOKEN_KEY_PREFIX)?;
        Ok(self.db.get_or_none::<AuthorizationToken>(key).await?)
    }

    async fn delete_authorization(&mut self, id: String) -> crate::errors::Result<()> {
        let tid = Hash::create(&[id.as_bytes()]);
        let key = utils_db::db::Key::new_with_prefix(&tid, API_AUTHORIZATION_TOKEN_KEY_PREFIX)?;
        let _ = self.db.remove::<AuthorizationToken>(key).await?;
        Ok(())
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use async_lock::RwLock;
    use std::sync::Arc;
    use utils_db::rusty::RustyLevelDbShim;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct HoprdPersistentDatabase {
        inner: Arc<RwLock<HoprdPersistentDb<RustyLevelDbShim>>>,
    }

    #[wasm_bindgen]
    impl HoprdPersistentDatabase {
        #[wasm_bindgen(constructor)]
        pub fn _new(path: &str) -> Self {
            Self {
                inner: Arc::new(RwLock::new(HoprdPersistentDb::new(DB::new(
                    utils_db::rusty::RustyLevelDbShim::new(path, true),
                )))),
            }
        }

        #[wasm_bindgen(js_name = newInMemory)]
        pub fn _new_in_memory() -> Self {
            Self {
                inner: Arc::new(RwLock::new(HoprdPersistentDb::new(DB::new(
                    utils_db::rusty::RustyLevelDbShim::new_in_memory(),
                )))),
            }
        }

        #[wasm_bindgen]
        pub async fn store_authorization(&self, token: AuthorizationToken) -> Result<(), JsError> {
            let data = self.inner.clone();
            let mut db = data.write().await;
            db.store_authorization(token).await.map_err(JsError::from)
        }

        #[wasm_bindgen]
        pub async fn retrieve_authorization(&self, id: String) -> Result<Option<AuthorizationToken>, JsError> {
            let data = self.inner.clone();
            let db = data.read().await;
            db.retrieve_authorization(id).await.map_err(JsError::from)
        }

        #[wasm_bindgen]
        pub async fn delete_authorization(&self, id: String) -> Result<(), JsError> {
            let data = self.inner.clone();
            let mut db = data.write().await;
            db.delete_authorization(id).await.map_err(JsError::from)
        }
    }
}

#[cfg(test)]
mod tests {
    use utils_db::rusty::RustyLevelDbShim;

    use super::*;

    #[async_std::test]
    async fn test_token_storage() {
        let mut db = HoprdPersistentDb::new(DB::new(RustyLevelDbShim::new_in_memory()));

        let token_id = "test";

        let token = AuthorizationToken::new(token_id.to_string(), &[0xffu8; 100]);
        db.store_authorization(token.clone()).await.unwrap();

        let token_2 = db
            .retrieve_authorization(token_id.to_string())
            .await
            .unwrap()
            .expect("db should contain a token");
        assert_eq!(token, token_2, "retrieved token should be equal to the stored one");

        db.delete_authorization(token_id.to_string())
            .await
            .expect("db should remove token");

        let nonexistent = db.retrieve_authorization(token_id.to_string()).await.unwrap();
        assert!(nonexistent.is_none(), "token should be removed from the db");
    }
}
