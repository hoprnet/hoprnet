mod utils {
    use std::io::{BufReader, Cursor};
    use std::sync::Arc;

    use rustls::{ClientConfig, RootCertStore, ServerConfig};
    use rustls_pemfile::{certs, rsa_private_keys};

    #[allow(dead_code)]
    pub fn make_configs() -> (Arc<ServerConfig>, Arc<ClientConfig>) {
        const CERT: &str = include_str!("end.cert");
        const CHAIN: &str = include_str!("end.chain");
        const RSA: &str = include_str!("end.rsa");

        let cert = certs(&mut BufReader::new(Cursor::new(CERT)))
            .map(|result| result.unwrap())
            .collect();
        let key = rsa_private_keys(&mut BufReader::new(Cursor::new(RSA)))
            .next()
            .unwrap()
            .unwrap();
        let sconfig = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key.into())
            .unwrap();

        let mut client_root_cert_store = RootCertStore::empty();
        let mut chain = BufReader::new(Cursor::new(CHAIN));
        for cert in certs(&mut chain) {
            client_root_cert_store.add(cert.unwrap()).unwrap();
        }

        let cconfig = ClientConfig::builder()
            .with_root_certificates(client_root_cert_store)
            .with_no_client_auth();

        (Arc::new(sconfig), Arc::new(cconfig))
    }
}
