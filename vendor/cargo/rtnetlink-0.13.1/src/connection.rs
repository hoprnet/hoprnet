// SPDX-License-Identifier: MIT

use std::io;

use futures::channel::mpsc::UnboundedReceiver;
use netlink_packet_core::NetlinkMessage;
use netlink_packet_route::RtnlMessage;
use netlink_proto::Connection;
use netlink_sys::{protocols::NETLINK_ROUTE, AsyncSocket, SocketAddr};

use crate::Handle;

#[cfg(feature = "tokio_socket")]
#[allow(clippy::type_complexity)]
pub fn new_connection() -> io::Result<(
    Connection<RtnlMessage>,
    Handle,
    UnboundedReceiver<(NetlinkMessage<RtnlMessage>, SocketAddr)>,
)> {
    new_connection_with_socket()
}

#[allow(clippy::type_complexity)]
pub fn new_connection_with_socket<S>() -> io::Result<(
    Connection<RtnlMessage, S>,
    Handle,
    UnboundedReceiver<(NetlinkMessage<RtnlMessage>, SocketAddr)>,
)>
where
    S: AsyncSocket,
{
    let (conn, handle, messages) =
        netlink_proto::new_connection_with_socket(NETLINK_ROUTE)?;
    Ok((conn, Handle::new(handle), messages))
}
