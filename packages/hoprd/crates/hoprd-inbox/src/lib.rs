pub mod config;
pub mod inbox;
pub mod ring;

pub type Inbox =
    inbox::MessageInbox<ring::RingBufferInboxBackend<core_types::protocol::Tag, core_types::protocol::ApplicationData>>;
