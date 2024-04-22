// Copyright (c) 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 or MIT license, at your option.
//
// A copy of the Apache License, Version 2.0 is included in the software as
// LICENSE-APACHE and a copy of the MIT license is included in the software
// as LICENSE-MIT. You may also obtain a copy of the Apache License, Version 2.0
// at https://www.apache.org/licenses/LICENSE-2.0 and a copy of the MIT license
// at https://opensource.org/licenses/MIT.

pub mod header;
mod io;

use futures::future::Either;
use header::{Data, GoAway, Header, Ping, StreamId, WindowUpdate};
use std::{convert::TryInto, num::TryFromIntError};

pub use io::FrameDecodeError;
pub(crate) use io::Io;

/// A Yamux message frame consisting of header and body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame<T> {
    header: Header<T>,
    body: Vec<u8>,
}

impl<T> Frame<T> {
    pub fn new(header: Header<T>) -> Self {
        Frame {
            header,
            body: Vec::new(),
        }
    }

    pub fn header(&self) -> &Header<T> {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header<T> {
        &mut self.header
    }

    /// Introduce this frame to the right of a binary frame type.
    pub(crate) fn right<U>(self) -> Frame<Either<U, T>> {
        Frame {
            header: self.header.right(),
            body: self.body,
        }
    }

    /// Introduce this frame to the left of a binary frame type.
    pub(crate) fn left<U>(self) -> Frame<Either<T, U>> {
        Frame {
            header: self.header.left(),
            body: self.body,
        }
    }
}

impl<A: header::private::Sealed> From<Frame<A>> for Frame<()> {
    fn from(f: Frame<A>) -> Frame<()> {
        Frame {
            header: f.header.into(),
            body: f.body,
        }
    }
}

impl Frame<()> {
    pub(crate) fn into_data(self) -> Frame<Data> {
        Frame {
            header: self.header.into_data(),
            body: self.body,
        }
    }

    pub(crate) fn into_window_update(self) -> Frame<WindowUpdate> {
        Frame {
            header: self.header.into_window_update(),
            body: self.body,
        }
    }

    pub(crate) fn into_ping(self) -> Frame<Ping> {
        Frame {
            header: self.header.into_ping(),
            body: self.body,
        }
    }
}

impl Frame<Data> {
    pub fn data(id: StreamId, b: Vec<u8>) -> Result<Self, TryFromIntError> {
        Ok(Frame {
            header: Header::data(id, b.len().try_into()?),
            body: b,
        })
    }

    pub fn close_stream(id: StreamId, ack: bool) -> Self {
        let mut header = Header::data(id, 0);
        header.fin();
        if ack {
            header.ack()
        }

        Frame::new(header)
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn body_len(&self) -> u32 {
        // Safe cast since we construct `Frame::<Data>`s only with
        // `Vec<u8>` of length [0, u32::MAX] in `Frame::data` above.
        self.body().len() as u32
    }

    pub fn into_body(self) -> Vec<u8> {
        self.body
    }
}

impl Frame<WindowUpdate> {
    pub fn window_update(id: StreamId, credit: u32) -> Self {
        Frame {
            header: Header::window_update(id, credit),
            body: Vec::new(),
        }
    }
}

impl Frame<Ping> {
    pub fn ping(nonce: u32) -> Self {
        let mut header = Header::ping(nonce);
        header.syn();

        Frame {
            header,
            body: Vec::new(),
        }
    }

    pub fn nonce(&self) -> u32 {
        self.header.nonce()
    }
}

impl Frame<GoAway> {
    pub fn term() -> Self {
        Frame {
            header: Header::term(),
            body: Vec::new(),
        }
    }

    pub fn protocol_error() -> Self {
        Frame {
            header: Header::protocol_error(),
            body: Vec::new(),
        }
    }

    pub fn internal_error() -> Self {
        Frame {
            header: Header::internal_error(),
            body: Vec::new(),
        }
    }
}
