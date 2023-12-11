use async_std::io;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::channel;
use async_std::task;
use async_tls::{TlsAcceptor, TlsConnector};
use lazy_static::lazy_static;
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{ClientConfig, ServerConfig};
use std::io::{BufReader, Cursor};
use std::net::SocketAddr;
use std::sync::Arc;

const CERT: &str = include_str!("end.cert");
const CHAIN: &str = include_str!("end.chain");
const RSA: &str = include_str!("end.rsa");

lazy_static! {
    static ref TEST_SERVER: (SocketAddr, &'static str, &'static str) = {
        let cert = certs(&mut BufReader::new(Cursor::new(CERT))).unwrap();
        let mut keys = rsa_private_keys(&mut BufReader::new(Cursor::new(RSA))).unwrap();

        let mut config = ServerConfig::new(rustls::NoClientAuth::new());
        config
            .set_single_cert(cert, keys.pop().unwrap())
            .expect("invalid key or certificate");
        let acceptor = TlsAcceptor::from(Arc::new(config));

        let (send, recv) = channel(1);

        task::spawn(async move {
            let addr = SocketAddr::from(([127, 0, 0, 1], 0));
            let listener = TcpListener::bind(&addr).await?;

            send.send(listener.local_addr()?).await;

            let mut incoming = listener.incoming();
            while let Some(stream) = incoming.next().await {
                let acceptor = acceptor.clone();
                task::spawn(async move {
                    use futures_util::io::AsyncReadExt;
                    let stream = acceptor.accept(stream?).await?;
                    let (mut reader, mut writer) = stream.split();
                    io::copy(&mut reader, &mut writer).await?;
                    Ok(()) as io::Result<()>
                });
            }

            Ok(()) as io::Result<()>
        });

        let addr = task::block_on(async move { recv.recv().await.unwrap() });
        (addr, "localhost", CHAIN)
    };
}

fn start_server() -> &'static (SocketAddr, &'static str, &'static str) {
    &*TEST_SERVER
}

async fn start_client(addr: SocketAddr, domain: &str, config: Arc<ClientConfig>) -> io::Result<()> {
    const FILE: &'static [u8] = include_bytes!("../README.md");

    let config = TlsConnector::from(config);
    let mut buf = vec![0; FILE.len()];

    let stream = TcpStream::connect(&addr).await?;
    let mut stream = config.connect(domain, stream).await?;
    stream.write_all(FILE).await?;
    stream.read_exact(&mut buf).await?;

    assert_eq!(buf, FILE);

    stream.flush().await?;
    Ok(())
}

#[test]
fn pass() {
    let (addr, domain, chain) = start_server();

    let mut config = ClientConfig::new();
    let mut chain = BufReader::new(Cursor::new(chain));
    config.root_store.add_pem_file(&mut chain).unwrap();
    let config = Arc::new(config);

    task::block_on(start_client(addr.clone(), domain, config.clone())).unwrap();
}

#[test]
fn fail() {
    let (addr, domain, chain) = start_server();

    let mut config = ClientConfig::new();
    let mut chain = BufReader::new(Cursor::new(chain));
    config.root_store.add_pem_file(&mut chain).unwrap();
    let config = Arc::new(config);

    assert_ne!(domain, &"google.com");
    assert!(task::block_on(start_client(addr.clone(), "google.com", config)).is_err());
}
