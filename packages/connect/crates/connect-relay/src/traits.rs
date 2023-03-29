use futures::{Sink, Stream};

/// Duplex stream trait used in relay protocol
pub trait DuplexStream:
    Stream<Item = Result<Box<[u8]>, String>> + Sink<Box<[u8]>, Error = String> + Unpin
{
}
