use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use async_std::net::TcpStream;
use async_trait::async_trait;
use deadpool::managed::{Manager, Object, RecycleResult};
use futures::io::{AsyncRead, AsyncWrite};
use futures::task::{Context, Poll};

use crate::Config;

#[derive(Clone)]
#[cfg_attr(not(feature = "rustls"), derive(std::fmt::Debug))]
pub(crate) struct TcpConnection {
    addr: SocketAddr,
    config: Arc<Config>,
}

impl TcpConnection {
    pub(crate) fn new(addr: SocketAddr, config: Arc<Config>) -> Self {
        Self { addr, config }
    }
}

pub(crate) struct TcpConnWrapper {
    conn: Object<TcpStream, std::io::Error>,
}
impl TcpConnWrapper {
    pub(crate) fn new(conn: Object<TcpStream, std::io::Error>) -> Self {
        Self { conn }
    }
}

impl AsyncRead for TcpConnWrapper {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut *self.conn).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpConnWrapper {
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
impl Manager<TcpStream, std::io::Error> for TcpConnection {
    async fn create(&self) -> Result<TcpStream, std::io::Error> {
        let tcp_stream = TcpStream::connect(self.addr).await?;

        tcp_stream.set_nodelay(self.config.tcp_no_delay)?;

        Ok(tcp_stream)
    }

    async fn recycle(&self, conn: &mut TcpStream) -> RecycleResult<std::io::Error> {
        let mut buf = [0; 4];
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        conn.set_nodelay(self.config.tcp_no_delay)?;

        match Pin::new(conn).poll_read(&mut cx, &mut buf) {
            Poll::Ready(Err(error)) => Err(error),
            Poll::Ready(Ok(bytes)) if bytes == 0 => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "connection appeared to be closed (EoF)",
            )),
            _ => Ok(()),
        }?;
        Ok(())
    }
}
