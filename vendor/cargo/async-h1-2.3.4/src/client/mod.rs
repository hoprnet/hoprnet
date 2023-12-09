//! Process HTTP connections on the client.

use futures_lite::io::{self, AsyncRead as Read, AsyncWrite as Write};
use http_types::{Request, Response};

mod decode;
mod encode;

pub use decode::decode;
pub use encode::Encoder;

/// Opens an HTTP/1.1 connection to a remote host.
pub async fn connect<RW>(mut stream: RW, req: Request) -> http_types::Result<Response>
where
    RW: Read + Write + Send + Sync + Unpin + 'static,
{
    let mut req = Encoder::new(req);
    log::trace!("> {:?}", &req);

    io::copy(&mut req, &mut stream).await?;

    let res = decode(stream).await?;
    log::trace!("< {:?}", &res);

    Ok(res)
}
