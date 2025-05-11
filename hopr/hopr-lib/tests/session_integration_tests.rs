use futures::StreamExt;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;

use hopr_lib::SendMsg;
use hopr_network_types::prelude::protocol::SessionMessage;
use hopr_transport::{Session, SessionId, TransportSessionError};
use hopr_transport_session::Capability;
use hopr_transport_session::{transfer_session, unwrap_chain_address};

struct BufferingMsgSender {
    buffer: futures::channel::mpsc::UnboundedSender<Box<[u8]>>,
}

#[async_trait::async_trait]
impl SendMsg for BufferingMsgSender {
    async fn send_message(&self, data: ApplicationData, _: DestinationRouting) -> Result<(), TransportSessionError> {
        let (_, data) = unwrap_chain_address(&data.plain_text)?;

        let len = data.len();
        self.buffer.unbounded_send(data).expect("buffer unbounded error");

        tracing::debug!("wrote {len} bytes");
        Ok(())
    }
}

#[tokio::test]
async fn udp_session_bridging() -> anyhow::Result<()> {
    let id = SessionId::new(1, (&ChainKeypair::random()).into());
    let (_tx, rx) = futures::channel::mpsc::unbounded();
    let (buffer_tx, mut buffer_rx) = futures::channel::mpsc::unbounded();

    let mut session = Session::new(
        id,
        (&ChainKeypair::random()).into(),
        DestinationRouting::forward_only(*id.peer(), RoutingOptions::Hops(0_u32.try_into()?)),
        HashSet::new(),
        Arc::new(BufferingMsgSender { buffer: buffer_tx }),
        rx,
        None,
    );

    const BUF_LEN: usize = 16384;
    let mut listener = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_receiver_parallelism(UdpStreamParallelism::Auto)
        .build(("127.0.0.1", 0))?;

    let addr = *listener.bound_address();

    tokio::task::spawn(async move {
        transfer_session(&mut session, &mut listener, BUF_LEN)
            .await
            .expect("transfer must not fail")
    });

    let msg = [1u8; 9183];
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg[..8192], addr).await?;
    assert_eq!(8192, w);

    let w = sender.send_to(&msg[8192..], addr).await?;
    assert_eq!(991, w);

    let mut recv_buf = Vec::with_capacity(msg.len());
    loop {
        if recv_buf.len() < msg.len() {
            let read = tokio::time::timeout(Duration::from_secs(5), buffer_rx.next())
                .await?
                .expect("must have data");
            recv_buf.extend_from_slice(&read);
        } else {
            break;
        }
    }

    assert_eq!(recv_buf.len(), msg.len());
    assert_eq!(recv_buf.as_slice(), msg);

    Ok(())
}

#[tokio::test]
async fn udp_session_bridging_with_segmentation() -> anyhow::Result<()> {
    let id = SessionId::new(1, (&ChainKeypair::random()).into());
    let (_tx, rx) = futures::channel::mpsc::unbounded();
    let (buffer_tx, mut buffer_rx) = futures::channel::mpsc::unbounded();

    let mut session = Session::new(
        id,
        (&ChainKeypair::random()).into(),
        DestinationRouting::forward_only(*id.peer(), RoutingOptions::Hops(0_u32.try_into()?)),
        HashSet::from_iter([Capability::Segmentation]),
        Arc::new(BufferingMsgSender { buffer: buffer_tx }),
        rx,
        None,
    );

    const BUF_LEN: usize = 16384;
    let mut listener = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_receiver_parallelism(UdpStreamParallelism::Auto)
        .build(("127.0.0.1", 0))?;

    let addr = *listener.bound_address();

    tokio::task::spawn(async move {
        transfer_session(&mut session, &mut listener, BUF_LEN)
            .await
            .expect("transfer must not fail")
    });

    let msg = [1u8; 9183];
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg[..8192], addr).await?;
    assert_eq!(8192, w);

    let w = sender.send_to(&msg[8192..], addr).await?;
    assert_eq!(991, w);

    let mut recv_buf = Vec::with_capacity(msg.len());
    loop {
        if recv_buf.len() < msg.len() {
            let read = tokio::time::timeout(Duration::from_secs(5), buffer_rx.next())
                .await?
                .expect("must have data");
            if let Some(msg) =
                SessionMessage::<{ hopr_transport_session::SESSION_USABLE_MTU_SIZE }>::try_from(read.as_ref())
                    .expect("must decode message")
                    .try_as_segment()
            {
                recv_buf.extend_from_slice(&msg.data);
            }
        } else {
            break;
        }
    }

    assert_eq!(recv_buf.len(), msg.len());
    assert_eq!(recv_buf.as_slice(), msg);

    Ok(())
}
