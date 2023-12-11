use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_lite::io::{self, AsyncRead as Read};
use futures_lite::ready;
use http_types::trailers::{Sender, Trailers};

/// Decodes a chunked body according to
/// https://tools.ietf.org/html/rfc7230#section-4.1
#[derive(Debug)]
pub struct ChunkedDecoder<R: Read> {
    /// The underlying stream
    inner: R,
    /// Current state.
    state: State,
    /// Current chunk size (increased while parsing size, decreased while reading chunk)
    chunk_size: u64,
    /// Trailer channel sender.
    trailer_sender: Option<Sender>,
}

impl<R: Read> ChunkedDecoder<R> {
    pub(crate) fn new(inner: R, trailer_sender: Sender) -> Self {
        ChunkedDecoder {
            inner,
            state: State::ChunkSize,
            chunk_size: 0,
            trailer_sender: Some(trailer_sender),
        }
    }
}

/// Decoder state.
enum State {
    /// Parsing bytes from a chunk size
    ChunkSize,
    /// Expecting the \n at the end of a chunk size
    ChunkSizeExpectLf,
    /// Parsing the chunk body
    ChunkBody,
    /// Expecting the \r at the end of a chunk body
    ChunkBodyExpectCr,
    /// Expecting the \n at the end of a chunk body
    ChunkBodyExpectLf,
    /// Parsing trailers.
    Trailers(usize, Box<[u8; 8192]>),
    /// Sending trailers over the channel.
    TrailerSending(Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>),
    /// All is said and done.
    Done,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::ChunkSize => write!(f, "State::ChunkSize"),
            State::ChunkSizeExpectLf => write!(f, "State::ChunkSizeExpectLf"),
            State::ChunkBody => write!(f, "State::ChunkBody"),
            State::ChunkBodyExpectCr => write!(f, "State::ChunkBodyExpectCr"),
            State::ChunkBodyExpectLf => write!(f, "State::ChunkBodyExpectLf"),
            State::Trailers(len, _) => write!(f, "State::Trailers({}, _)", len),
            State::TrailerSending(_) => write!(f, "State::TrailerSending"),
            State::Done => write!(f, "State::Done"),
        }
    }
}

impl<R: Read + Unpin> ChunkedDecoder<R> {
    fn poll_read_byte(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<u8>> {
        let mut byte = [0u8];
        if ready!(Pin::new(&mut self.inner).poll_read(cx, &mut byte))? == 1 {
            Poll::Ready(Ok(byte[0]))
        } else {
            eof()
        }
    }

    fn expect_byte(
        &mut self,
        cx: &mut Context<'_>,
        expected_byte: u8,
        expected: &'static str,
    ) -> Poll<io::Result<()>> {
        let byte = ready!(self.poll_read_byte(cx))?;
        if byte == expected_byte {
            Poll::Ready(Ok(()))
        } else {
            unexpected(byte, expected)
        }
    }

    fn send_trailers(&mut self, trailers: Trailers) {
        let sender = self
            .trailer_sender
            .take()
            .expect("invalid chunked state, tried sending multiple trailers");
        let fut = Box::pin(sender.send(trailers));
        self.state = State::TrailerSending(fut);
    }
}

fn eof<T>() -> Poll<io::Result<T>> {
    Poll::Ready(Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "Unexpected EOF when decoding chunked data",
    )))
}

fn unexpected<T>(byte: u8, expected: &'static str) -> Poll<io::Result<T>> {
    Poll::Ready(Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Unexpected byte {}; expected {}", byte, expected),
    )))
}

fn overflow() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "Chunk size overflowed 64 bits")
}

impl<R: Read + Unpin> Read for ChunkedDecoder<R> {
    #[allow(missing_doc_code_examples)]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = &mut *self;

