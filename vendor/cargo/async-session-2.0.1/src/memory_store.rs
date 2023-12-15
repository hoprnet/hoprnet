use crate::{async_trait, log, Result, Session, SessionStore};
use async_std::sync::{Arc, RwLock};
use std::collections::HashMap;

/// # in-memory session store
/// Because there is no external
/// persistance, this session store is ephemeral and will be cleared
/// on server restart.
///
/// # ***DO NOT USE THIS IN A PRODUCTION DEPLOYMENT.***
#[derive(Debug, Clone)]
pub struct MemoryStore {
    inner: Arc<RwLock<HashMap<String, Session>>>,
}

#[async_trait]
impl SessionStore for MemoryStore {
    async fn load_session(&self, cookie_value: String) -> Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        log::trace!("loading session by id `{}`", id);
        Ok(self
            .inner
            .read()
            .await
            .get(&id)
            .cloned()
            .and_then(Session::validate))
    }

    async fn store_session(&self, session: Session) -> Result<Option<String>> {
        log::trace!("storing session by id `{}`", session.id());
        self.inner
            .write()
            .await
            .insert(session.id().to_string(), session.clone());

        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> Result {
        log::trace!("destroying session by id `{}`", session.id());
        self.inner.write().await.remove(session.id());
        Ok(())
    }

    async fn clear_store(&self) -> Result {
        log::trace!("clearing memory store");
        self.inner.write().await.clear();
        Ok(())
    }
}

impl MemoryStore {
    /// Create a new instance of MemoryStore
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Performs session cleanup. This should be run on an
    /// intermittent basis if this store is run for long enough that
    /// memory accumulation is a concern
    pub async fn cleanup(&self) -> Result {
        log::trace!("cleaning up memory store...");
        let ids_to_delete: Vec<_> = self
            .inner
            .read()
            .await
            .values()
            .filter_map(|session| {
                if session.is_expired() {
                    Some(session.id().to_owned())
                } else {
                    None
                }
            })
            .collect();

        log::trace!("found {} expired sessions", ids_to_delete.len());
        for id in ids_to_delete {
            self.inner.write().await.remove(&id);
        }
        Ok(())
    }

    /// returns the number of elements in the memory store
    /// # Example
    /// ```rust
    /// # use async_session::{MemoryStore, Session, SessionStore};
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// let mut store = MemoryStore::new();
    /// assert_eq!(store.count().await, 0);
    /// store.store_session(Session::new()).await?;
    /// assert_eq!(store.count().await, 1);
    /// # Ok(()) }) }
    /// ```
    pub async fn count(&self) -> usize {
        let data = self.inner.read().await;
        data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use std::time::Duration;
    #[async_std::test]
    async fn creating_a_new_session_with_no_expiry() -> Result {
        let store = MemoryStore::new();
        let mut session = Session::new();
        session.insert("key", "Hello")?;
        let cloned = session.clone();
        let cookie_value = store.store_session(session).await?.unwrap();
        let loaded_session = store.load_session(cookie_value).await?.unwrap();
        assert_eq!(cloned.id(), loaded_session.id());
        assert_eq!("Hello", &loaded_session.get::<String>("key").unwrap());
        assert!(!loaded_session.is_expired());
        assert!(loaded_session.validate().is_some());
        Ok(())
    }

    #[async_std::test]
    async fn updating_a_session() -> Result {
        let store = MemoryStore::new();
        let mut session = Session::new();

        session.insert("key", "value")?;
        let cookie_value = store.store_session(session).await?.unwrap();

        let mut session = store.load_session(cookie_value.clone()).await?.unwrap();
        session.insert("key", "other value")?;

        assert_eq!(store.store_session(session).await?, None);
        let session = store.load_session(cookie_value).await?.unwrap();
        assert_eq!(&session.get::<String>("key").unwrap(), "other value");

        Ok(())
    }

    #[async_std::test]
    async fn updating_a_session_extending_expiry() -> Result {
        let store = MemoryStore::new();
        let mut session = Session::new();
        session.expire_in(Duration::from_secs(1));
        let original_expires = session.expiry().unwrap().clone();
        let cookie_value = store.store_session(session).await?.unwrap();

        let mut session = store.load_session(cookie_value.clone()).await?.unwrap();

        assert_eq!(session.expiry().unwrap(), &original_expires);
        session.expire_in(Duration::from_secs(3));
        let new_expires = session.expiry().unwrap().clone();
        assert_eq!(None, store.store_session(session).await?);

        let session = store.load_session(cookie_value.clone()).await?.unwrap();
        assert_eq!(session.expiry().unwrap(), &new_expires);

        task::sleep(Duration::from_secs(3)).await;
        assert_eq!(None, store.load_session(cookie_value).await?);

        Ok(())
    }

    #[async_std::test]
    async fn creating_a_new_session_with_expiry() -> Result {
        let store = MemoryStore::new();
        let mut session = Session::new();
        session.expire_in(Duration::from_secs(3));
        session.insert("key", "value")?;
        let cloned = session.clone();

        let cookie_value = store.store_session(session).await?.unwrap();

        let loaded_session = store.load_session(cookie_value.clone()).await?.unwrap();
        assert_eq!(cloned.id(), loaded_session.id());
        assert_eq!("value", &*loaded_session.get::<String>("key").unwrap());

        assert!(!loaded_session.is_expired());

        task::sleep(Duration::from_secs(3)).await;
        assert_eq!(None, store.load_session(cookie_value).await?);

        Ok(())
    }

    #[async_std::test]
    async fn destroying_a_single_session() -> Result {
        let store = MemoryStore::new();
        for _ in 0..3i8 {
            store.store_session(Session::new()).await?;
        }

        let cookie = store.store_session(Session::new()).await?.unwrap();
        assert_eq!(4, store.count().await);
        let session = store.load_session(cookie.clone()).await?.unwrap();
        store.destroy_session(session.clone()).await?;
        assert_eq!(None, store.load_session(cookie).await?);
        assert_eq!(3, store.count().await);

        // attempting to destroy the session again is not an error
        assert!(store.destroy_session(session).await.is_ok());
        Ok(())
    }

    #[async_std::test]
    async fn clearing_the_whole_store() -> Result {
        let store = MemoryStore::new();
        for _ in 0..3i8 {
            store.store_session(Session::new()).await?;
        }

        assert_eq!(3, store.count().await);
        store.clear_store().await.unwrap();
        assert_eq!(0, store.count().await);

        Ok(())
    }
}
