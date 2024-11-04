// Copyright (c) 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 or MIT license, at your option.
//
// A copy of the Apache License, Version 2.0 is included in the software as
// LICENSE-APACHE and a copy of the MIT license is included in the software
// as LICENSE-MIT. You may also obtain a copy of the Apache License, Version 2.0
// at https://www.apache.org/licenses/LICENSE-2.0 and a copy of the MIT license
// at https://opensource.org/licenses/MIT.

use crate::frame::FrameDecodeError;

/// The various error cases a connection may encounter.
#[non_exhaustive]
#[derive(Debug)]
pub enum ConnectionError {
    /// An underlying I/O error occured.
    Io(std::io::Error),
    /// Decoding a Yamux message frame failed.
    Decode(FrameDecodeError),
    /// The whole range of stream IDs has been used up.
    NoMoreStreamIds,
    /// An operation fails because the connection is closed.
    Closed,
    /// Too many streams are open, so no further ones can be opened at this time.
    TooManyStreams,
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConnectionError::Io(e) => write!(f, "i/o error: {}", e),
            ConnectionError::Decode(e) => write!(f, "decode error: {}", e),
            ConnectionError::NoMoreStreamIds => {
                f.write_str("number of stream ids has been exhausted")
            }
            ConnectionError::Closed => f.write_str("connection is closed"),
            ConnectionError::TooManyStreams => f.write_str("maximum number of streams reached"),
        }
    }
}

impl std::error::Error for ConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConnectionError::Io(e) => Some(e),
            ConnectionError::Decode(e) => Some(e),
            ConnectionError::NoMoreStreamIds
            | ConnectionError::Closed
            | ConnectionError::TooManyStreams => None,
        }
    }
}

impl From<std::io::Error> for ConnectionError {
    fn from(e: std::io::Error) -> Self {
        ConnectionError::Io(e)
    }
}

impl From<FrameDecodeError> for ConnectionError {
    fn from(e: FrameDecodeError) -> Self {
        ConnectionError::Decode(e)
    }
}

impl From<futures::channel::mpsc::SendError> for ConnectionError {
    fn from(_: futures::channel::mpsc::SendError) -> Self {
        ConnectionError::Closed
    }
}

impl From<futures::channel::oneshot::Canceled> for ConnectionError {
    fn from(_: futures::channel::oneshot::Canceled) -> Self {
        ConnectionError::Closed
    }
}
