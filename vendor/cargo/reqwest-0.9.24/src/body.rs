use std::fs::File;
use std::fmt;
use std::io::{self, Cursor, Read};

use bytes::Bytes;
use futures::Future;
use hyper::{self};

use {async_impl};

/// The body of a `Request`.
///
/// In most cases, this is not needed directly, as the
/// [`RequestBuilder.body`][builder] method uses `Into<Body>`, which allows
/// passing many things (like a string or vector of bytes).
///
/// [builder]: ./struct.RequestBuilder.html#method.body
#[derive(Debug)]
pub struct Body {
    kind: Kind,
}

impl Body {
    /// Instantiate a `Body` from a reader.
    ///
    /// # Note
    ///
    /// While allowing for many types to be used, these bodies do not have
    /// a way to reset to the beginning and be reused. This means that when
    /// encountering a 307 or 308 status code, instead of repeating the
    /// request at the new location, the `Response` will be returned with
    /// the redirect status code set.
    ///
    /// ```rust
    /// # use std::fs::File;
    /// # use reqwest::Body;
    /// # fn run() -> Result<(), Box<std::error::Error>> {
    /// let file = File::open("national_secrets.txt")?;
    /// let body = Body::new(file);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// If you have a set of bytes, like `String` or `Vec<u8>`, using the
    /// `From` implementations for `Body` will store the data in a manner
    /// it can be reused.
    ///
    /// ```rust
    /// # use reqwest::Body;
    /// # fn run() -> Result<(), Box<std::error::Error>> {
    /// let s = "A stringy body";
    /// let body = Body::from(s);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<R: Read + Send + 'static>(reader: R) -> Body {
        Body {
            kind: Kind::Reader(Box::from(reader), None),
        }
    }

    /// Create a `Body` from a `Read` where the size is known in advance
    /// but the data should not be fully loaded into memory. This will
    /// set the `Content-Length` header and stream from the `Read`.
    ///
    /// ```rust
    /// # use std::fs::File;
    /// # use reqwest::Body;
    /// # fn run() -> Result<(), Box<std::error::Error>> {
    /// let file = File::open("a_large_file.txt")?;
    /// let file_size = file.metadata()?.len();
    /// let body = Body::sized(file, file_size);
    /// # Ok(())
    /// # }
    /// ```
    pub fn sized<R: Read + Send + 'static>(reader: R, len: u64) -> Body {
        Body {
            kind: Kind::Reader(Box::from(reader), Some(len)),
        }
    }

    pub(crate) fn len(&self) -> Option<u64> {
        match self.kind {
            Kind::Reader(_, len) => len,
            Kind::Bytes(ref bytes) => Some(bytes.len() as u64),
        }
    }

    pub(crate) fn into_reader(self) -> Reader {
        match self.kind {
            Kind::Reader(r, _) => Reader::Reader(r),
            Kind::Bytes(b) => Reader::Bytes(Cursor::new(b)),
        }
    }

    pub(crate) fn into_async(self) -> (Option<Sender>, async_impl::Body, Option<u64>) {
        match self.kind {
            Kind::Reader(read, len) => {
                let (tx, rx) = hyper::Body::channel();
                let tx = Sender {
                    body: (read, len),
                    tx: tx,
                };
                (Some(tx), async_impl::Body::wrap(rx), len)
            },
            Kind::Bytes(chunk) => {
                let len = chunk.len() as u64;
                (None, async_impl::Body::reusable(chunk), Some(len))
            }
        }
    }

    pub(crate) fn try_clone(&self) -> Option<Body> {
        self.kind.try_clone()
            .map(|kind| Body { kind })
    }
}


enum Kind {
    Reader(Box<dyn Read + Send>, Option<u64>),
    Bytes(Bytes),
}

impl Kind {
    fn try_clone(&self) -> Option<Kind> {
        match self {
            Kind::Reader(..) => None,
            Kind::Bytes(v) => Some(Kind::Bytes(v.clone())),
        }
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(v: Vec<u8>) -> Body {
        Body {
            kind: Kind::Bytes(v.into()),
        }
    }
}

impl From<String> for Body {
    #[inline]
    fn from(s: String) -> Body {
        s.into_bytes().into()
    }
}


impl From<&'static [u8]> for Body {
    #[inline]
    fn from(s: &'static [u8]) -> Body {
        Body {
            kind: Kind::Bytes(Bytes::from_static(s)),
        }
    }
}

