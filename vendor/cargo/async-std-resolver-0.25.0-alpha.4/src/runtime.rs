// Copyright 2015-2020 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// https://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

use async_std::future::timeout;
use socket2::{Domain, Protocol, Socket, Type};

use crate::net::{AsyncStdTcpStream, AsyncStdUdpSocket};
use crate::proto::runtime::{Executor, RuntimeProvider, Spawn};
use crate::proto::udp::UdpSocket;
use crate::proto::ProtoError;
use crate::time::AsyncStdTime;
use hickory_resolver::config::{NameServerConfig, ResolverOpts};
use hickory_resolver::name_server::{ConnectionProvider, GenericConnector};

/// The async_std runtime.
///
/// The runtime provides a task scheduler, [timer], and blocking
/// pool, necessary for running asynchronous tasks.
///
/// Instances of `AsyncStdRuntime` can be created using [`new`]. However, most
/// users will use the `#[async_std::main]` annotation on their entry point instead.
///
/// See [module level][mod] documentation for more details.
///
/// # Shutdown
///
/// Shutting down the runtime is done by dropping the value. The current thread
/// will block until the shut down operation has completed.
///
/// * Drain any scheduled work queues.
/// * Drop any futures that have not yet completed.
/// * Drop the reactor.
///
/// Once the reactor has dropped, any outstanding I/O resources bound to
/// that reactor will no longer function. Calling any method on them will
/// result in an error.
///
/// [timer]: crate::time
/// [mod]: index.html
/// [`new`]: #method.new
#[derive(Clone, Copy, Default)]
pub struct AsyncStdRuntimeProvider;

impl Executor for AsyncStdRuntimeProvider {
    fn new() -> Self {
        Self {}
    }

    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        async_std::task::block_on(future)
    }
}

#[derive(Clone, Copy)]
pub struct AsyncStdRuntimeHandle;
impl Spawn for AsyncStdRuntimeHandle {
    fn spawn_bg<F>(&mut self, future: F)
    where
        F: Future<Output = Result<(), ProtoError>> + Send + 'static,
    {
        let _join = async_std::task::spawn(future);
    }
}

impl RuntimeProvider for AsyncStdRuntimeProvider {
    type Handle = AsyncStdRuntimeHandle;
    type Timer = AsyncStdTime;
    type Udp = AsyncStdUdpSocket;
    type Tcp = AsyncStdTcpStream;

    fn create_handle(&self) -> Self::Handle {
        AsyncStdRuntimeHandle {}
    }

    fn connect_tcp(
        &self,
        server_addr: SocketAddr,
        bind_addr: Option<SocketAddr>,
        wait_for: Option<Duration>,
    ) -> Pin<Box<dyn Send + Future<Output = std::io::Result<Self::Tcp>>>> {
        let wait_for = wait_for.unwrap_or_else(|| Duration::from_secs(5));
        Box::pin(async move {
            let stream = match bind_addr {
                Some(bind_addr) => {
                    let domain = match bind_addr {
                        SocketAddr::V4(_) => Domain::IPV4,
                        SocketAddr::V6(_) => Domain::IPV6,
                    };

                    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
                    socket.bind(&bind_addr.into())?;

                    socket.connect_timeout(&server_addr.into(), wait_for)?;
                    let std_stream = std::net::TcpStream::from(socket);
                    async_std::net::TcpStream::from(std_stream)
                }
                None => {
                    let future = async_std::net::TcpStream::connect(server_addr);
                    match timeout(wait_for, future).await {
                        Ok(Ok(socket)) => socket,
                        Ok(Err(e)) => return Err(e),
                        Err(_) => {
                            return Err(io::Error::new(
                                io::ErrorKind::TimedOut,
                                "connection to {server_addr:?} timed out after {wait_for:?}",
                            ))
                        }
                    }
                }
            };

            stream.set_nodelay(true)?;
            Ok(AsyncStdTcpStream(stream))
        })
    }

    fn bind_udp(
        &self,
        local_addr: SocketAddr,
        _server_addr: SocketAddr,
    ) -> Pin<Box<dyn Send + Future<Output = std::io::Result<Self::Udp>>>> {
        Box::pin(AsyncStdUdpSocket::bind(local_addr))
    }
}

#[derive(Clone, Default)]
pub struct AsyncStdConnectionProvider {
    runtime_provider: AsyncStdRuntimeProvider,
    connection_provider: GenericConnector<AsyncStdRuntimeProvider>,
}

impl Executor for AsyncStdConnectionProvider {
    fn new() -> Self {
        let p = AsyncStdRuntimeProvider::new();
        Self {
            runtime_provider: p,
            connection_provider: GenericConnector::new(p),
        }
    }

    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        self.runtime_provider.block_on(future)
    }
}

impl ConnectionProvider for AsyncStdConnectionProvider {
    type Conn = <GenericConnector<AsyncStdRuntimeProvider> as ConnectionProvider>::Conn;
    type FutureConn = <GenericConnector<AsyncStdRuntimeProvider> as ConnectionProvider>::FutureConn;
    type RuntimeProvider = AsyncStdRuntimeProvider;

    fn new_connection(
        &self,
        config: &NameServerConfig,
        options: &ResolverOpts,
    ) -> Result<Self::FutureConn, io::Error> {
        self.connection_provider.new_connection(config, options)
    }
}
