use std::io::Write;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_lite::io::{self, AsyncRead as Read, Cursor};
use http_types::headers::{CONTENT_LENGTH, HOST, TRANSFER_ENCODING};
use http_types::{Method, Request};

use crate::body_encoder::BodyEncoder;
use crate::read_to_end;
use crate::EncoderState;

/// An HTTP encoder.
#[doc(hidden)]
#[derive(Debug)]
pub struct Encoder {
    request: Request,
    state: EncoderState,
}

impl Encoder {
    /// build a new client encoder
    pub fn new(request: Request) -> Self {
        Self {
            request,
            state: EncoderState::Start,
        }
    }

    fn finalize_headers(&mut self) -> io::Result<()> {
        if self.request.header(HOST).is_none() {
            let url = self.request.url();
            let host = url
                .host_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing hostname"))?
                .to_owned();

            if let Some(port) = url.port() {
                self.request
                    .insert_header(HOST, format!("{}:{}", host, port));
            } else {
                self.request.insert_header(HOST, host);
            };
        }

        // Insert Proxy-Connection header when method is CONNECT
        if self.request.method() == Method::Connect {
            self.request.insert_header("proxy-connection", "keep-alive");
        }

        // If the body isn't streaming, we can set the content-length ahead of time. Else we need to
        // send all items in chunks.
        if let Some(len) = self.request.len() {
            self.request.insert_header(CONTENT_LENGTH, len.to_string());
        } else {
            self.request.insert_header(TRANSFER_ENCODING, "chunked");
        }

        Ok(())
    }

    fn compute_head(&mut self) -> io::Result<Cursor<Vec<u8>>> {
        let mut buf = Vec::with_capacity(128);
        let url = self.request.url();
        let method = self.request.method();
        write!(buf, "{} ", method)?;

        // A client sending a CONNECT request MUST consists of only the host
        // name and port number of the tunnel destination, separated by a colon.
        // See: https://tools.ietf.org/html/rfc7231#section-4.3.6
        if method == Method::Connect {
            let host = url
                .host_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing hostname"))?;

            let port = url.port_or_known_default().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unexpected scheme with no default port",
                )
            })?;

            write!(buf, "{}:{}", host, port)?;
        } else {
            write!(buf, "{}", url.path())?;
            if let Some(query) = url.query() {
                write!(buf, "?{}", query)?;
            }
        }

        write!(buf, " HTTP/1.1\r\n")?;

        self.finalize_headers()?;
        let mut headers = self.request.iter().collect::<Vec<_>>();
        headers.sort_unstable_by_key(|(h, _)| if **h == HOST { "0" } else { h.as_str() });
        for (header, values) in headers {
            for value in values.iter() {
                write!(buf, "{}: {}\r\n", header, value)?;
            }
        }

        write!(buf, "\r\n")?;
        Ok(Cursor::new(buf))
    }
}

impl Read for Encoder {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            self.state = match self.state {
                EncoderState::Start => EncoderState::Head(self.compute_head()?),

                EncoderState::Head(ref mut cursor) => {
                    read_to_end!(Pin::new(cursor).poll_read(cx, buf));
                    EncoderState::Body(BodyEncoder::new(self.request.take_body()))
                }

                EncoderState::Body(ref mut encoder) => {
                    read_to_end!(Pin::new(encoder).poll_read(cx, buf));
                    EncoderState::End
                }

                EncoderState::End => return Poll::Ready(Ok(0)),
            }
        }
    }
}
