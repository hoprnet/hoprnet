//! Utility methods for instantiating common connectors for clients.
#[cfg(all(
    any(target_os = "macos", target_os = "windows", target_os = "ios"),
    feature = "tls"
))]
use std::convert::From as _;
#[cfg(all(
    not(any(target_os = "macos", target_os = "windows", target_os = "ios")),
    feature = "tls"
))]
use std::path::{Path, PathBuf};

/// HTTP Connector construction
#[derive(Debug)]
pub struct Connector;

impl Connector {
    /// Alows building a HTTP(S) connector. Used for instantiating clients with custom
    /// connectors.
    pub fn builder() -> Builder {
        Builder {}
    }
}

/// Builder for HTTP(S) connectors
#[derive(Debug)]
pub struct Builder {}

impl Builder {
    /// Use HTTPS instead of HTTP
    #[cfg(feature = "tls")]
    pub fn https(self) -> HttpsBuilder {
        HttpsBuilder {
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
            server_cert: None,
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
            client_cert: None,
        }
    }

    /// Build a HTTP connector
    #[cfg(feature = "tcp")]
    pub fn build(self) -> hyper::client::connect::HttpConnector {
        hyper::client::connect::HttpConnector::new()
    }
}

/// Builder for HTTPS connectors
#[cfg(feature = "tls")]
#[derive(Debug)]
pub struct HttpsBuilder {
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    server_cert: Option<PathBuf>,
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    client_cert: Option<(PathBuf, PathBuf)>,
}

#[cfg(feature = "tls")]
impl HttpsBuilder {
    /// Pin the CA certificate for the server's certificate.
    ///
    /// # Arguments
    ///
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn pin_server_certificate<CA>(mut self, ca_certificate: CA) -> Self
    where
        CA: AsRef<Path>,
    {
        self.server_cert = Some(ca_certificate.as_ref().to_owned());
        self
    }

    /// Provide the Client Certificate and Key for the connection for Mutual TLS
    ///
    /// # Arguments
    ///
    /// * `client_key` - Path to the client private key
    /// * `client_certificate` - Path to the client's public certificate associated with the private key
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn client_authentication<K, C>(mut self, client_key: K, client_certificate: C) -> Self
    where
        K: AsRef<Path>,
        C: AsRef<Path>,
    {
        self.client_cert = Some((
            client_key.as_ref().to_owned(),
            client_certificate.as_ref().to_owned(),
        ));
        self
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    /// Build the HTTPS connector. Will fail if the provided certificates/keys can't be loaded
    /// or the SSL connector can't be created
    pub fn build(
        self,
    ) -> Result<
        hyper_openssl::HttpsConnector<hyper::client::HttpConnector>,
        openssl::error::ErrorStack,
    > {
        // SSL implementation
        let mut ssl = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())?;

        if let Some(ca_certificate) = self.server_cert {
            // Server authentication
            ssl.set_ca_file(ca_certificate)?;
        }

        if let Some((client_key, client_certificate)) = self.client_cert {
            // Client authentication
            ssl.set_private_key_file(client_key, openssl::ssl::SslFiletype::PEM)?;
            ssl.set_certificate_chain_file(client_certificate)?;
            ssl.check_private_key()?;
        }

        let mut connector = hyper::client::HttpConnector::new();
        connector.enforce_http(false);
        hyper_openssl::HttpsConnector::<hyper::client::HttpConnector>::with_connector(
            connector, ssl,
        )
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
    /// Build the HTTPS connector. Will fail if the SSL connector can't be created.
    pub fn build(
        self,
    ) -> Result<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, native_tls::Error> {
        let tls = native_tls::TlsConnector::new()?.into();
        let mut connector = hyper::client::HttpConnector::new();
        connector.enforce_http(false);
        let mut connector = hyper_tls::HttpsConnector::from((connector, tls));
        connector.https_only(true);
        Ok(connector)
    }
}
