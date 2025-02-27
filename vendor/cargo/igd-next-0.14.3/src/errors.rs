use std::error;
use std::fmt;
use std::io;
use std::str;
#[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
use std::string::FromUtf8Error;

#[cfg(feature = "aio_tokio")]
use tokio::time::error::Elapsed;

#[cfg(feature = "aio_async_std")]
use async_std::future::TimeoutError;

/// Errors that can occur when sending the request to the gateway.
#[derive(Debug)]
pub enum RequestError {
    /// attohttp error
    AttoHttpError(attohttpc::Error),
    /// IO Error
    IoError(io::Error),
    /// The response from the gateway could not be parsed.
    InvalidResponse(String),
    /// The gateway returned an unhandled error code and description.
    ErrorCode(u16, String),
    /// Action is not supported by the gateway
    UnsupportedAction(String),
    /// When using the aio feature.
    #[cfg(feature = "aio_tokio")]
    HyperError(hyper::Error),

    /// When using aio async std feature
    #[cfg(feature = "aio_async_std")]
    SurfError(surf::Error),

    #[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
    /// http crate error type
    HttpError(http::Error),

    #[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
    /// Error parsing HTTP body
    Utf8Error(FromUtf8Error),
}

impl From<attohttpc::Error> for RequestError {
    fn from(err: attohttpc::Error) -> RequestError {
        RequestError::AttoHttpError(err)
    }
}

impl From<io::Error> for RequestError {
    fn from(err: io::Error) -> RequestError {
        RequestError::IoError(err)
    }
}

#[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
impl From<http::Error> for RequestError {
    fn from(err: http::Error) -> RequestError {
        RequestError::HttpError(err)
    }
}

#[cfg(feature = "aio_async_std")]
impl From<surf::Error> for RequestError {
    fn from(err: surf::Error) -> RequestError {
        RequestError::SurfError(err)
    }
}

#[cfg(feature = "aio_tokio")]
impl From<hyper::Error> for RequestError {
    fn from(err: hyper::Error) -> RequestError {
        RequestError::HyperError(err)
    }
}

#[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
impl From<FromUtf8Error> for RequestError {
    fn from(err: FromUtf8Error) -> RequestError {
        RequestError::Utf8Error(err)
    }
}

#[cfg(any(feature = "aio_async_std"))]
impl From<TimeoutError> for RequestError {
    fn from(_err: TimeoutError) -> RequestError {
        RequestError::IoError(io::Error::new(io::ErrorKind::TimedOut, "timer failed"))
    }
}

#[cfg(any(feature = "aio_tokio"))]
impl From<Elapsed> for RequestError {
    fn from(_err: Elapsed) -> RequestError {
        RequestError::IoError(io::Error::new(io::ErrorKind::TimedOut, "timer failed"))
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RequestError::AttoHttpError(ref e) => write!(f, "HTTP error {e}"),
            RequestError::InvalidResponse(ref e) => write!(f, "Invalid response from gateway: {e}"),
            RequestError::IoError(ref e) => write!(f, "IO error. {e}"),
            RequestError::ErrorCode(n, ref e) => write!(f, "Gateway response error {n}: {e}"),
            RequestError::UnsupportedAction(ref e) => write!(f, "Gateway does not support action: {e}"),
            #[cfg(feature = "aio_async_std")]
            RequestError::SurfError(ref e) => write!(f, "Surf Error: {e}"),
            #[cfg(feature = "aio_tokio")]
            RequestError::HyperError(ref e) => write!(f, "Hyper Error: {e}"),
            #[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
            RequestError::HttpError(ref e) => write!(f, "Http  Error: {e}"),
            #[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
            RequestError::Utf8Error(ref e) => write!(f, "Utf8Error Error: {e}"),
        }
    }
}

impl std::error::Error for RequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            RequestError::AttoHttpError(ref e) => Some(e),
            RequestError::InvalidResponse(..) => None,
            RequestError::IoError(ref e) => Some(e),
            RequestError::ErrorCode(..) => None,
            RequestError::UnsupportedAction(..) => None,
            #[cfg(feature = "aio_async_std")]
            RequestError::SurfError(ref e) => Some(e.as_ref()),
            #[cfg(feature = "aio_tokio")]
            RequestError::HyperError(ref e) => Some(e),
            #[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
            RequestError::HttpError(ref e) => Some(e),
            #[cfg(any(feature = "aio_tokio", feature = "aio_async_std"))]
            RequestError::Utf8Error(ref e) => Some(e),
        }
    }
}

