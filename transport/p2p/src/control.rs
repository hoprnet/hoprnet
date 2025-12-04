use futures::{AsyncRead, AsyncWrite, Stream};
use libp2p::{PeerId, StreamProtocol};

// Control object for the streams over the HOPR protocols
#[derive(Clone)]
pub struct HoprStreamProtocolControl {
    control: libp2p_stream::Control,
    protocol: StreamProtocol,
}

impl HoprStreamProtocolControl {
    pub fn new(control: libp2p_stream::Control, protocol: &'static str) -> Self {
        Self {
            control,
            protocol: StreamProtocol::new(protocol),
        }
    }
}

impl std::fmt::Debug for HoprStreamProtocolControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprStreamProtocolControl")
            .field("protocol", &self.protocol)
            .finish()
    }
}

#[async_trait::async_trait]
impl hopr_transport_protocol::stream::BidirectionalStreamControl for HoprStreamProtocolControl {
    fn accept(
        mut self,
    ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error> {
        self.control.accept(self.protocol)
    }

    async fn open(mut self, peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error> {
        self.control.open_stream(peer, self.protocol).await
    }
}
