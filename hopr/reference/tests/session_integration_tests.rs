use futures::StreamExt;
use hopr_lib::{
    ApplicationDataIn, ApplicationDataOut,
    exports::transport::session::{
        Capabilities, Capability, HoprSession, HoprSessionConfig, SessionId, transfer_session,
    },
};
use rstest::*;
use tokio::{io::AsyncReadExt, net::UdpSocket, sync::oneshot};

#[rstest]
#[case(Capabilities::empty())]
#[case(Capabilities::from(Capability::Segmentation))]
#[tokio::test]
/// Creates paired Hopr sessions bridged to a UDP listener to prove that messages
/// sent over UDP end up in the remote session buffer regardless of capability set.
async fn udp_session_bridging(#[case] cap: Capabilities) -> anyhow::Result<()> {
    use hopr_lib::{
        Address, ChainKeypair, ConnectedUdpStream, DestinationRouting, HoprPseudonym, Keypair, RoutingOptions,
        UdpStreamParallelism, crypto_traits::Randomizable,
    };

    let dst: Address = (&ChainKeypair::random()).into();
    let id = SessionId::new(1u64, HoprPseudonym::random());
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    let mut alice_session = HoprSession::new(
        id,
        DestinationRouting::forward_only(dst, RoutingOptions::Hops(0_u32.try_into()?)),
        HoprSessionConfig {
            capabilities: cap,
            ..Default::default()
        },
        (
            alice_tx,
            alice_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        None,
    )?;

    let mut bob_session = HoprSession::new(
        id,
        DestinationRouting::Return(id.pseudonym().into()),
        HoprSessionConfig {
            capabilities: cap,
            ..Default::default()
        },
        (
            bob_tx,
            bob_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        None,
    )?;

    const BUF_LEN: usize = 16384;

    let mut listener = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_receiver_parallelism(UdpStreamParallelism::Auto)
        .build(("127.0.0.1", 0))?;

    let addr = *listener.bound_address();

    let (ready_tx, ready_rx) = oneshot::channel();
    let transfer_handle = tokio::task::spawn(async move {
        ready_tx.send(()).ok();
        transfer_session(&mut alice_session, &mut listener, BUF_LEN, None).await
    });
    ready_rx.await.ok();

    let mut msg = [0u8; 9183];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut msg);
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg[..8192], addr).await?;
    assert_eq!(8192, w);

    let w = sender.send_to(&msg[8192..], addr).await?;
    assert_eq!(991, w);

    let mut recv_msg = [0u8; 9183];
    bob_session.read_exact(&mut recv_msg).await?;

    assert_eq!(recv_msg, msg);

    transfer_handle.abort();
    match transfer_handle.await {
        Ok(Err(e)) => panic!("transfer failed: {e}"),
        _ => {} // Task was aborted (expected)
    }

    Ok(())
}
