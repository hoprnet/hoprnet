use futures_rustls::{TlsAcceptor, TlsConnector};
use futures_util::future::TryFutureExt;
use futures_util::io::{copy, AsyncReadExt, AsyncWriteExt};
use lazy_static::lazy_static;
use rustls::ClientConfig;
use rustls_pemfile::{certs, private_key};
use smol::net::{TcpListener, TcpStream};
use smol::Timer;
use std::convert::TryFrom;
use std::io::{BufReader, Cursor};
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{io, thread};

const CERT: &str = include_str!("end.cert");
const CHAIN: &[u8] = include_bytes!("end.chain");
const RSA: &str = include_str!("end.rsa");

lazy_static! {
    static ref TEST_SERVER: (SocketAddr, &'static str, &'static [u8]) = {
        let cert = certs(&mut BufReader::new(Cursor::new(CERT)))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let key = private_key(&mut BufReader::new(Cursor::new(RSA)))
            .unwrap()
            .unwrap();

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key)
            .unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(config));

        let (send, recv) = channel();

        thread::spawn(move || {
            let done = async move {
                let addr = SocketAddr::from(([127, 0, 0, 1], 0));
                let listener = TcpListener::bind(&addr).await?;

                send.send(listener.local_addr()?).unwrap();

                loop {
                    let (stream, _) = listener.accept().await?;

                    let acceptor = acceptor.clone();
                    let fut = async move {
                        let stream = acceptor.accept(stream).await?;

                        let (mut reader, mut writer) = stream.split();
                        copy(&mut reader, &mut writer).await?;

                        Ok(()) as io::Result<()>
                    }
                    .unwrap_or_else(|err| eprintln!("server: {:?}", err));

                    smol::spawn(fut).detach();
                }
            }
            .unwrap_or_else(|err: io::Error| eprintln!("server: {:?}", err));

            smol::block_on(done);
        });

        let addr = recv.recv().unwrap();
        (addr, "testserver.com", CHAIN)
    };
}

fn start_server() -> &'static (SocketAddr, &'static str, &'static [u8]) {
    &*TEST_SERVER
}

async fn start_client(addr: SocketAddr, domain: &str, config: Arc<ClientConfig>) -> io::Result<()> {
    const FILE: &[u8] = include_bytes!("../README.md");

    let domain = pki_types::ServerName::try_from(domain).unwrap().to_owned();
    let config = TlsConnector::from(config);
    let mut buf = vec![0; FILE.len()];

    let stream = TcpStream::connect(&addr).await?;
    let mut stream = config.connect(domain, stream).await?;
    stream.write_all(FILE).await?;
    stream.flush().await?;
    stream.read_exact(&mut buf).await?;

    assert_eq!(buf, FILE);

    Ok(())
}

#[test]
fn pass() -> io::Result<()> {
    let fut = async {
        let (addr, domain, chain) = start_server();

        // TODO: not sure how to resolve this right now but since
        // TcpStream::bind now returns a future it creates a race
        // condition until its ready sometimes.
        use std::time::*;
        Timer::after(Duration::from_secs(1)).await;

        let chain = certs(&mut std::io::Cursor::new(*chain))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_parsable_certificates(chain);

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let config = Arc::new(config);

        start_client(*addr, domain, config).await?;

        Ok(())
    };

    smol::block_on(fut)
}

#[test]
fn fail() -> io::Result<()> {
    let fut = async {
        let (addr, domain, chain) = start_server();

        let chain = certs(&mut std::io::Cursor::new(*chain))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_parsable_certificates(chain);
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let config = Arc::new(config);

        assert_ne!(domain, &"google.com");
        let ret = start_client(*addr, "google.com", config).await;
        assert!(ret.is_err());

        Ok(())
    };

    smol::block_on(fut)
}