/// Errors returned by `Gateway::get_external_ip`
#[derive(Debug)]
pub enum GetExternalIpError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

/// Errors returned by `Gateway::remove_port`
#[derive(Debug)]
pub enum RemovePortError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// No such port mapping.
    NoSuchPortMapping,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

/// Errors returned by `Gateway::add_any_port` and `Gateway::get_any_address`
#[derive(Debug)]
pub enum AddAnyPortError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// Can not add a mapping for local port 0.
    InternalPortZeroInvalid,
    /// The gateway does not have any free ports.
    NoPortsAvailable,
    /// The gateway can only map internal ports to same-numbered external ports
    /// and this external port is in use.
    ExternalPortInUse,
    /// The gateway only supports permanent leases (ie. a `lease_duration` of 0).
    OnlyPermanentLeasesSupported,
    /// The description was too long for the gateway to handle.
    DescriptionTooLong,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

impl From<RequestError> for AddAnyPortError {
    fn from(err: RequestError) -> AddAnyPortError {
        AddAnyPortError::RequestError(err)
    }
}

impl From<GetExternalIpError> for AddAnyPortError {
    fn from(err: GetExternalIpError) -> AddAnyPortError {
        match err {
            GetExternalIpError::ActionNotAuthorized => AddAnyPortError::ActionNotAuthorized,
            GetExternalIpError::RequestError(e) => AddAnyPortError::RequestError(e),
        }
    }
}

/// Errors returned by `Gateway::add_port`
#[derive(Debug)]
pub enum AddPortError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// Can not add a mapping for local port 0.
    InternalPortZeroInvalid,
    /// External port number 0 (any port) is considered invalid by the gateway.
    ExternalPortZeroInvalid,
    /// The requested mapping conflicts with a mapping assigned to another client.
    PortInUse,
    /// The gateway requires that the requested internal and external ports are the same.
    SamePortValuesRequired,
    /// The gateway only supports permanent leases (ie. a `lease_duration` of 0).
    OnlyPermanentLeasesSupported,
    /// The description was too long for the gateway to handle.
    DescriptionTooLong,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

impl fmt::Display for GetExternalIpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GetExternalIpError::ActionNotAuthorized => write!(f, "The client is not authorized to remove the port"),
            GetExternalIpError::RequestError(ref e) => write!(f, "Request Error. {e}"),
        }
    }
}

impl From<io::Error> for GetExternalIpError {
    fn from(err: io::Error) -> GetExternalIpError {
        GetExternalIpError::RequestError(RequestError::from(err))
    }
}

impl std::error::Error for GetExternalIpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for RemovePortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RemovePortError::ActionNotAuthorized => write!(f, "The client is not authorized to remove the port"),
            RemovePortError::NoSuchPortMapping => write!(f, "The port was not mapped"),
            RemovePortError::RequestError(ref e) => write!(f, "Request error. {e}"),
        }
    }
}

impl std::error::Error for RemovePortError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for AddAnyPortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddAnyPortError::ActionNotAuthorized => {
                write!(f, "The client is not authorized to remove the port")
            }
            AddAnyPortError::InternalPortZeroInvalid => {
                write!(f, "Can not add a mapping for local port 0")
            }
            AddAnyPortError::NoPortsAvailable => {
                write!(f, "The gateway does not have any free ports")
            }
            AddAnyPortError::OnlyPermanentLeasesSupported => {
                write!(
                    f,
                    "The gateway only supports permanent leases (ie. a `lease_duration` of 0),"
                )
            }
            AddAnyPortError::ExternalPortInUse => {
                write!(
                    f,
                    "The gateway can only map internal ports to same-numbered external ports and this external port is in use."
                )
            }
            AddAnyPortError::DescriptionTooLong => {
                write!(f, "The description was too long for the gateway to handle.")
            }
            AddAnyPortError::RequestError(ref e) => write!(f, "Request error. {e}"),
        }
    }
}

