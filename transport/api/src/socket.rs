use futures::{Sink, Stream};
use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};
use hopr_transport_session::DestinationRouting;

/// Represents the socket behavior of the hopr-lib spawned [`Hopr`] object.
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
    U: Sink<(ApplicationDataOut, DestinationRouting)> + Send + Clone + 'static,
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
    U: Sink<(ApplicationDataOut, DestinationRouting)> + Send + Clone + 'static,
{
    pub fn reader(self) -> T {
        self.rx
    }

    pub fn writer(&self) -> U {
        self.tx.clone()
    }
}
