use futures::StreamExt;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_lib::ApplicationData;
use hopr_network_types::prelude::*;
use hopr_primitive_types::prelude::Address;
use hopr_transport::{Session, SessionId};
use hopr_transport_session::{Capabilities, Capability, transfer_session};
use parameterized::parameterized;
use tokio::{io::AsyncReadExt, net::UdpSocket};

#[parameterized(cap = { Capabilities::empty(), Capabilities::from(Capability::Segmentation) })]
#[parameterized_macro(tokio::test)]
async fn udp_session_bridging(cap: Capabilities) -> anyhow::Result<()> {
    let dst: Address = (&ChainKeypair::random()).into();
    let id = SessionId::new(1u64, HoprPseudonym::random());
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationData)>();
    let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationData)>();

    let mut alice_session = Session::new(
        id,
        DestinationRouting::forward_only(dst, RoutingOptions::Hops(0_u32.try_into()?)),
        cap,
        (alice_tx, alice_rx.map(|(_, d)| d.plain_text)),
        None,
    )?;

    let mut bob_session = Session::new(
        id,
        DestinationRouting::Return(id.pseudonym().into()),
        cap,
        (bob_tx, bob_rx.map(|(_, d)| d.plain_text)),
        None,
    )?;

    const BUF_LEN: usize = 16384;

    let mut listener = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_receiver_parallelism(UdpStreamParallelism::Auto)
        .build(("127.0.0.1", 0))?;

    let addr = *listener.bound_address();

    tokio::task::spawn(async move {
        transfer_session(&mut alice_session, &mut listener, BUF_LEN)
            .await
            .expect("transfer must not fail")
    });

    let msg: [u8; 9183] = hopr_crypto_random::random_bytes();
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg[..8192], addr).await?;
    assert_eq!(8192, w);

    let w = sender.send_to(&msg[8192..], addr).await?;
    assert_eq!(991, w);

    let mut recv_msg = [0u8; 9183];
    bob_session.read_exact(&mut recv_msg).await?;

    assert_eq!(recv_msg, msg);
    Ok(())
}
