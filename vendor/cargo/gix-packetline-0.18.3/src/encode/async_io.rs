use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures_io::AsyncWrite;
use futures_lite::AsyncWriteExt;

use super::u16_to_hex;
use crate::{encode::Error, Channel, DELIMITER_LINE, ERR_PREFIX, FLUSH_LINE, MAX_DATA_LEN, RESPONSE_END_LINE};

pin_project_lite::pin_project! {
    /// A way of writing packet lines asynchronously.
    pub struct LineWriter<'a, W> {
        #[pin]
        pub(crate) writer: W,
        pub(crate) prefix: &'a [u8],
        pub(crate) suffix: &'a [u8],
        state: State<'a>,
    }
}

enum State<'a> {
    Idle,
    WriteHexLen([u8; 4], usize),
    WritePrefix(&'a [u8]),
    WriteData(usize),
    WriteSuffix(&'a [u8]),
}

impl<'a, W: AsyncWrite + Unpin> LineWriter<'a, W> {
    /// Create a new line writer writing data with a `prefix` and `suffix`.
    ///
    /// Keep the additional `prefix` or `suffix` buffers empty if no prefix or suffix should be written.
    pub fn new(writer: W, prefix: &'a [u8], suffix: &'a [u8]) -> Self {
        LineWriter {
            writer,
            prefix,
            suffix,
            state: State::Idle,
        }
    }

    /// Consume self and reveal the inner writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

fn into_io_err(err: Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

impl<W: AsyncWrite + Unpin> AsyncWrite for LineWriter<'_, W> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, data: &[u8]) -> Poll<io::Result<usize>> {
        use futures_lite::ready;
        let mut this = self.project();
        loop {
            match &mut this.state {
                State::Idle => {
                    let data_len = this.prefix.len() + data.len() + this.suffix.len();
                    if data_len > MAX_DATA_LEN {
                        return Poll::Ready(Err(into_io_err(Error::DataLengthLimitExceeded {
                            length_in_bytes: data_len,
                        })));
                    }
                    if data.is_empty() {
                        return Poll::Ready(Err(into_io_err(Error::DataIsEmpty)));
                    }
                    let data_len = data_len + 4;
                    let len_buf = u16_to_hex(data_len as u16);
                    *this.state = State::WriteHexLen(len_buf, 0);
                }
                State::WriteHexLen(hex_len, written) => {
                    while *written != hex_len.len() {
                        let n = ready!(this.writer.as_mut().poll_write(cx, &hex_len[*written..]))?;
                        if n == 0 {
                            return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                        }
                        *written += n;
                    }
                    if this.prefix.is_empty() {
                        *this.state = State::WriteData(0);
                    } else {
                        *this.state = State::WritePrefix(this.prefix);
                    }
                }
                State::WritePrefix(buf) => {
                    while !buf.is_empty() {
                        let n = ready!(this.writer.as_mut().poll_write(cx, buf))?;
                        if n == 0 {
                            return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                        }
                        let (_, rest) = std::mem::take(buf).split_at(n);
                        *buf = rest;
                    }
                    *this.state = State::WriteData(0);
                }
                State::WriteData(written) => {
                    while *written != data.len() {
                        let n = ready!(this.writer.as_mut().poll_write(cx, &data[*written..]))?;
                        if n == 0 {
                            return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                        }
                        *written += n;
                    }
                    if this.suffix.is_empty() {
                        let written = 4 + this.prefix.len() + *written;
                        *this.state = State::Idle;
                        return Poll::Ready(Ok(written));
                    } else {
                        *this.state = State::WriteSuffix(this.suffix);
                    }
                }
                State::WriteSuffix(buf) => {
                    while !buf.is_empty() {
                        let n = ready!(this.writer.as_mut().poll_write(cx, buf))?;
                        if n == 0 {
                            return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                        }
                        let (_, rest) = std::mem::take(buf).split_at(n);
                        *buf = rest;
                    }
                    *this.state = State::Idle;
                    return Poll::Ready(Ok(4 + this.prefix.len() + data.len() + this.suffix.len()));
                }
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.project();
        this.writer.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.project();
        this.writer.poll_close(cx)
    }
}

async fn prefixed_and_suffixed_data_to_write(
    prefix: &[u8],
    data: &[u8],
    suffix: &[u8],
    mut out: impl AsyncWrite + Unpin,
) -> io::Result<usize> {
    let data_len = prefix.len() + data.len() + suffix.len();
    if data_len > MAX_DATA_LEN {
        return Err(into_io_err(Error::DataLengthLimitExceeded {
            length_in_bytes: data_len,
        }));
    }
    if data.is_empty() {
        return Err(into_io_err(Error::DataIsEmpty));
    }

    let data_len = data_len + 4;
    let buf = u16_to_hex(data_len as u16);

    out.write_all(&buf).await?;
    if !prefix.is_empty() {
        out.write_all(prefix).await?;
    }
    out.write_all(data).await?;
    if !suffix.is_empty() {
        out.write_all(suffix).await?;
    }
    Ok(data_len)
}

async fn prefixed_data_to_write(prefix: &[u8], data: &[u8], out: impl AsyncWrite + Unpin) -> io::Result<usize> {
    prefixed_and_suffixed_data_to_write(prefix, data, &[], out).await
}

/// Write a `text` message to `out`, which is assured to end in a newline.
pub async fn text_to_write(text: &[u8], out: impl AsyncWrite + Unpin) -> io::Result<usize> {
    prefixed_and_suffixed_data_to_write(&[], text, b"\n", out).await
}

/// Write a `data` message to `out`.
pub async fn data_to_write(data: &[u8], out: impl AsyncWrite + Unpin) -> io::Result<usize> {
    prefixed_data_to_write(&[], data, out).await
}

/// Write an error `message` to `out`.
pub async fn error_to_write(message: &[u8], out: impl AsyncWrite + Unpin) -> io::Result<usize> {
    prefixed_data_to_write(ERR_PREFIX, message, out).await
}

/// Write a response-end message to `out`.
pub async fn response_end_to_write(mut out: impl AsyncWrite + Unpin) -> io::Result<usize> {
    out.write_all(RESPONSE_END_LINE).await?;
    Ok(4)
}

/// Write a delim message to `out`.
pub async fn delim_to_write(mut out: impl AsyncWrite + Unpin) -> io::Result<usize> {
    out.write_all(DELIMITER_LINE).await?;
    Ok(4)
}

/// Write a flush message to `out`.
pub async fn flush_to_write(mut out: impl AsyncWrite + Unpin) -> io::Result<usize> {
    out.write_all(FLUSH_LINE).await?;
    Ok(4)
}

/// Write `data` of `kind` to `out` using side-band encoding.
pub async fn band_to_write(kind: Channel, data: &[u8], out: impl AsyncWrite + Unpin) -> io::Result<usize> {
    prefixed_data_to_write(&[kind as u8], data, out).await
}
