// Copyright (c) 2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 or MIT license, at your option.
//
// A copy of the Apache License, Version 2.0 is included in the software as
// LICENSE-APACHE and a copy of the MIT license is included in the software
// as LICENSE-MIT. You may also obtain a copy of the Apache License, Version 2.0
// at https://www.apache.org/licenses/LICENSE-2.0 and a copy of the MIT license
// at https://opensource.org/licenses/MIT.

use std::{collections::VecDeque, io};

/// A sequence of [`Chunk`] values.
///
/// [`Chunks::len`] considers all [`Chunk`] elements and computes the total
/// result, i.e. the length of all bytes, by summing up the lengths of all
/// [`Chunk`] elements.
#[derive(Debug)]
pub(crate) struct Chunks {
    seq: VecDeque<Chunk>,
    len: usize,
}

impl Chunks {
    /// A new empty chunk list.
    pub(crate) fn new() -> Self {
        Chunks {
            seq: VecDeque::new(),
            len: 0,
        }
    }

    /// The total length of bytes yet-to-be-read in all `Chunk`s.
    pub(crate) fn len(&self) -> usize {
        self.len - self.seq.front().map(|c| c.offset()).unwrap_or(0)
    }

    /// Add another chunk of bytes to the end.
    pub(crate) fn push(&mut self, x: Vec<u8>) {
        self.len += x.len();
        if !x.is_empty() {
            self.seq.push_back(Chunk {
                cursor: io::Cursor::new(x),
            })
        }
    }

    /// Remove and return the first chunk.
    pub(crate) fn pop(&mut self) -> Option<Chunk> {
        let chunk = self.seq.pop_front();
        self.len -= chunk.as_ref().map(|c| c.len() + c.offset()).unwrap_or(0);
        chunk
    }

    /// Get a mutable reference to the first chunk.
    pub(crate) fn front_mut(&mut self) -> Option<&mut Chunk> {
        self.seq.front_mut()
    }
}

/// A `Chunk` wraps a `std::io::Cursor<Vec<u8>>`.
///
/// It provides a byte-slice view and a way to advance the cursor so the
/// vector can be consumed in steps.
#[derive(Debug)]
pub(crate) struct Chunk {
    cursor: io::Cursor<Vec<u8>>,
}

impl Chunk {
    /// Is this chunk empty?
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The remaining number of bytes in this `Chunk`.
    pub(crate) fn len(&self) -> usize {
        self.cursor.get_ref().len() - self.offset()
    }

    /// The sum of bytes that the cursor has been `advance`d over.
    pub(crate) fn offset(&self) -> usize {
        self.cursor.position() as usize
    }

    /// Move the cursor position by `amount` bytes.
    ///
    /// The `AsRef<[u8]>` impl of `Chunk` provides a byte-slice view
    /// from the current position to the end.
    pub(crate) fn advance(&mut self, amount: usize) {
        assert!({
            // the new position must not exceed the vector's length
            let pos = self.offset().checked_add(amount);
            let max = self.cursor.get_ref().len();
            pos.is_some() && pos <= Some(max)
        });

        self.cursor
            .set_position(self.cursor.position() + amount as u64);
    }

    /// Consume `self` and return the inner vector.
    pub(crate) fn into_vec(self) -> Vec<u8> {
        self.cursor.into_inner()
    }
}

impl AsRef<[u8]> for Chunk {
    fn as_ref(&self) -> &[u8] {
        &self.cursor.get_ref()[self.offset()..]
    }
}
