#![cfg(feature = "early-data")]

use std::io::{self, BufReader, Cursor, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;

use futures_util::{future::Future, ready};
use rustls::{self, ClientConfig, RootCertStore, ServerConfig, ServerConnection, Stream};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::{client::TlsStream, TlsConnector};

struct Read1<T>(T);

impl<T: AsyncRead + Unpin> Future for Read1<T> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut buf = [0];
        let mut buf = ReadBuf::new(&mut buf);

        ready!(Pin::new(&mut self.0).poll_read(cx, &mut buf))?;

        if buf.filled().is_empty() {
            Poll::Ready(Ok(()))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

async fn send(
    config: Arc<ClientConfig>,
    addr: SocketAddr,
    data: &[u8],
    vectored: bool,
) -> io::Result<(TlsStream<TcpStream>, Vec<u8>)> {
    let connector = TlsConnector::from(config).early_data(true);
    let stream = TcpStream::connect(&addr).await?;
    let domain = pki_types::ServerName::try_from("foobar.com").unwrap();

    let mut stream = connector.connect(domain, stream).await?;
    utils::write(&mut stream, data, vectored).await?;
    stream.flush().await?;
    stream.shutdown().await?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;

    Ok((stream, buf))
}

#[tokio::test]
async fn test_0rtt() -> io::Result<()> {
    test_0rtt_impl(false).await
}

#[tokio::test]
async fn test_0rtt_vectored() -> io::Result<()> {
    test_0rtt_impl(true).await
}

async fn test_0rtt_impl(vectored: bool) -> io::Result<()> {
    let cert_chain = rustls_pemfile::certs(&mut Cursor::new(include_bytes!("end.cert")))
        .collect::<io::Result<Vec<_>>>()?;
    let key_der =
        rustls_pemfile::private_key(&mut Cursor::new(include_bytes!("end.rsa")))?.unwrap();
    let mut server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)
        .unwrap();
    server.max_early_data_size = 8192;
    let server = Arc::new(server);

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let server_port = listener.local_addr().unwrap().port();
    thread::spawn(move || loop {
        let (mut sock, _addr) = listener.accept().unwrap();

        let server = Arc::clone(&server);
        thread::spawn(move || {
            let mut conn = ServerConnection::new(server).unwrap();
            conn.complete_io(&mut sock).unwrap();

            if let Some(mut early_data) = conn.early_data() {
                let mut buf = Vec::new();
                early_data.read_to_end(&mut buf).unwrap();
                let mut stream = Stream::new(&mut conn, &mut sock);
                stream.write_all(b"EARLY:").unwrap();
                stream.write_all(&buf).unwrap();
            }

            let mut stream = Stream::new(&mut conn, &mut sock);
            stream.write_all(b"LATE:").unwrap();
            loop {
                let mut buf = [0; 1024];
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    conn.send_close_notify();
                    conn.complete_io(&mut sock).unwrap();
                    break;
                }
                stream.write_all(&buf[..n]).unwrap();
            }
        });
    });

    let mut chain = BufReader::new(Cursor::new(include_str!("end.chain")));
    let mut root_store = RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut chain) {
        root_store.add(cert.unwrap()).unwrap();
    }

    let mut config =
        rustls::ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
            .with_root_certificates(root_store)
            .with_no_client_auth();
    config.enable_early_data = true;
    let config = Arc::new(config);
    let addr = SocketAddr::from(([127, 0, 0, 1], server_port));

    let (io, buf) = send(config.clone(), addr, b"hello", vectored).await?;
    assert!(!io.get_ref().1.is_early_data_accepted());
    assert_eq!("LATE:hello", String::from_utf8_lossy(&buf));

    let (io, buf) = send(config, addr, b"world!", vectored).await?;
    assert!(io.get_ref().1.is_early_data_accepted());
    assert_eq!("EARLY:world!LATE:", String::from_utf8_lossy(&buf));

    Ok(())
}

// Include `utils` module
include!("utils.rs");
