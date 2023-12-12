//! Process HTTP connections on the server.

use std::io::Write;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::SystemTime;

use futures_lite::io::{self, AsyncRead as Read, Cursor};
use http_types::headers::{CONTENT_LENGTH, DATE, TRANSFER_ENCODING};
use http_types::{Method, Response};

use crate::body_encoder::BodyEncoder;
use crate::date::fmt_http_date;
use crate::read_to_end;
use crate::EncoderState;

/// A streaming HTTP encoder.
#[derive(Debug)]
pub struct Encoder {
    response: Response,
    state: EncoderState,
    method: Method,
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

                    if self.method == Method::Head {
                        EncoderState::End
                    } else {
                        EncoderState::Body(BodyEncoder::new(self.response.take_body()))
                    }
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

impl Encoder {
    /// Create a new instance of Encoder.
    pub fn new(response: Response, method: Method) -> Self {
        Self {
            method,
            response,
            state: EncoderState::Start,
        }
    }

    fn finalize_headers(&mut self) {
        // If the body isn't streaming, we can set the content-length ahead of time. Else we need to
        // send all items in chunks.
        if let Some(len) = self.response.len() {
            self.response.insert_header(CONTENT_LENGTH, len.to_string());
        } else {
            self.response.insert_header(TRANSFER_ENCODING, "chunked");
        }

        if self.response.header(DATE).is_none() {
            let date = fmt_http_date(SystemTime::now());
            self.response.insert_header(DATE, date);
        }
    }

    /// Encode the headers to a buffer, the first time we poll.
    fn compute_head(&mut self) -> io::Result<Cursor<Vec<u8>>> {
        let mut head = Vec::with_capacity(128);
        let reason = self.response.status().canonical_reason();
        let status = self.response.status();
        write!(head, "HTTP/1.1 {} {}\r\n", status, reason)?;

        self.finalize_headers();
        let mut headers = self.response.iter().collect::<Vec<_>>();
        headers.sort_unstable_by_key(|(h, _)| h.as_str());
        for (header, values) in headers {
            for value in values.iter() {
                write!(head, "{}: {}\r\n", header, value)?;
            }
        }
        write!(head, "\r\n")?;
        Ok(Cursor::new(head))
    }
}
