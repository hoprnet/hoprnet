use std::marker::PhantomData;
use std::time::Duration;
use async_lock::{Mutex};
use async_trait::async_trait;

pub type Tag = u16;
const RESERVED_TAG: Tag = 0;

#[async_trait(?Send)]
pub trait InboxBackend<T> {
    fn new_with_capacity(cap: usize) -> Self;
    async fn push(&mut self, tag: Tag, payload: T);
    async fn count(&self, tag: Option<Tag>) -> usize;
    async fn pop(&mut self, tag: Option<Tag>) -> Option<T>;
    async fn pop_all(&mut self, tag: Option<Tag>) -> Vec<T>;
    async fn purge(&mut self, older_than_ts: u64);
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct MessageInboxConfiguration {
    capacity: u32,
    max_age_millis: u64,
    excluded_tags: Vec<Tag>,
}

impl Default for MessageInboxConfiguration {
    fn default() -> Self {
        Self {
            capacity: 1024,
            max_age_millis: 15*60000_u64,
            excluded_tags: vec![]
        }
    }
}

pub struct MessageInbox<P, B>
where P: AsRef<[u8]>, B: InboxBackend<P> {
    cfg: MessageInboxConfiguration,
    backend: Mutex<B>,
    time: fn() -> Duration,
    _data: PhantomData<P>
}

impl<P,B> MessageInbox<P, B>
where P: AsRef<[u8]>, B: InboxBackend<P> {
    #[cfg(any(not(feature = "wasm"), test))]
    pub fn new(cfg: MessageInboxConfiguration) -> Self {
        Self::new_with_time(cfg, || std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap())
    }

    pub fn new_with_time(cfg: MessageInboxConfiguration, time: fn() -> Duration) -> Self {
        Self {
            backend: Mutex::new(B::new_with_capacity(cfg.capacity as usize)),
            time, cfg, _data: PhantomData
        }
    }

    fn extract_tag(payload: &P) -> Option<Tag> {
        // TODO: specify application-layer format
        let data: [u8; 2] = payload.as_ref()[0..2].try_into().ok()?;
        Some(Tag::from_be_bytes(data))
    }

    pub async fn push(&mut self, payload: P) {
        let tag = Self::extract_tag(&payload);
        let mut db = self.backend.lock().await;
        db.push(tag.unwrap_or(RESERVED_TAG), payload)
            .await;
        db.purge((self.time)().as_secs() - self.cfg.max_age_millis).await;
    }

    pub async fn size(&self, tag: Option<Tag>) -> usize {
        self.backend.lock().await.count(tag).await
    }

    pub async fn pop(&mut self, tag: Option<Tag>) -> Option<P> {
        let mut db = self.backend.lock().await;
        db.purge((self.time)().as_secs() - self.cfg.max_age_millis).await;
        db.pop(tag).await
    }

    pub async fn pop_all(&mut self, tag: Option<Tag>) -> Vec<P> {
        let mut db = self.backend.lock().await;
        db.purge((self.time)().as_secs() - self.cfg.max_age_millis).await;
        db.pop_all(tag).await
    }
}

#[cfg(test)]
mod tests {

}

#[cfg(feature = "wasm")]
pub mod wasm {

}