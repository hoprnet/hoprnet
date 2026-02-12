pub use crate::protocol::SessionMessageDiscriminants;

/// Used to track various statistics of a [`SessionSocket`](crate::SessionSocket).
#[auto_impl::auto_impl(&, Arc)]
pub trait SessionStatisticsTracker {
    /// Records a frame that has been emitted from the Sequencer.
    fn frame_emitted(&self);
    /// Records a frame that has successfully reassembled by the Reassembler.
    fn frame_completed(&self);
    /// Records a frame that has been discarded due to timeout or other errors.
    fn frame_discarded(&self);
    /// Records an incomplete frame that could not be reassembled.
    fn incomplete_frame(&self);
    /// Records an incoming Session message.
    fn incoming_message(&self, msg: SessionMessageDiscriminants);
    /// Records an outgoing Session message.
    fn outgoing_message(&self, msg: SessionMessageDiscriminants);
    /// Records an error that occurred during processing of a Session packet.
    fn error(&self);
}

/// Session socket statistics tracker that does nothing.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct NoopTracker;

impl SessionStatisticsTracker for NoopTracker {
    fn frame_emitted(&self) {}

    fn frame_completed(&self) {}

    fn frame_discarded(&self) {}

    fn incomplete_frame(&self) {}

    fn incoming_message(&self, _: SessionMessageDiscriminants) {}

    fn outgoing_message(&self, _: SessionMessageDiscriminants) {}

    fn error(&self) {}
}

#[cfg(test)]
pub mod tests {
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};

    use super::*;

    /// Statistics tracker that records all events in a map and is possible to serialize (e.g.: for snapshot testing)
    #[derive(Debug, Clone)]
    pub struct TestStatsTracker(std::sync::Arc<Mutex<indexmap::IndexMap<String, usize>>>);

    impl Serialize for TestStatsTracker {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.lock().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for TestStatsTracker {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let map = indexmap::IndexMap::deserialize(deserializer)?;
            Ok(Self(std::sync::Arc::new(Mutex::new(map))))
        }
    }

    impl Default for TestStatsTracker {
        fn default() -> Self {
            let mut map = indexmap::IndexMap::new();
            map.insert("frames_emitted".into(), 0);
            map.insert("frames_completed".into(), 0);
            map.insert("frames_discarded".into(), 0);
            map.insert("incomplete_frames".into(), 0);
            map.insert("errors".into(), 0);
            map.insert("incoming_messages_Segment".into(), 0);
            map.insert("incoming_messages_Request".into(), 0);
            map.insert("incoming_messages_Acknowledge".into(), 0);
            map.insert("outgoing_messages_Segment".into(), 0);
            map.insert("outgoing_messages_Request".into(), 0);
            map.insert("outgoing_messages_Acknowledge".into(), 0);

            Self(std::sync::Arc::new(Mutex::new(map)))
        }
    }

    impl TestStatsTracker {
        fn increment(&self, key: &str) {
            let mut map = self.0.lock();
            *map.entry(key.to_string()).or_insert(0) += 1;
        }
    }

    impl SessionStatisticsTracker for TestStatsTracker {
        fn frame_emitted(&self) {
            self.increment("frames_emitted");
        }

        fn frame_completed(&self) {
            self.increment("frames_completed");
        }

        fn frame_discarded(&self) {
            self.increment("frames_discarded");
        }

        fn incomplete_frame(&self) {
            self.increment("incomplete_frames");
        }

        fn incoming_message(&self, msg: SessionMessageDiscriminants) {
            self.increment(&format!("incoming_messages_{:?}", msg));
        }

        fn outgoing_message(&self, msg: SessionMessageDiscriminants) {
            self.increment(&format!("outgoing_messages_{:?}", msg));
        }

        fn error(&self) {
            self.increment("errors");
        }
    }
}
