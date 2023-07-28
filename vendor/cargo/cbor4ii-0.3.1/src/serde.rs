//! serde support

mod error;
mod ser;
mod de;

#[cfg(feature = "use_std")]
mod io_writer {
    use std::io;
    use serde::Serialize;
    use crate::core::utils::IoWriter;
    use crate::serde::error::EncodeError;
    use crate::serde::ser;

    /// Serializes a value to a writer.
    pub fn to_writer<W, T>(writer: &mut W, value: &T)
        -> Result<(), EncodeError<io::Error>>
    where
        W: io::Write,
        T: Serialize
    {
        let writer = IoWriter::new(writer);
        let mut writer = ser::Serializer::new(writer);
        value.serialize(&mut writer)
    }
}

mod buf_writer {
    use crate::alloc::vec::Vec;
    use crate::alloc::collections::TryReserveError;
    use serde::Serialize;
    use crate::core::utils::BufWriter;
    use crate::serde::error::EncodeError;
    use crate::serde::ser;

    /// Serializes a value to a writer.
    pub fn to_vec<T>(buf: Vec<u8>, value: &T)
        -> Result<Vec<u8>, EncodeError<TryReserveError>>
    where T: Serialize
    {
        let writer = BufWriter::new(buf);
        let mut writer = ser::Serializer::new(writer);
        value.serialize(&mut writer)?;
        Ok(writer.into_inner().into_inner())
    }
}

mod slice_reader {
    use core::convert::Infallible;
    use crate::core::utils::SliceReader;
    use crate::serde::de;
    use crate::serde::error::DecodeError;

    /// Decodes a value from a bytes.
    pub fn from_slice<'a, T>(buf: &'a [u8]) -> Result<T, DecodeError<Infallible>>
    where
        T: serde::Deserialize<'a>,
    {
        let reader = SliceReader::new(buf);
        let mut deserializer = de::Deserializer::new(reader);
        serde::Deserialize::deserialize(&mut deserializer)
    }
}

#[cfg(feature = "use_std")]
mod io_buf_reader {
    use std::io::{ self, BufRead };
    use crate::core::utils::IoReader;
    use crate::serde::de;
    use crate::serde::error::DecodeError;

    /// Decodes a value from a reader.
    pub fn from_reader<T, R>(reader: R) -> Result<T, DecodeError<io::Error>>
    where
        T: serde::de::DeserializeOwned,
        R: BufRead
    {
        let reader = IoReader::new(reader);
        let mut deserializer = de::Deserializer::new(reader);
        serde::Deserialize::deserialize(&mut deserializer)
    }
}

#[cfg(feature = "use_std")] pub use io_writer::to_writer;
#[cfg(feature = "use_std")] pub use io_buf_reader::from_reader;
pub use buf_writer::to_vec;
pub use slice_reader::from_slice;

pub use error::{ EncodeError, DecodeError };
pub use ser::Serializer;
pub use de::Deserializer;