impl std::error::Error for AddAnyPortError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for AddPortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddPortError::ActionNotAuthorized => write!(f, "The client is not authorized to map this port."),
            AddPortError::InternalPortZeroInvalid => write!(f, "Can not add a mapping for local port 0"),
            AddPortError::ExternalPortZeroInvalid => write!(
                f,
                "External port number 0 (any port) is considered invalid by the gateway."
            ),
            AddPortError::PortInUse => write!(
                f,
                "The requested mapping conflicts with a mapping assigned to another client."
            ),
            AddPortError::SamePortValuesRequired => write!(
                f,
                "The gateway requires that the requested internal and external ports are the same."
            ),
            AddPortError::OnlyPermanentLeasesSupported => write!(
                f,
                "The gateway only supports permanent leases (ie. a `lease_duration` of 0),"
            ),
            AddPortError::DescriptionTooLong => write!(f, "The description was too long for the gateway to handle."),
            AddPortError::RequestError(ref e) => write!(f, "Request error. {e}"),
        }
    }
}

impl std::error::Error for AddPortError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Errors than can occur while trying to find the gateway.
#[derive(Debug)]
pub enum SearchError {
    /// Http/Hyper error
    HttpError(attohttpc::Error),
    /// Unable to process the response
    InvalidResponse,
    /// IO Error
    IoError(io::Error),
    /// UTF-8 decoding error
    Utf8Error(str::Utf8Error),
    /// XML processing error
    XmlError(xmltree::ParseError),
    /// When using aio_async_std feature
    #[cfg(feature = "aio_async_std")]
    SurfError(surf::Error),
    /// When using the aio feature.
    #[cfg(feature = "aio_tokio")]
    HyperError(hyper::Error),
    /// Error parsing URI
    #[cfg(feature = "aio_tokio")]
    InvalidUri(hyper::http::uri::InvalidUri),
}

impl From<attohttpc::Error> for SearchError {
    fn from(err: attohttpc::Error) -> SearchError {
        SearchError::HttpError(err)
    }
}

impl From<io::Error> for SearchError {
    fn from(err: io::Error) -> SearchError {
        SearchError::IoError(err)
    }
}

impl From<str::Utf8Error> for SearchError {
    fn from(err: str::Utf8Error) -> SearchError {
        SearchError::Utf8Error(err)
    }
}

impl From<xmltree::ParseError> for SearchError {
    fn from(err: xmltree::ParseError) -> SearchError {
        SearchError::XmlError(err)
    }
}

#[cfg(feature = "aio_async_std")]
impl From<surf::Error> for SearchError {
    fn from(err: surf::Error) -> SearchError {
        SearchError::SurfError(err)
    }
}

#[cfg(feature = "aio_tokio")]
impl From<hyper::Error> for SearchError {
    fn from(err: hyper::Error) -> SearchError {
        SearchError::HyperError(err)
    }
}

#[cfg(feature = "aio_tokio")]
impl From<hyper::http::uri::InvalidUri> for SearchError {
    fn from(err: hyper::http::uri::InvalidUri) -> SearchError {
        SearchError::InvalidUri(err)
    }
}

#[cfg(any(feature = "aio_async_std"))]
impl From<TimeoutError> for SearchError {
    fn from(_err: TimeoutError) -> SearchError {
        SearchError::IoError(io::Error::new(io::ErrorKind::TimedOut, "timer failed"))
    }
}

#[cfg(feature = "aio_tokio")]
impl From<Elapsed> for SearchError {
    fn from(_err: Elapsed) -> SearchError {
        SearchError::IoError(io::Error::new(io::ErrorKind::TimedOut, "search timed out"))
    }
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SearchError::HttpError(ref e) => write!(f, "HTTP error {e}"),
            SearchError::InvalidResponse => write!(f, "Invalid response"),
            SearchError::IoError(ref e) => write!(f, "IO error: {e}"),
            SearchError::Utf8Error(ref e) => write!(f, "UTF-8 error: {e}"),
            SearchError::XmlError(ref e) => write!(f, "XML error: {e}"),
            #[cfg(feature = "aio_async_std")]
            SearchError::SurfError(ref e) => write!(f, "Surf Error: {e}"),
            #[cfg(feature = "aio_tokio")]
            SearchError::HyperError(ref e) => write!(f, "Hyper Error: {e}"),
            #[cfg(feature = "aio_tokio")]
            SearchError::InvalidUri(ref e) => write!(f, "InvalidUri Error: {e}"),
        }
    }
}

