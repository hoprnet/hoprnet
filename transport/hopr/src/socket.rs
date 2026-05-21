use futures::{Sink, Stream};
use hopr_api::types::internal::routing::DestinationRouting;
use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};

/// Represents the socket behavior of the hopr-lib spawned `Hopr` object.
///
/// HOPR socket aims to mimic a simple socket like write and read behavior for unstructured
/// communication without the advanced session properties.
///
/// Typical use cases might be low level UDP based protocols that can wrap the HOPR socket
/// without needing the advanced session functionality.
pub struct HoprSocket<T, U> {
    rx: T,
    tx: U,
}

impl<T, U> From<(T, U)> for HoprSocket<T, U>
where
    T: Stream<Item = ApplicationDataIn> + Send + 'static,
    U: Sink<(DestinationRouting, ApplicationDataOut)> + Send + Clone + 'static,
{
    fn from(value: (T, U)) -> Self {
        Self {
            rx: value.0,
            tx: value.1,
        }
    }
}

impl<T, U> HoprSocket<T, U>
where
    T: Stream<Item = ApplicationDataIn> + Send + 'static,
    U: Sink<(DestinationRouting, ApplicationDataOut)> + Send + Clone + 'static,
{
    pub fn reader(self) -> T {
        self.rx
    }

    pub fn writer(&self) -> U {
        self.tx.clone()
    }
}

#[cfg(test)]
mod tests {
    use futures::channel::mpsc;
    use hopr_api::types::internal::routing::DestinationRouting;
    use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};

    use super::*;

    #[tokio::test]
    async fn socket_from_tuple_and_reader_writer_work() -> anyhow::Result<()> {
        let (tx, _rx) = mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let (_in_tx, in_rx) = mpsc::unbounded::<ApplicationDataIn>();

        let socket = HoprSocket::from((in_rx, tx));
        // writer() should return a clone of the sender
        let _writer = socket.writer();
        // reader() consumes the socket and returns the stream
        let _reader = socket.reader();

        Ok(())
    }
}
