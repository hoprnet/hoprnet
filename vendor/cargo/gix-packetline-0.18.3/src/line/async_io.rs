use std::io;

use futures_io::AsyncWrite;

use crate::{encode, BandRef, Channel, ErrorRef, PacketLineRef, TextRef};

impl BandRef<'_> {
    /// Serialize this instance to `out`, returning the amount of bytes written.
    ///
    /// The data written to `out` can be decoded with [`Borrowed::decode_band()]`.
    pub async fn write_to(&self, out: impl AsyncWrite + Unpin) -> io::Result<usize> {
        match self {
            BandRef::Data(d) => encode::band_to_write(Channel::Data, d, out),
            BandRef::Progress(d) => encode::band_to_write(Channel::Progress, d, out),
            BandRef::Error(d) => encode::band_to_write(Channel::Error, d, out),
        }
        .await
    }
}

impl TextRef<'_> {
    /// Serialize this instance to `out`, appending a newline if there is none, returning the amount of bytes written.
    pub async fn write_to(&self, out: impl AsyncWrite + Unpin) -> io::Result<usize> {
        encode::text_to_write(self.0, out).await
    }
}

impl ErrorRef<'_> {
    /// Serialize this line as error to `out`.
    ///
    /// This includes a marker to allow decoding it outside of a side-band channel, returning the amount of bytes written.
    pub async fn write_to(&self, out: impl AsyncWrite + Unpin) -> io::Result<usize> {
        encode::error_to_write(self.0, out).await
    }
}

impl PacketLineRef<'_> {
    /// Serialize this instance to `out` in git `packetline` format, returning the amount of bytes written to `out`.
    pub async fn write_to(&self, out: impl AsyncWrite + Unpin) -> io::Result<usize> {
        match self {
            PacketLineRef::Data(d) => encode::data_to_write(d, out).await,
            PacketLineRef::Flush => encode::flush_to_write(out).await,
            PacketLineRef::Delimiter => encode::delim_to_write(out).await,
            PacketLineRef::ResponseEnd => encode::response_end_to_write(out).await,
        }
    }
}
