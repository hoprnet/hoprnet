pub use crate::protocol::SessionMessageDiscriminants;

/// Used to track various statistics of a [`SessionSocket`](crate::SessionSocket).
#[auto_impl::auto_impl(&, Arc)]
pub trait SessionTelemetryTracker {
    /// Records a frame that has been emitted from the Sequencer.
    fn frame_emitted(&self);
    /// Records a frame that has successfully reassembled by the Reassembler.
    fn frame_completed(&self);
    /// Records a frame that has been discarded due to timeout or other errors.
    fn frame_discarded(&self);
    /// Records `count` consecutive frames that have been discarded at once.
    ///
    /// The Sequencer gives up on a whole gap in one step, and a gap can span up to 2^32 frame ids, so
    /// an implementation should override this to account for them in one step as well. The default
    /// records each frame with [`frame_discarded`](Self::frame_discarded), one by one.
    fn frames_discarded(&self, count: u32) {
        for _ in 0..count {
            self.frame_discarded();
        }
    }
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

impl SessionTelemetryTracker for NoopTracker {
    fn frame_emitted(&self) {}

    fn frame_completed(&self) {}

    fn frame_discarded(&self) {}

    fn frames_discarded(&self, _: u32) {}

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
    pub struct TestTelemetryTracker(std::sync::Arc<Mutex<indexmap::IndexMap<String, usize>>>);

    impl Serialize for TestTelemetryTracker {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.lock().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for TestTelemetryTracker {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let map = indexmap::IndexMap::deserialize(deserializer)?;
            Ok(Self(std::sync::Arc::new(Mutex::new(map))))
        }
    }

    impl Default for TestTelemetryTracker {
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

    impl TestTelemetryTracker {
        fn increment(&self, key: &str) {
            self.increment_by(key, 1);
        }

        fn increment_by(&self, key: &str, by: usize) {
            let mut map = self.0.lock();
            *map.entry(key.to_string()).or_insert(0) += by;
        }
    }

    impl SessionTelemetryTracker for TestTelemetryTracker {
        fn frame_emitted(&self) {
            self.increment("frames_emitted");
        }

        fn frame_completed(&self) {
            self.increment("frames_completed");
        }

        fn frame_discarded(&self) {
            self.increment("frames_discarded");
        }

        fn frames_discarded(&self, count: u32) {
            self.increment_by("frames_discarded", count as usize);
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

    /// A tracker written before runs of discarded frames existed: it only knows `frame_discarded`.
    #[derive(Default)]
    struct PerFrameTracker(std::sync::atomic::AtomicU32);

    impl SessionTelemetryTracker for PerFrameTracker {
        fn frame_emitted(&self) {}

        fn frame_completed(&self) {}

        fn frame_discarded(&self) {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        fn incomplete_frame(&self) {}

        fn incoming_message(&self, _: SessionMessageDiscriminants) {}

        fn outgoing_message(&self, _: SessionMessageDiscriminants) {}

        fn error(&self) {}
    }

    #[test]
    fn a_tracker_without_its_own_run_handling_should_see_every_discarded_frame() {
        let tracker = PerFrameTracker::default();

        tracker.frames_discarded(0);
        assert_eq!(0, tracker.0.load(std::sync::atomic::Ordering::Relaxed));

        tracker.frames_discarded(5);
        assert_eq!(5, tracker.0.load(std::sync::atomic::Ordering::Relaxed));
    }
}
