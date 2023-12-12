//! Module for combining hyper services
//!
//! Use by passing `hyper::server::MakeService` instances to a `CompositeMakeService`
//! together with the base path for requests that should be handled by that service.
use futures::future::{BoxFuture, FutureExt, TryFutureExt};
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::task::{Context, Poll};

/// Trait for generating a default "not found" response. Must be implemented on
/// the `Response` associated type for `MakeService`s being combined in a
/// `CompositeMakeService`.
pub trait NotFound<V> {
    /// Return a "not found" response
    fn not_found() -> Response<V>;
}

impl<B: Default> NotFound<B> for B {
    fn not_found() -> Response<B> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(B::default())
            .unwrap()
    }
}

/// Connection which has a remote address, which can thus be composited.
pub trait HasRemoteAddr {
    /// Get the remote address for the connection to pass
    /// to the composited service
    fn remote_addr(&self) -> Option<SocketAddr>;
}

impl<'a> HasRemoteAddr for &'a Option<SocketAddr> {
    fn remote_addr(&self) -> Option<SocketAddr> {
        **self
    }
}

impl<'a> HasRemoteAddr for &'a hyper::server::conn::AddrStream {
    fn remote_addr(&self) -> Option<SocketAddr> {
        Some(hyper::server::conn::AddrStream::remote_addr(self))
    }
}

#[cfg(feature = "uds")]
impl<'a> HasRemoteAddr for &'a tokio::net::UnixStream {
    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }
}

/// Trait implemented by services which can be composited.
///
/// Wraps tower_service::Service
pub trait CompositedService<ReqBody, ResBody, Error> {
    /// See tower_service::Service::poll_ready
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>>;
    /// See tower_service::Service::call
    fn call(
        &mut self,
        req: Request<ReqBody>,
    ) -> BoxFuture<'static, Result<Response<ResBody>, Error>>;
}

impl<T, ReqBody, ResBody, Error> CompositedService<ReqBody, ResBody, Error> for T
where
    T: Service<Request<ReqBody>, Response = Response<ResBody>, Error = Error>,
    T::Future: Send + 'static,
{
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Service::poll_ready(self, cx)
    }

    fn call(
        &mut self,
        req: Request<ReqBody>,
    ) -> BoxFuture<'static, Result<Response<ResBody>, Error>> {
        Box::pin(Service::call(self, req))
    }
}

type FutureService<ReqBody, ResBody, Error, MakeError> = BoxFuture<
    'static,
    Result<Box<dyn CompositedService<ReqBody, ResBody, Error> + Send>, MakeError>,
>;

/// Trait implemented by make services which can be composited.
///
/// Wraps tower_service::Service
pub trait CompositedMakeService<Target, ReqBody, ResBody, Error, MakeError> {
    /// See tower_service::Service::poll_ready
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), MakeError>>;
    /// See tower_service::Service::call
    fn call(&mut self, target: Target) -> FutureService<ReqBody, ResBody, Error, MakeError>;
}

impl<T, S, F, Target, ReqBody, ResBody, Error, MakeError>
    CompositedMakeService<Target, ReqBody, ResBody, Error, MakeError> for T
where
    Target: Send,
    T: Service<Target, Response = S, Future = F, Error = MakeError> + Send,
    F: Future<Output = Result<S, MakeError>> + Send + 'static,
    S: CompositedService<ReqBody, ResBody, Error> + Send + 'static,
{
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), MakeError>> {
        Service::poll_ready(self, cx)
    }

    fn call(&mut self, target: Target) -> FutureService<ReqBody, ResBody, Error, MakeError> {
        Box::pin(Service::call(self, target).map(|r| match r {
            Ok(s) => {
                let s: Box<dyn CompositedService<ReqBody, ResBody, Error> + Send> = Box::new(s);
                Ok(s)
            }
            Err(e) => Err(e),
        }))
    }
}

type CompositeServiceVec<ReqBody, ResBody, Error> = Vec<(
    &'static str,
    Box<dyn CompositedService<ReqBody, ResBody, Error> + Send>,
)>;

type CompositeMakeServiceVec<Target, ReqBody, ResBody, Error, MakeError> =
    Vec<CompositeMakeServiceEntry<Target, ReqBody, ResBody, Error, MakeError>>;

/// Service which can be composited with other services as part of a CompositeMakeService
///
/// Consists of a base path for requests which should be handled by this service, and a boxed
/// MakeService.
pub type CompositeMakeServiceEntry<Target, ReqBody, ResBody, Error, MakeError> = (
    &'static str,
    Box<dyn CompositedMakeService<Target, ReqBody, ResBody, Error, MakeError> + Send>,
);

