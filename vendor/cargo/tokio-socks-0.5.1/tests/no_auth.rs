mod common;

use crate::common::{runtime, test_bind};
use common::{connect_unix, test_connect, ECHO_SERVER_ADDR, PROXY_ADDR};
use tokio_socks::{
    tcp::{Socks5Listener, Socks5Stream},
    Result,
};

#[test]
fn connect_no_auth() -> Result<()> {
    let runtime = runtime().lock().unwrap();
    let conn = runtime.block_on(Socks5Stream::connect(PROXY_ADDR, ECHO_SERVER_ADDR))?;
    runtime.block_on(test_connect(conn))
}

#[test]
fn bind_no_auth() -> Result<()> {
    let bind = {
        let runtime = runtime().lock().unwrap();
        runtime.block_on(Socks5Listener::bind(PROXY_ADDR, ECHO_SERVER_ADDR))
    }?;
    test_bind(bind)
}

#[test]
fn connect_with_socket_no_auth() -> Result<()> {
    let runtime = runtime().lock().unwrap();
    let socket = runtime.block_on(connect_unix())?;
    let conn = runtime.block_on(Socks5Stream::connect_with_socket(socket, ECHO_SERVER_ADDR))?;
    runtime.block_on(test_connect(conn))
}

#[test]
fn bind_with_socket_no_auth() -> Result<()> {
    let bind = {
        let runtime = runtime().lock().unwrap();
        let socket = runtime.block_on(connect_unix())?;
        runtime.block_on(Socks5Listener::bind_with_socket(socket, ECHO_SERVER_ADDR))
    }?;
    test_bind(bind)
}
