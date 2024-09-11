use anyhow::anyhow;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::UdpSocket;

use hopr_lib::{transfer_session, SendMsg};
use hopr_transport::{Session, SessionId, TransportSessionError};
use hopr_transport_session::types::unwrap_offchain_key;

#[derive(Default)]
struct BufferingMsgSender {
    buffer: std::sync::Mutex<Vec<u8>>,
}

#[async_trait::async_trait]
impl SendMsg for BufferingMsgSender {
    async fn send_message(
        &self,
        data: ApplicationData,
        _destination: PeerId,
        _options: RoutingOptions,
    ) -> Result<(), TransportSessionError> {
        let (_, data) = unwrap_offchain_key(data.plain_text)?;

        self.buffer.lock().unwrap().extend_from_slice(&data);

        tracing::debug!("wrote {} bytes", data.len());
        Ok(())
    }

    fn close(&self) {}
}

#[test_log::test(tokio::test)]
async fn udp_session_bridging() -> anyhow::Result<()> {
    let id = SessionId::new(1, OffchainKeypair::random().public().into());
    let (_tx, rx) = futures::channel::mpsc::unbounded();
    let mock = Arc::new(BufferingMsgSender::default());

    let mut session = Session::new(
        id,
        OffchainKeypair::random().public().into(),
        RoutingOptions::Hops(0_u32.try_into()?),
        HashSet::new(),
        mock.clone(),
        rx,
        None,
    );

    const BUF_LEN: usize = 16384;
    let mut listener = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_parallelism(0)
        .build(("127.0.0.1", 0))?;

    let addr = *listener.bound_address();

    tokio::task::spawn(async move {
        transfer_session(&mut session, &mut listener, BUF_LEN).await.unwrap();
    });

    let msg = [1u8; 9183];
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg[..8192], addr).await?;
    assert_eq!(8192, w);

    let w = sender.send_to(&msg[8192..], addr).await?;
    assert_eq!(991, w);

    for _ in 0..5 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        {
            let buf = mock.buffer.lock().unwrap();
            tracing::debug!("buf len is {}", buf.len());
            if buf.len() == msg.len() {
                assert_eq!(buf.as_slice(), msg);
                return Ok(());
            }
        }
    }

    Err(anyhow!("timeout"))
}
