//! Configuration for `HttpClient`s.

use std::fmt::Debug;
use std::time::Duration;

/// Configuration for `HttpClient`s.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    /// HTTP/1.1 `keep-alive` (connection pooling).
    ///
    /// Default: `true`.
    ///
    /// Note: Does nothing on `wasm_client`.
    pub http_keep_alive: bool,
    /// TCP `NO_DELAY`.
    ///
    /// Default: `false`.
    ///
    /// Note: Does nothing on `wasm_client`.
    pub tcp_no_delay: bool,
    /// Connection timeout duration.
    ///
    /// Default: `Some(Duration::from_secs(60))`.
    pub timeout: Option<Duration>,
    /// Maximum number of simultaneous connections that this client is allowed to keep open to individual hosts at one time.
    ///
    /// Default: `50`.
    /// This number is based on a few random benchmarks and see whatever gave decent perf vs resource use in Orogene.
    ///
    /// Note: The behavior of this is different depending on the backend in use.
    /// - `h1_client`: `0` is disallowed and asserts as otherwise it would cause a semaphore deadlock.
    /// - `curl_client`: `0` allows for limitless connections per host.
    /// - `hyper_client`: No effect. Hyper does not support such an option.
    /// - `wasm_client`: No effect. Web browsers do not support such an option.
    pub max_connections_per_host: usize,
    /// TLS Configuration (Rustls)
    #[cfg_attr(feature = "docs", doc(cfg(feature = "h1_client")))]
    #[cfg(all(feature = "h1_client", feature = "rustls"))]
    pub tls_config: Option<std::sync::Arc<rustls_crate::ClientConfig>>,
    /// TLS Configuration (Native TLS)
    #[cfg_attr(feature = "docs", doc(cfg(feature = "h1_client")))]
    #[cfg(all(feature = "h1_client", feature = "native-tls", not(feature = "rustls")))]
    pub tls_config: Option<std::sync::Arc<async_native_tls::TlsConnector>>,
}

impl Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg_struct = f.debug_struct("Config");
        dbg_struct
            .field("http_keep_alive", &self.http_keep_alive)
            .field("tcp_no_delay", &self.tcp_no_delay)
            .field("timeout", &self.timeout)
            .field("max_connections_per_host", &self.max_connections_per_host);

        #[cfg(all(feature = "h1_client", feature = "rustls"))]
        {
            if self.tls_config.is_some() {
                dbg_struct.field("tls_config", &"Some(rustls::ClientConfig)");
            } else {
                dbg_struct.field("tls_config", &"None");
            }
        }
        #[cfg(all(feature = "h1_client", feature = "native-tls", not(feature = "rustls")))]
        {
            dbg_struct.field("tls_config", &self.tls_config);
        }

        dbg_struct.finish()
    }
}

impl Config {
    /// Construct new empty config.
    pub fn new() -> Self {
        Self {
            http_keep_alive: true,
            tcp_no_delay: false,
            timeout: Some(Duration::from_secs(60)),
            max_connections_per_host: 50,
            #[cfg(all(feature = "h1_client", any(feature = "rustls", feature = "native-tls")))]
            tls_config: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Set HTTP/1.1 `keep-alive` (connection pooling).
    pub fn set_http_keep_alive(mut self, keep_alive: bool) -> Self {
        self.http_keep_alive = keep_alive;
        self
    }

    /// Set TCP `NO_DELAY`.
    pub fn set_tcp_no_delay(mut self, no_delay: bool) -> Self {
        self.tcp_no_delay = no_delay;
        self
    }

    /// Set connection timeout duration.
    pub fn set_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum number of simultaneous connections that this client is allowed to keep open to individual hosts at one time.
    pub fn set_max_connections_per_host(mut self, max_connections_per_host: usize) -> Self {
        self.max_connections_per_host = max_connections_per_host;
        self
    }

    /// Set TLS Configuration (Rustls)
    #[cfg_attr(feature = "docs", doc(cfg(feature = "h1_client")))]
    #[cfg(all(feature = "h1_client", feature = "rustls"))]
    pub fn set_tls_config(
        mut self,
        tls_config: Option<std::sync::Arc<rustls_crate::ClientConfig>>,
    ) -> Self {
        self.tls_config = tls_config;
        self
    }
    /// Set TLS Configuration (Native TLS)
    #[cfg_attr(feature = "docs", doc(cfg(feature = "h1_client")))]
    #[cfg(all(feature = "h1_client", feature = "native-tls", not(feature = "rustls")))]
    pub fn set_tls_config(
        mut self,
        tls_config: Option<std::sync::Arc<async_native_tls::TlsConnector>>,
    ) -> Self {
        self.tls_config = tls_config;
        self
    }
}