impl error::Error for SearchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            SearchError::HttpError(ref e) => Some(e),
            SearchError::InvalidResponse => None,
            SearchError::IoError(ref e) => Some(e),
            SearchError::Utf8Error(ref e) => Some(e),
            SearchError::XmlError(ref e) => Some(e),
            #[cfg(feature = "aio_async_std")]
            SearchError::SurfError(ref e) => Some(e.as_ref()),
            #[cfg(feature = "aio_tokio")]
            SearchError::HyperError(ref e) => Some(e),
            #[cfg(feature = "aio_tokio")]
            SearchError::InvalidUri(ref e) => Some(e),
        }
    }
}

/// Errors than can occur while getting a port mapping
#[derive(Debug)]
pub enum GetGenericPortMappingEntryError {
    /// The client is not authorized to perform the operation.
    ActionNotAuthorized,
    /// The specified array index is out of bounds.
    SpecifiedArrayIndexInvalid,
    /// Some other error occured performing the request.
    RequestError(RequestError),
}

impl From<RequestError> for GetGenericPortMappingEntryError {
    fn from(err: RequestError) -> GetGenericPortMappingEntryError {
        match err {
            RequestError::ErrorCode(code, _) if code == 606 => GetGenericPortMappingEntryError::ActionNotAuthorized,
            RequestError::ErrorCode(code, _) if code == 713 => {
                GetGenericPortMappingEntryError::SpecifiedArrayIndexInvalid
            }
            other => GetGenericPortMappingEntryError::RequestError(other),
        }
    }
}

impl fmt::Display for GetGenericPortMappingEntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GetGenericPortMappingEntryError::ActionNotAuthorized => {
                write!(f, "The client is not authorized to look up port mappings.")
            }
            GetGenericPortMappingEntryError::SpecifiedArrayIndexInvalid => {
                write!(f, "The provided index into the port mapping list is invalid.")
            }
            GetGenericPortMappingEntryError::RequestError(ref e) => e.fmt(f),
        }
    }
}

impl std::error::Error for GetGenericPortMappingEntryError {}

/// An error type that emcompasses all possible errors.
#[derive(Debug)]
pub enum Error {
    /// `AddAnyPortError`
    AddAnyPortError(AddAnyPortError),
    /// `AddPortError`
    AddPortError(AddPortError),
    /// `GetExternalIpError`
    GetExternalIpError(GetExternalIpError),
    /// `RemovePortError`
    RemovePortError(RemovePortError),
    /// `RequestError`
    RequestError(RequestError),
    /// `SearchError`
    SearchError(SearchError),
}

/// A result type where the error is `igd::Error`.
pub type Result<T = ()> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::AddAnyPortError(ref e) => e.fmt(f),
            Error::AddPortError(ref e) => e.fmt(f),
            Error::GetExternalIpError(ref e) => e.fmt(f),
            Error::RemovePortError(ref e) => e.fmt(f),
            Error::RequestError(ref e) => e.fmt(f),
            Error::SearchError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::AddAnyPortError(ref e) => Some(e),
            Error::AddPortError(ref e) => Some(e),
            Error::GetExternalIpError(ref e) => Some(e),
            Error::RemovePortError(ref e) => Some(e),
            Error::RequestError(ref e) => Some(e),
            Error::SearchError(ref e) => Some(e),
        }
    }
}

impl From<AddAnyPortError> for Error {
    fn from(err: AddAnyPortError) -> Error {
        Error::AddAnyPortError(err)
    }
}

impl From<AddPortError> for Error {
    fn from(err: AddPortError) -> Error {
        Error::AddPortError(err)
    }
}

impl From<GetExternalIpError> for Error {
    fn from(err: GetExternalIpError) -> Error {
        Error::GetExternalIpError(err)
    }
}

impl From<RemovePortError> for Error {
    fn from(err: RemovePortError) -> Error {
        Error::RemovePortError(err)
    }
}

impl From<RequestError> for Error {
    fn from(err: RequestError) -> Error {
        Error::RequestError(err)
    }
}

impl From<SearchError> for Error {
    fn from(err: SearchError) -> Error {
        Error::SearchError(err)
    }
}
