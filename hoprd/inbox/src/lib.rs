//! ## hoprd-inbox
//! This crate implements the functionality of the Message Inbox (MI) backend.
//!
//! The MI works as a non-persistent storage of received HOPR packets categorized by the application tag.
//! The application tag works as a distinguisher of applications running on top of the HOPR protocol, similarly like
//! ports work in TCP or UDP.
//! Application tag is [represented](hopr_internal_types::protocol::Tag) by a 16-bit payload prefix and is built-in the HOPR protocol.
//! If no tag is given, it defaults to `0`.
//!
//! The MI can use different backends, which must implement the [InboxBackend](inbox::InboxBackend) trait.
//!
//! ## How `InboxBackend` works
//! The backend must ensure that both tagged and untagged messages can be `push`ed and `pop`ed to/from it.
//! Backend can be persistent, but the current [RingBufferInboxBackend](ring::RingBufferInboxBackend) is in-memory only.
//!
//! The backend must have a finite capacity of messages, overwriting the oldest elements on `push` if the total capacity is reached.
//! Messages can be `pop`ed by one (oldest first), or can be drained entirely using `pop_all` (sorted by oldest to latest).
//! If application tag is specified on a `pop`/`pop_all`, only the messages of the given tag are retrieved. If no tag is
//! specified, the messages are retrieved regardless the tag (`pop` retrieves the oldest message across all tags, `pop_all`
//! retrieves all the messages from the entire backend sorted oldest to latest).
//!
//! Finally, the backends must hold some sort of timestamp information of when each message was `push`ed. That's because
//! the backend must also support a `purge` operation which removes messages older than the given Unix timestamp.
//!
//! ## The Frontend
//! The MI has a front-end type [MessageInbox](inbox::MessageInbox), which is thin thread-safe wrapper over a selected backend. It contains choices
//! of specific parameters: tag type is fixed as a 16-bit unsigned integer to correspond with `core-packet`, the message
//! type is set to [ApplicationData](hopr_internal_types::protocol::ApplicationData) type.
//!
//! The `MessageInbox` currently uses the `RingBufferInboxBacked` as its `InboxBackend` implementations.
//! This backend is implemented as hash map (with application tag as a key) of ring-buffers of certain capacity `N`.
//! Each bucket can therefore hold `N` messages which can be `pop`ed from oldest to newest.
//! The maximum number of messages held in this instantiation of MI is therefore 65536 \* `N`
//! (unless some tags are excluded). Note that in this implementation `N` must be a power of 2.
//!
//! Each `push` and `pop` operation is always followed by a `purge` call to evict expired entries.
//!
//! The frontend has the ability to filter out certain application tags on `push`, so that they are ignored by the MI.
//!
//! ## Usage
//!
//! The [Inbox] instance is supposed to live a singleton in the `hoprd` application, and as messages arrive, they
//! will be pushed into the inbox. The REST API can then access this singleton to pop messages per request.
//!
//! For details on default configuration see [MessageInboxConfiguration](crate::config::MessageInboxConfiguration).

/// Contains configuration object for [MessageInbox](inbox::MessageInbox).
pub mod config;
/// Defines the common traits and the [MessageInbox](inbox::MessageInbox) type.
pub mod inbox;
/// Implements ring-buffer based in-memory [InboxBackend](inbox::InboxBackend).
pub mod ring;

/// Alias type for message inbox.
pub type Inbox = inbox::MessageInbox<
    ring::RingBufferInboxBackend<hopr_internal_types::protocol::Tag, hopr_internal_types::protocol::ApplicationData>,
>;
