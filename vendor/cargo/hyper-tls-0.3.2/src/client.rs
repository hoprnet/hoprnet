use std::fmt;
use std::io;

use futures::{Async, Future, future, Poll};
use hyper::client::connect::{Connect, Connected, Destination, HttpConnector};
pub use native_tls::Error;
use native_tls::{self, HandshakeError, TlsConnector};

use stream::{MaybeHttpsStream, TlsStream};

/// A Connector for the `https` scheme.
#[derive(Clone)]
pub struct HttpsConnector<T> {
    force_https: bool,
    http: T,
    tls: TlsConnector,
}

impl HttpsConnector<HttpConnector> {
    /// Construct a new HttpsConnector.
    ///
    /// Takes number of DNS worker threads.
    ///
    /// This uses hyper's default `HttpConnector`, and default `TlsConnector`.
    /// If you wish to use something besides the defaults, use `From::from`.
    ///
    /// # Note
    ///
    /// By default this connector will use plain HTTP if the URL provded uses
    /// the HTTP scheme (eg: http://example.com/).
    ///
    /// If you would like to force the use of HTTPS then call https_only(true)
    /// on the returned connector.
    pub fn new(threads: usize) -> Result<Self, Error> {
        TlsConnector::builder()
            .build()
            .map(|tls| HttpsConnector::new_(threads, tls))
    }

    fn new_(threads: usize, tls: TlsConnector) -> Self {
        let mut http = HttpConnector::new(threads);
        http.enforce_http(false);
        HttpsConnector::from((http, tls))
    }
}

impl<T> HttpsConnector<T> {
    /// Force the use of HTTPS when connecting.
    ///
    /// If a URL is not `https` when connecting, an error is returned.
    pub fn https_only(&mut self, enable: bool) {
        self.force_https = enable;
    }

    #[doc(hidden)]
    #[deprecated(since = "0.3", note = "use `https_only` method instead")]
    pub fn force_https(&mut self, enable: bool) {
        self.force_https = enable;
    }
}

impl<T> From<(T, TlsConnector)> for HttpsConnector<T> {
    fn from(args: (T, TlsConnector)) -> HttpsConnector<T> {
        HttpsConnector {
            force_https: false,
            http: args.0,
            tls: args.1,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for HttpsConnector<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HttpsConnector")
            .field("force_https", &self.force_https)
            .field("http", &self.http)
            .finish()
    }
}

impl<T> Connect for HttpsConnector<T>
where
    T: Connect<Error=io::Error>,
    T::Transport: 'static,
    T::Future: 'static,
{
    type Transport = MaybeHttpsStream<T::Transport>;
    type Error = io::Error;
    type Future = HttpsConnecting<T::Transport>;

    fn connect(&self, dst: Destination) -> Self::Future {
        let is_https = dst.scheme() == "https";
        // Early abort if HTTPS is forced but can't be used
        if !is_https && self.force_https {
            let err = io::Error::new(io::ErrorKind::Other, "HTTPS scheme forced but can't be used");
            return HttpsConnecting(Box::new(future::err(err)));
        }

        let host = dst.host().to_owned();
        let connecting = self.http.connect(dst);
        let tls = self.tls.clone();
        let fut: BoxedFut<T::Transport> = if is_https {
            let fut = connecting.and_then(move |(tcp, connected)| {
                let handshake = Handshaking {
                    inner: Some(tls.connect(&host, tcp)),
                };
                handshake
                    .map(|conn| (MaybeHttpsStream::Https(TlsStream::new(conn)), connected))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            });
            Box::new(fut)
        } else {
            Box::new(connecting.map(|(tcp, connected)| {
                (MaybeHttpsStream::Http(tcp), connected)
            }))
        };
        HttpsConnecting(fut)
    }

}

type BoxedFut<T> = Box<Future<Item=(MaybeHttpsStream<T>, Connected), Error=io::Error> + Send>;

/// A Future representing work to connect to a URL, and a TLS handshake.
pub struct HttpsConnecting<T>(BoxedFut<T>);


impl<T> Future for HttpsConnecting<T> {
    type Item = (MaybeHttpsStream<T>, Connected);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

impl<T> fmt::Debug for HttpsConnecting<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("HttpsConnecting")
    }
}

struct Handshaking<T> {
    inner: Option<Result<native_tls::TlsStream<T>, HandshakeError<T>>>,
}

impl<T: io::Read + io::Write> Future for Handshaking<T> {
    type Item = native_tls::TlsStream<T>;
    type Error = native_tls::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner.take().expect("polled after ready") {
            Ok(stream) => Ok(stream.into()),
            Err(HandshakeError::WouldBlock(mid)) => {
                match mid.handshake() {
                    Ok(stream) => Ok(stream.into()),
                    Err(HandshakeError::Failure(err)) => Err(err),
                    Err(HandshakeError::WouldBlock(mid)) => {
                        self.inner = Some(Err(HandshakeError::WouldBlock(mid)));
                        Ok(Async::NotReady)
                    }
                }
            },
            Err(HandshakeError::Failure(err)) => Err(err),
        }
    }
}
