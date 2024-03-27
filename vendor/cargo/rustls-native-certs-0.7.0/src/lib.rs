//! rustls-native-certs allows rustls to use the platform's native certificate
//! store when operating as a TLS client.
//!
//! It provides a single function [`load_native_certs()`], which returns a
//! collection of certificates found by reading the platform-native
//! certificate store.
//!
//! If the SSL_CERT_FILE environment variable is set, certificates (in PEM
//! format) are read from that file instead.
//!
//! If you want to load these certificates into a `rustls::RootCertStore`,
//! you'll likely want to do something like this:
//!
//! ```no_run
//! let mut roots = rustls::RootCertStore::empty();
//! for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
//!     roots.add(cert).unwrap();
//! }
//! ```

// Enable documentation for all features on docs.rs
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(all(unix, not(target_os = "macos")))]
mod unix;
#[cfg(all(unix, not(target_os = "macos")))]
use unix as platform;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as platform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use pki_types::CertificateDer;

/// Load root certificates found in the platform's native certificate store.
///
/// If the SSL_CERT_FILE environment variable is set, certificates (in PEM
/// format) are read from that file instead.
///
/// This function fails in a platform-specific way, expressed in a `std::io::Error`.
///
/// This function can be expensive: on some platforms it involves loading
/// and parsing a ~300KB disk file.  It's therefore prudent to call
/// this sparingly.
pub fn load_native_certs() -> Result<Vec<CertificateDer<'static>>, Error> {
    load_certs_from_env().unwrap_or_else(platform::load_native_certs)
}

const ENV_CERT_FILE: &str = "SSL_CERT_FILE";

/// Returns None if SSL_CERT_FILE is not defined in the current environment.
///
/// If it is defined, it is always used, so it must be a path to a real
/// file from which certificates can be loaded successfully.
fn load_certs_from_env() -> Option<Result<Vec<CertificateDer<'static>>, Error>> {
    let cert_var_path = PathBuf::from(env::var_os(ENV_CERT_FILE)?);

    Some(load_pem_certs(&cert_var_path))
}

fn load_pem_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, Error> {
    let mut f = BufReader::new(File::open(path)?);
    rustls_pemfile::certs(&mut f)
        .map(|result| match result {
            Ok(der) => Ok(der),
            Err(err) => Err(Error::new(
                ErrorKind::InvalidData,
                format!("could not load PEM file {path:?}: {err}"),
            )),
        })
        .collect()
}