        loop {
            match this.state {
                State::ChunkSize => {
                    let byte = ready!(this.poll_read_byte(cx))?;
                    let digit = match byte {
                        b'0'..=b'9' => byte - b'0',
                        b'a'..=b'f' => 10 + byte - b'a',
                        b'A'..=b'F' => 10 + byte - b'A',
                        b'\r' => {
                            this.state = State::ChunkSizeExpectLf;
                            continue;
                        }
                        _ => {
                            return unexpected(byte, "hex digit or CR");
                        }
                    };
                    this.chunk_size = this
                        .chunk_size
                        .checked_mul(16)
                        .ok_or_else(overflow)?
                        .checked_add(digit as u64)
                        .ok_or_else(overflow)?;
                }
                State::ChunkSizeExpectLf => {
                    ready!(this.expect_byte(cx, b'\n', "LF"))?;
                    if this.chunk_size == 0 {
                        this.state = State::Trailers(0, Box::new([0u8; 8192]));
                    } else {
                        this.state = State::ChunkBody;
                    }
                }
                State::ChunkBody => {
                    let max_bytes = std::cmp::min(
                        buf.len(),
                        std::cmp::min(this.chunk_size, usize::MAX as u64) as usize,
                    );
                    let bytes_read =
                        ready!(Pin::new(&mut this.inner).poll_read(cx, &mut buf[..max_bytes]))?;
                    this.chunk_size -= bytes_read as u64;
                    if bytes_read == 0 {
                        return eof();
                    } else if this.chunk_size == 0 {
                        this.state = State::ChunkBodyExpectCr;
                    }
                    return Poll::Ready(Ok(bytes_read));
                }
                State::ChunkBodyExpectCr => {
                    ready!(this.expect_byte(cx, b'\r', "CR"))?;
                    this.state = State::ChunkBodyExpectLf;
                }
                State::ChunkBodyExpectLf => {
                    ready!(this.expect_byte(cx, b'\n', "LF"))?;
                    this.state = State::ChunkSize;
                }
                State::Trailers(ref mut len, ref mut buf) => {
                    let bytes_read =
                        ready!(Pin::new(&mut this.inner).poll_read(cx, &mut buf[*len..]))?;
                    *len += bytes_read;
                    let len = *len;
                    if len == 0 {
                        this.send_trailers(Trailers::new());
                        continue;
                    }
                    if bytes_read == 0 {
                        return eof();
                    }
                    let mut headers = [httparse::EMPTY_HEADER; 16];
                    let parse_result = httparse::parse_headers(&buf[..len], &mut headers)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    use httparse::Status;
                    match parse_result {
                        Status::Partial => {
                            if len == buf.len() {
                                return eof();
                            } else {
                                return Poll::Pending;
                            }
                        }
                        Status::Complete((offset, headers)) => {
                            if offset != len {
                                return unexpected(buf[offset], "end of trailers");
                            }
                            let mut trailers = Trailers::new();
                            for header in headers {
                                trailers.insert(
                                    header.name,
                                    String::from_utf8_lossy(header.value).as_ref(),
                                );
                            }
                            this.send_trailers(trailers);
                        }
                    }
                }
                State::TrailerSending(ref mut fut) => {
                    ready!(Pin::new(fut).poll(cx));
                    this.state = State::Done;
                }
                State::Done => return Poll::Ready(Ok(0)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::prelude::*;

    #[test]
    fn test_chunked_wiki() {
        async_std::task::block_on(async move {
            let input = async_std::io::Cursor::new(
                "4\r\n\
                  Wiki\r\n\
                  5\r\n\
                  pedia\r\n\
                  E\r\n in\r\n\
                  \r\n\
                  chunks.\r\n\
                  0\r\n\
                  \r\n"
                    .as_bytes(),
            );

            let (s, _r) = async_channel::bounded(1);
            let sender = Sender::new(s);
            let mut decoder = ChunkedDecoder::new(input, sender);

            let mut output = String::new();
            decoder.read_to_string(&mut output).await.unwrap();
            assert_eq!(
                output,
                "Wikipedia in\r\n\
                 \r\n\
                 chunks."
            );
        });
    }

    #[test]
    fn test_chunked_big() {
        async_std::task::block_on(async move {
            let mut input: Vec<u8> = b"800\r\n".to_vec();
            input.extend(vec![b'X'; 2048]);
            input.extend(b"\r\n1800\r\n");
            input.extend(vec![b'Y'; 6144]);
            input.extend(b"\r\n800\r\n");
            input.extend(vec![b'Z'; 2048]);
            input.extend(b"\r\n0\r\n\r\n");

            let (s, _r) = async_channel::bounded(1);
            let sender = Sender::new(s);
            let mut decoder = ChunkedDecoder::new(async_std::io::Cursor::new(input), sender);

            let mut output = String::new();
            decoder.read_to_string(&mut output).await.unwrap();

            let mut expected = vec![b'X'; 2048];
            expected.extend(vec![b'Y'; 6144]);
            expected.extend(vec![b'Z'; 2048]);
            assert_eq!(output.len(), 10240);
            assert_eq!(output.as_bytes(), expected.as_slice());
        });
    }

    #[test]
    fn test_chunked_mdn() {
        async_std::task::block_on(async move {
            let input = async_std::io::Cursor::new(
                "7\r\n\
                 Mozilla\r\n\
                 9\r\n\
                 Developer\r\n\
                 7\r\n\
                 Network\r\n\
                 0\r\n\
                 Expires: Wed, 21 Oct 2015 07:28:00 GMT\r\n\
                 \r\n"
                    .as_bytes(),
            );
            let (s, r) = async_channel::bounded(1);
            let sender = Sender::new(s);
            let mut decoder = ChunkedDecoder::new(input, sender);

            let mut output = String::new();
            decoder.read_to_string(&mut output).await.unwrap();
            assert_eq!(output, "MozillaDeveloperNetwork");

            let trailers = r.recv().await.unwrap();
            assert_eq!(trailers.iter().count(), 1);
            assert_eq!(trailers["Expires"], "Wed, 21 Oct 2015 07:28:00 GMT");
        });
    }

    #[test]
    fn test_ff7() {
        async_std::task::block_on(async move {
            let mut input: Vec<u8> = b"FF7\r\n".to_vec();
            input.extend(vec![b'X'; 0xFF7]);
            input.extend(b"\r\n4\r\n");
            input.extend(vec![b'Y'; 4]);
            input.extend(b"\r\n0\r\n\r\n");

            let (s, _r) = async_channel::bounded(1);
            let sender = Sender::new(s);
            let mut decoder = ChunkedDecoder::new(async_std::io::Cursor::new(input), sender);

            let mut output = String::new();
            decoder.read_to_string(&mut output).await.unwrap();
            assert_eq!(
                output,
                "X".to_string().repeat(0xFF7) + &"Y".to_string().repeat(4)
            );
        });
    }
}
