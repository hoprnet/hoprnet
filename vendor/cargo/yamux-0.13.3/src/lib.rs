// Copyright (c) 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 or MIT license, at your option.
//
// A copy of the Apache License, Version 2.0 is included in the software as
// LICENSE-APACHE and a copy of the MIT license is included in the software
// as LICENSE-MIT. You may also obtain a copy of the Apache License, Version 2.0
// at https://www.apache.org/licenses/LICENSE-2.0 and a copy of the MIT license
// at https://opensource.org/licenses/MIT.

//! This crate implements the [Yamux specification][1].
//!
//! It multiplexes independent I/O streams over reliable, ordered connections,
//! such as TCP/IP.
//!
//! The two primary objects, clients of this crate interact with, are:
//!
//! - [`Connection`], which wraps the underlying I/O resource, e.g. a socket, and
//!   provides methods for opening outbound or accepting inbound streams.
//! - [`Stream`], which implements [`futures::io::AsyncRead`] and
//!   [`futures::io::AsyncWrite`].
//!
//! [1]: https://github.com/hashicorp/yamux/blob/master/spec.md

#![forbid(unsafe_code)]

mod chunks;
mod error;
mod frame;

pub(crate) mod connection;
mod tagged_stream;

pub use crate::connection::{Connection, Mode, Packet, Stream};
pub use crate::error::ConnectionError;
pub use crate::frame::{
    header::{HeaderDecodeError, StreamId},
    FrameDecodeError,
};

const KIB: usize = 1024;
const MIB: usize = KIB * 1024;
const GIB: usize = MIB * 1024;

pub const DEFAULT_CREDIT: u32 = 256 * KIB as u32; // as per yamux specification

pub type Result<T> = std::result::Result<T, ConnectionError>;

/// The maximum number of streams we will open without an acknowledgement from the other peer.
///
/// This enables a very basic form of backpressure on the creation of streams.
const MAX_ACK_BACKLOG: usize = 256;

/// Default maximum number of bytes a Yamux data frame might carry as its
/// payload when being send. Larger Payloads will be split.
///
/// The data frame payload size is not restricted by the yamux specification.
/// Still, this implementation restricts the size to:
///
/// 1. Reduce delays sending time-sensitive frames, e.g. window updates.
/// 2. Minimize head-of-line blocking across streams.
/// 3. Enable better interleaving of send and receive operations, as each is
///    carried out atomically instead of concurrently with its respective
///    counterpart.
///
/// For details on why this concrete value was chosen, see
/// https://github.com/paritytech/yamux/issues/100.
const DEFAULT_SPLIT_SEND_SIZE: usize = 16 * KIB;

/// Yamux configuration.
///
/// The default configuration values are as follows:
///
/// - max. for the total receive window size across all streams of a connection = 1 GiB
/// - max. number of streams = 512
/// - read after close = true
/// - split send size = 16 KiB
#[derive(Debug, Clone)]
pub struct Config {
    max_connection_receive_window: Option<usize>,
    max_num_streams: usize,
    read_after_close: bool,
    split_send_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_connection_receive_window: Some(1 * GIB),
            max_num_streams: 512,
            read_after_close: true,
            split_send_size: DEFAULT_SPLIT_SEND_SIZE,
        }
    }
}

impl Config {
    /// Set the upper limit for the total receive window size across all streams of a connection.
    ///
    /// Must be `>= 256 KiB * max_num_streams` to allow each stream at least the Yamux default
    /// window size.
    ///
    /// The window of a stream starts at 256 KiB and is increased (auto-tuned) based on the
    /// connection's round-trip time and the stream's bandwidth (striving for the
    /// bandwidth-delay-product).
    ///
    /// Set to `None` to disable limit, i.e. allow each stream to grow receive window based on
    /// connection's round-trip time and stream's bandwidth without limit.
    ///
    /// ## DOS attack mitigation
    ///
    /// A remote node (attacker) might trick the local node (target) into allocating large stream
    /// receive windows, trying to make the local node run out of memory.
    ///
    /// This attack is difficult, as the local node only increases the stream receive window up to
    /// 2x the bandwidth-delay-product, where bandwidth is the amount of bytes read, not just
    /// received. In other words, the attacker has to send (and have the local node read)
    /// significant amount of bytes on a stream over a long period of time to increase the stream
    /// receive window. E.g. on a 60ms 10Gbit/s connection the bandwidth-delay-product is ~75 MiB
    /// and thus the local node will at most allocate ~150 MiB (2x bandwidth-delay-product) per
    /// stream.
    ///
    /// Despite the difficulty of the attack one should choose a reasonable
    /// `max_connection_receive_window` to protect against this attack, especially since an attacker
    /// might use more than one stream per connection.
    pub fn set_max_connection_receive_window(&mut self, n: Option<usize>) -> &mut Self {
        self.max_connection_receive_window = n;

        assert!(
            self.max_connection_receive_window.unwrap_or(usize::MAX)
                >= self.max_num_streams * DEFAULT_CREDIT as usize,
            "`max_connection_receive_window` must be `>= 256 KiB * max_num_streams` to allow each
            stream at least the Yamux default window size"
        );

        self
    }

    /// Set the max. number of streams per connection.
    pub fn set_max_num_streams(&mut self, n: usize) -> &mut Self {
        self.max_num_streams = n;

        assert!(
            self.max_connection_receive_window.unwrap_or(usize::MAX)
                >= self.max_num_streams * DEFAULT_CREDIT as usize,
            "`max_connection_receive_window` must be `>= 256 KiB * max_num_streams` to allow each
            stream at least the Yamux default window size"
        );

        self
    }

    /// Allow or disallow streams to read from buffered data after
    /// the connection has been closed.
    pub fn set_read_after_close(&mut self, b: bool) -> &mut Self {
        self.read_after_close = b;
        self
    }

    /// Set the max. payload size used when sending data frames. Payloads larger
    /// than the configured max. will be split.
    pub fn set_split_send_size(&mut self, n: usize) -> &mut Self {
        self.split_send_size = n;
        self
    }
}

// Check that we can safely cast a `usize` to a `u64`.
static_assertions::const_assert! {
    std::mem::size_of::<usize>() <= std::mem::size_of::<u64>()
}

// Check that we can safely cast a `u32` to a `usize`.
static_assertions::const_assert! {
    std::mem::size_of::<u32>() <= std::mem::size_of::<usize>()
}

#[cfg(test)]
impl quickcheck::Arbitrary for Config {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use quickcheck::GenRange;

        let max_num_streams = g.gen_range(0..u16::MAX as usize);

        Config {
            max_connection_receive_window: if bool::arbitrary(g) {
                Some(g.gen_range((DEFAULT_CREDIT as usize * max_num_streams)..usize::MAX))
            } else {
                None
            },
            max_num_streams,
            read_after_close: bool::arbitrary(g),
            split_send_size: g.gen_range(DEFAULT_SPLIT_SEND_SIZE..usize::MAX),
        }
    }
}