/// Wraps a vector of pairs, each consisting of a base path as a `&'static str`
/// and a `MakeService` instance. Implements `Deref<Vec>` and `DerefMut<Vec>` so
/// these can be manipulated using standard `Vec` methods.
///
/// The `Service` returned by calling `make_service()` will pass an incoming
/// request to the first `Service` in the list for which the associated
/// base path is a prefix of the request path.
///
/// Example Usage
/// =============
///
/// ```ignore
/// let my_make_service1 = MakeService1::new();
/// let my_make_service2 = MakeService2::new();
///
/// let mut composite_make_service = CompositeMakeService::new();
/// composite_make_service.push(("/base/path/1", my_make_service1));
/// composite_make_service.push(("/base/path/2", my_make_service2));
///
/// // use as you would any `MakeService` instance
/// ```
#[derive(Default)]
pub struct CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>(
    CompositeMakeServiceVec<Target, ReqBody, ResBody, Error, MakeError>,
)
where
    ResBody: NotFound<ResBody>;

impl<Target, ReqBody, ResBody, Error, MakeError>
    CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ResBody: NotFound<ResBody>,
{
    /// create an empty `CompositeMakeService`
    pub fn new() -> Self {
        CompositeMakeService(Vec::new())
    }
}

impl<ReqBody, ResBody, Error, MakeError, Connection> Service<Connection>
    for CompositeMakeService<Option<SocketAddr>, ReqBody, ResBody, Error, MakeError>
where
    Connection: HasRemoteAddr,
    ReqBody: 'static,
    ResBody: NotFound<ResBody> + 'static,
    MakeError: Send + 'static,
    Error: 'static,
{
    type Error = MakeError;
    type Response = CompositeService<ReqBody, ResBody, Error>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for service in &mut self.0 {
            match service.1.poll_ready(cx) {
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => {
                    return Poll::Ready(Err(e));
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: Connection) -> Self::Future {
        let mut services = Vec::with_capacity(self.0.len());
        let addr = target.remote_addr();
        for (path, service) in &mut self.0 {
            let path: &'static str = path;
            services.push(service.call(addr).map_ok(move |s| (path, s)));
        }
        Box::pin(futures::future::join_all(services).map(|results| {
            let services: Result<Vec<_>, MakeError> = results.into_iter().collect();

            Ok(CompositeService(services?))
        }))
    }
}

impl<Target, ReqBody, ResBody, Error, MakeError> fmt::Debug
    for CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ResBody: NotFound<ResBody>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // Get vector of base paths
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(
            f,
            "CompositeMakeService accepting base paths: {:?}",
            str_vec,
        )
    }
}

impl<Target, ReqBody, ResBody, Error, MakeError> Deref
    for CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ResBody: NotFound<ResBody>,
{
    type Target = CompositeMakeServiceVec<Target, ReqBody, ResBody, Error, MakeError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Target, ReqBody, ResBody, Error, MakeError> DerefMut
    for CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ResBody: NotFound<ResBody>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Wraps a vector of pairs, each consisting of a base path as a `&'static str`
/// and a `Service` instance.
pub struct CompositeService<ReqBody, ResBody, Error>(CompositeServiceVec<ReqBody, ResBody, Error>)
where
    ResBody: NotFound<ResBody>;

impl<ReqBody, ResBody, Error> Service<Request<ReqBody>>
    for CompositeService<ReqBody, ResBody, Error>
where
    Error: Send + 'static,
    ResBody: NotFound<ResBody> + Send + 'static,
{
    type Error = Error;
    type Response = Response<ResBody>;
    type Future = BoxFuture<'static, Result<Response<ResBody>, Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for service in &mut self.0 {
            match service.1.poll_ready(cx) {
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        for &mut (base_path, ref mut service) in &mut self.0 {
            if req.uri().path().starts_with(base_path) {
                return service.call(req);
            }
        }

        Box::pin(futures::future::ok(ResBody::not_found()))
    }
}

impl<ReqBody, ResBody, Error> fmt::Debug for CompositeService<ReqBody, ResBody, Error>
where
    ResBody: NotFound<ResBody>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // Get vector of base paths
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(f, "CompositeService accepting base paths: {:?}", str_vec,)
    }
}

impl<ReqBody, ResBody, Error> Deref for CompositeService<ReqBody, ResBody, Error>
where
    ResBody: NotFound<ResBody> + 'static,
    Error: 'static,
{
    type Target = CompositeServiceVec<ReqBody, ResBody, Error>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<ReqBody, ResBody, Error> DerefMut for CompositeService<ReqBody, ResBody, Error>
where
    ResBody: NotFound<ResBody> + 'static,
    Error: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
