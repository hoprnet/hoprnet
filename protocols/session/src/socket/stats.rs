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