impl From<&'static str> for Body {
    #[inline]
    fn from(s: &'static str) -> Body {
        s.as_bytes().into()
    }
}

impl From<File> for Body {
    #[inline]
    fn from(f: File) -> Body {
        let len = f.metadata().map(|m| m.len()).ok();
        Body {
            kind: Kind::Reader(Box::new(f), len),
        }
    }
}

impl fmt::Debug for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Kind::Reader(_, ref v) => f.debug_struct("Reader")
                .field("length", &DebugLength(v))
                .finish(),
            Kind::Bytes(ref v) => fmt::Debug::fmt(v, f),
        }
    }
}

struct DebugLength<'a>(&'a Option<u64>);

impl<'a> fmt::Debug for DebugLength<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            Some(ref len) => fmt::Debug::fmt(len, f),
            None => f.write_str("Unknown"),
        }
    }
}

pub(crate) enum Reader {
    Reader(Box<dyn Read + Send>),
    Bytes(Cursor<Bytes>),
}

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Reader::Reader(ref mut rdr) => rdr.read(buf),
            Reader::Bytes(ref mut rdr) => rdr.read(buf),
        }
    }
}

pub(crate) struct Sender {
    body: (Box<dyn Read + Send>, Option<u64>),
    tx: hyper::body::Sender,
}

impl Sender {
    // A `Future` that may do blocking read calls.
    // As a `Future`, this integrates easily with `wait::timeout`.
    pub(crate) fn send(self) -> impl Future<Item=(), Error=::Error> {
        use std::cmp;
        use bytes::{BufMut, BytesMut};
        use futures::future;

        let con_len = self.body.1;
        let cap = cmp::min(self.body.1.unwrap_or(8192), 8192);
        let mut written = 0;
        let mut buf = BytesMut::with_capacity(cap as usize);
        let mut body = self.body.0;
        // Put in an option so that it can be consumed on error to call abort()
        let mut tx = Some(self.tx);

        future::poll_fn(move || loop {
            if Some(written) == con_len {
                // Written up to content-length, so stop.
                return Ok(().into());
            }

            // The input stream is read only if the buffer is empty so
            // that there is only one read in the buffer at any time.
            //
            // We need to know whether there is any data to send before
            // we check the transmission channel (with poll_ready below)
            // because somestimes the receiver disappears as soon as is
            // considers the data is completely transmitted, which may
            // be true.
            //
            // The use case is a web server that closes its
            // input stream as soon as the data received is valid JSON.
            // This behaviour is questionable, but it exists and the
            // fact is that there is actually no remaining data to read.
            if buf.len() == 0 {
                if buf.remaining_mut() == 0 {
                    buf.reserve(8192);
                }

                match body.read(unsafe { buf.bytes_mut() }) {
                    Ok(0) => {
                        // The buffer was empty and nothing's left to
                        // read. Return.
                        return Ok(().into());
                    }
                    Ok(n) => {
                        unsafe { buf.advance_mut(n); }
                    }
                    Err(e) => {
                        let ret = io::Error::new(e.kind(), e.to_string());
                        tx
                            .take()
                            .expect("tx only taken on error")
                            .abort();
                        return Err(::error::from(ret));
                    }
                }
            }

            // The only way to get here is when the buffer is not empty.
            // We can check the transmission channel
            try_ready!(tx
                .as_mut()
                .expect("tx only taken on error")
                .poll_ready()
                .map_err(::error::from));

            written += buf.len() as u64;
            let tx = tx.as_mut().expect("tx only taken on error");
            if let Err(_) = tx.send_data(buf.take().freeze().into()) {
                return Err(::error::timedout(None));
            }
        })
    }
}

// useful for tests, but not publicly exposed
#[cfg(test)]
pub(crate) fn read_to_string(mut body: Body) -> io::Result<String> {
    let mut s = String::new();
    match body.kind {
            Kind::Reader(ref mut reader, _) => reader.read_to_string(&mut s),
            Kind::Bytes(ref mut bytes) => (&**bytes).read_to_string(&mut s),
        }
        .map(|_| s)
}
