use crate::{async_trait, Result, Session, SessionStore};

/// A session store that serializes the entire session into a Cookie.
///
/// # ***This is not recommended for most production deployments.***
///
/// This implementation uses [`bincode`](::bincode) to serialize the
/// Session to decrease the size of the cookie. Note: There is a
/// maximum of 4093 cookie bytes allowed _per domain_, so the cookie
/// store is limited in capacity.
///
/// **Note:** Currently, the data in the cookie is only signed, but *not
/// encrypted*. If the contained session data is sensitive and
/// should not be read by a user, the cookie store is not an
/// appropriate choice.
///
/// Expiry: `SessionStore::destroy_session` and
/// `SessionStore::clear_store` are not meaningful for the
/// CookieStore, and noop. Destroying a session must be done at the
/// cookie setting level, which is outside of the scope of this crate.

#[derive(Debug, Clone, Copy)]
pub struct CookieStore;

impl CookieStore {
    /// constructs a new CookieStore
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SessionStore for CookieStore {
    async fn load_session(&self, cookie_value: String) -> Result<Option<Session>> {
        let serialized = base64::decode(&cookie_value)?;
        let session: Session = bincode::deserialize(&serialized)?;
        Ok(session.validate())
    }

    async fn store_session(&self, session: Session) -> Result<Option<String>> {
        let serialized = bincode::serialize(&session)?;
        Ok(Some(base64::encode(serialized)))
    }

    async fn destroy_session(&self, _session: Session) -> Result {
        Ok(())
    }

    async fn clear_store(&self) -> Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use std::time::Duration;
    #[async_std::test]
    async fn creating_a_new_session_with_no_expiry() -> Result {
        let store = CookieStore::new();
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
        let store = CookieStore::new();
        let mut session = Session::new();

        session.insert("key", "value")?;
        let cookie_value = store.store_session(session).await?.unwrap();

        let mut session = store.load_session(cookie_value.clone()).await?.unwrap();
        session.insert("key", "other value")?;

        let new_cookie_value = store.store_session(session).await?.unwrap();
        let session = store.load_session(new_cookie_value).await?.unwrap();
        assert_eq!(&session.get::<String>("key").unwrap(), "other value");

        Ok(())
    }

    #[async_std::test]
    async fn updating_a_session_extending_expiry() -> Result {
        let store = CookieStore::new();
        let mut session = Session::new();
        session.expire_in(Duration::from_secs(1));
        let original_expires = session.expiry().unwrap().clone();
        let cookie_value = store.store_session(session).await?.unwrap();

        let mut session = store.load_session(cookie_value.clone()).await?.unwrap();

        assert_eq!(session.expiry().unwrap(), &original_expires);
        session.expire_in(Duration::from_secs(3));
        let new_expires = session.expiry().unwrap().clone();
        let cookie_value = store.store_session(session).await?.unwrap();

        let session = store.load_session(cookie_value.clone()).await?.unwrap();
        assert_eq!(session.expiry().unwrap(), &new_expires);

        task::sleep(Duration::from_secs(3)).await;
        assert_eq!(None, store.load_session(cookie_value).await?);

        Ok(())
    }

    #[async_std::test]
    async fn creating_a_new_session_with_expiry() -> Result {
        let store = CookieStore::new();
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
}
