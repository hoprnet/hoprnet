use async_h1::{
    client::Encoder,
    server::{ConnectionStatus, Server},
};
use async_std::io::{Read as AsyncRead, Write as AsyncWrite};
use http_types::{Request, Response, Result};
use std::{
    fmt::{Debug, Display},
    future::Future,
    io,
    pin::Pin,
    sync::RwLock,
    task::{Context, Poll, Waker},
};

use async_dup::Arc;

#[pin_project::pin_project]
pub struct TestServer<F, Fut> {
    server: Server<TestIO, F, Fut>,
    #[pin]
    client: TestIO,
}

impl<F, Fut> TestServer<F, Fut>
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Result<Response>>,
{
    #[allow(dead_code)]
    pub fn new(f: F) -> Self {
        let (client, server) = TestIO::new();
        Self {
            server: Server::new(server, f),
            client,
        }
    }

    #[allow(dead_code)]
    pub async fn accept_one(&mut self) -> http_types::Result<ConnectionStatus> {
        self.server.accept_one().await
    }

    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.client.close();
    }

    #[allow(dead_code)]
    pub fn all_read(&self) -> bool {
        self.client.all_read()
    }

    #[allow(dead_code)]
    pub async fn write_request(&mut self, request: Request) -> io::Result<()> {
        async_std::io::copy(&mut Encoder::new(request), self).await?;
        Ok(())
    }
}

impl<F, Fut> AsyncRead for TestServer<F, Fut>
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Result<Response>>,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().client.poll_read(cx, buf)
    }
}

impl<F, Fut> AsyncWrite for TestServer<F, Fut>
where
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Result<Response>>,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().client.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().client.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().client.poll_close(cx)
    }
}

/// a Test IO
#[derive(Default, Clone, Debug)]
pub struct TestIO {
    pub read: Arc<CloseableCursor>,
    pub write: Arc<CloseableCursor>,
}

#[derive(Default)]
struct CloseableCursorInner {
    data: Vec<u8>,
    cursor: usize,
    waker: Option<Waker>,
    closed: bool,
}

#[derive(Default)]
pub struct CloseableCursor(RwLock<CloseableCursorInner>);

impl CloseableCursor {
    pub fn len(&self) -> usize {
        self.0.read().unwrap().data.len()
    }

    pub fn cursor(&self) -> usize {
        self.0.read().unwrap().cursor
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn current(&self) -> bool {
        let inner = self.0.read().unwrap();
        inner.data.len() == inner.cursor
    }

    pub fn close(&self) {
        let mut inner = self.0.write().unwrap();
        inner.closed = true;
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }
}

impl Display for CloseableCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.0.read().unwrap();
        let s = std::str::from_utf8(&inner.data).unwrap_or("not utf8");
        write!(f, "{}", s)
    }
}

impl TestIO {
    pub fn new() -> (TestIO, TestIO) {
        let client = Arc::new(CloseableCursor::default());
        let server = Arc::new(CloseableCursor::default());

        (
            TestIO {
                read: client.clone(),
                write: server.clone(),
            },
            TestIO {
                read: server,
                write: client,
            },
        )
    }

    pub fn all_read(&self) -> bool {
        self.write.current()
    }

    pub fn close(&mut self) {
        self.write.close();
    }
}

impl Debug for CloseableCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.0.read().unwrap();
        f.debug_struct("CloseableCursor")
            .field(
                "data",
                &std::str::from_utf8(&inner.data).unwrap_or("not utf8"),
            )
            .field("closed", &inner.closed)
            .field("cursor", &inner.cursor)
            .finish()
    }
}

impl AsyncRead for CloseableCursor {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self).poll_read(cx, buf)
    }
}

impl AsyncRead for &CloseableCursor {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut inner = self.0.write().unwrap();
        if inner.cursor < inner.data.len() {
            let bytes_to_copy = buf.len().min(inner.data.len() - inner.cursor);
            buf[..bytes_to_copy]
                .copy_from_slice(&inner.data[inner.cursor..inner.cursor + bytes_to_copy]);
            inner.cursor += bytes_to_copy;
            Poll::Ready(Ok(bytes_to_copy))
        } else if inner.closed {
            Poll::Ready(Ok(0))
        } else {
            inner.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl AsyncWrite for &CloseableCursor {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut inner = self.0.write().unwrap();
        if inner.closed {
            Poll::Ready(Ok(0))
        } else {
            inner.data.extend_from_slice(buf);
            if let Some(waker) = inner.waker.take() {
                waker.wake();
            }
            Poll::Ready(Ok(buf.len()))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.close();
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for TestIO {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.read).poll_read(cx, buf)
    }
}

impl AsyncWrite for TestIO {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.write).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.write).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.write).poll_close(cx)
    }
}

impl std::io::Write for CloseableCursor {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write().unwrap().data.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
