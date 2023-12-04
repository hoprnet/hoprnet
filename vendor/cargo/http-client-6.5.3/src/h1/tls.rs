use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use async_std::net::TcpStream;
use async_trait::async_trait;
use deadpool::managed::{Manager, Object, RecycleResult};
use futures::io::{AsyncRead, AsyncWrite};
use futures::task::{Context, Poll};

cfg_if::cfg_if! {
    if #[cfg(feature = "rustls")] {
        use async_tls::client::TlsStream;
    } else if #[cfg(feature = "native-tls")] {
        use async_native_tls::TlsStream;
    }
}

use crate::{Config, Error};

#[derive(Clone)]
#[cfg_attr(not(feature = "rustls"), derive(std::fmt::Debug))]
pub(crate) struct TlsConnection {
    host: String,
    addr: SocketAddr,
    config: Arc<Config>,
}

impl TlsConnection {
    pub(crate) fn new(host: String, addr: SocketAddr, config: Arc<Config>) -> Self {
        Self { host, addr, config }
    }
}

pub(crate) struct TlsConnWrapper {
    conn: Object<TlsStream<TcpStream>, Error>,
}
impl TlsConnWrapper {
    pub(crate) fn new(conn: Object<TlsStream<TcpStream>, Error>) -> Self {
        Self { conn }
    }
}

impl AsyncRead for TlsConnWrapper {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut *self.conn).poll_read(cx, buf)
    }
}

impl AsyncWrite for TlsConnWrapper {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut *self.conn).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.conn).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.conn).poll_close(cx)
    }
}

#[async_trait]
impl Manager<TlsStream<TcpStream>, Error> for TlsConnection {
    async fn create(&self) -> Result<TlsStream<TcpStream>, Error> {
        let raw_stream = async_std::net::TcpStream::connect(self.addr).await?;

        raw_stream.set_nodelay(self.config.tcp_no_delay)?;

        let tls_stream = add_tls(&self.host, raw_stream, &self.config).await?;
        Ok(tls_stream)
    }

    async fn recycle(&self, conn: &mut TlsStream<TcpStream>) -> RecycleResult<Error> {
        let mut buf = [0; 4];
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        conn.get_ref()
            .set_nodelay(self.config.tcp_no_delay)
            .map_err(Error::from)?;

        match Pin::new(conn).poll_read(&mut cx, &mut buf) {
            Poll::Ready(Err(error)) => Err(error),
            Poll::Ready(Ok(bytes)) if bytes == 0 => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "connection appeared to be closed (EoF)",
            )),
            _ => Ok(()),
        }
        .map_err(Error::from)?;

        Ok(())
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "rustls")] {
        #[allow(unused_variables)]
        pub(crate) async fn add_tls(host: &str, stream: TcpStream, config: &Config) -> Result<TlsStream<TcpStream>, std::io::Error> {
            let connector = if let Some(tls_config) = config.tls_config.as_ref().cloned() {
                tls_config.into()
            } else {
                async_tls::TlsConnector::default()
            };

            connector.connect(host, stream).await
        }
    } else if #[cfg(feature = "native-tls")] {
        #[allow(unused_variables)]
        pub(crate) async fn add_tls(
            host: &str,
            stream: TcpStream,
            config: &Config,
        ) -> Result<TlsStream<TcpStream>, async_native_tls::Error> {
            let connector = config.tls_config.as_ref().cloned().unwrap_or_default();

            connector.connect(host, stream).await
        }
    }
}
