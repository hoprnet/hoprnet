/// Errors returned by this library
use std::fmt;
use std::io;
use url::ParseError;

#[cfg(feature = "remote_list")]
use std::net::TcpStream;

#[cfg(feature = "remote_list")]
use native_tls::{Error as TlsError, HandshakeError};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(
    /// The kind of the error.
    pub ErrorKind,
);
impl std::error::Error for Error {}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
/// The kind of an error.
pub enum ErrorKind {
    Io(::std::io::Error),
    Url(::url::ParseError),
    #[cfg(feature = "remote_list")]
    Tls(::native_tls::Error),
    #[cfg(feature = "remote_list")]
    Handshake(HandshakeError<TcpStream>),
    /// A convenient variant for String.
    Msg(String),
    UnsupportedScheme,
    InvalidList,
    NoHost,
    NoPort,
    InvalidHost,
    InvalidEmail,
    InvalidRule(String),
    InvalidDomain(String),
    Uts46(::idna::Errors),
    #[doc(hidden)]
    __Nonexhaustive {},
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Io(e) => write!(f, "{}", e),
            ErrorKind::Url(e) => write!(f, "{}", e),
            #[cfg(feature = "remote_list")]
            ErrorKind::Tls(e) => write!(f, "{}", e),
            #[cfg(feature = "remote_list")]
            ErrorKind::Handshake(e) => write!(f, "{}", e),
            ErrorKind::Msg(e) => write!(f, "{}", e),
            ErrorKind::UnsupportedScheme => write!(f, "UnsupportedScheme"),
            ErrorKind::InvalidList => write!(f, "InvalidList"),
            ErrorKind::NoHost => write!(f, "NoHost"),
            ErrorKind::NoPort => write!(f, "NoPort"),
            ErrorKind::InvalidHost => write!(f, "InvalidHost"),
            ErrorKind::InvalidEmail => write!(f, "InvalidEmail"),
            ErrorKind::InvalidRule(t) => write!(f, "invalid rule: '{}'", t),
            ErrorKind::InvalidDomain(t) => write!(f, "invalid domain: '{}'", t),
            ErrorKind::Uts46(e) => write!(f, "UTS #46 processing error: '{:?}'", e),
            ErrorKind::__Nonexhaustive {} => write!(f, "__Nonexhaustive"),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error(kind)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error(ErrorKind::Io(e))
    }
}

impl From<ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error(ErrorKind::Url(e))
    }
}

#[cfg(feature = "remote_list")]
impl From<TlsError> for Error {
    fn from(e: TlsError) -> Self {
        Error(ErrorKind::Tls(e))
    }
}

#[cfg(feature = "remote_list")]
impl From<HandshakeError<TcpStream>> for Error {
    fn from(e: HandshakeError<TcpStream>) -> Self {
        Error(ErrorKind::Handshake(e))
    }
}
