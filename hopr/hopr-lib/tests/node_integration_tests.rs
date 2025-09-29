mod common;

use common::fixtures::{SWARM_N, cluster_fixture, random_int, random_int_pair, random_int_triple};
use common::hopr_tester::HoprTester;
use hopr_lib::{DestinationRouting, HoprBalance, RoutingOptions, Tag};
use hopr_primitive_types::bounded::BoundedVec;
use rstest::rstest;
use std::ops::Mul;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_cluster_connectivity(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
    use tokio::time::{Instant, sleep};

    let start = Instant::now();
    let timeout_duration = Duration::from_secs(30);

    loop {
        let results = futures::future::join_all(
            cluster_fixture
                .iter()
                .map(|node| node.inner().network_connected_peers()),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to get connected peers");

        if results.iter().all(|peers| peers.len() == SWARM_N - 1) {
            break;
        }

        if start.elapsed() >= timeout_duration {
            panic!("Timeout: not all nodes connected within 60s");
        }

        sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_balance(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
    use hopr_lib::{HoprBalance, WxHOPR, XDai, XDaiBalance};

    let node: &HoprTester = &cluster_fixture[0];
    let safe_native = node
        .get_safe_balance::<XDai>()
        .await
        .expect("should get safe xdai balance");
    let native = node.get_balance::<XDai>().await.expect("should get node xdai balance");
    let safe_hopr = node
        .get_safe_balance::<WxHOPR>()
        .await
        .expect("should get safe hopr balance");
    let hopr = node
        .get_balance::<WxHOPR>()
        .await
        .expect("should get node hopr balance");
    let safe_allowance = node.safe_allowance().await.expect("should get safe hopr allowance");

    assert!(safe_native != XDaiBalance::zero());
    assert!(native != XDaiBalance::zero());
    assert!(safe_hopr != HoprBalance::zero());
    assert!(hopr == HoprBalance::zero());
    assert!(safe_allowance != HoprBalance::zero());

    info!("Safe xDai balance: {}", safe_native);
    info!("Node xDai balance: {}", native);
    info!("Safe HOPR balance: {}", safe_hopr);
    info!("Node HOPR balance: {}", hopr);
    info!("Safe HOPR allowance: {}", safe_allowance);

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_ping_peer_inside_cluster(
    #[future(awt)] cluster_fixture: &Vec<HoprTester>,
    random_int_pair: (usize, usize),
) -> anyhow::Result<()> {
    let (src, dst) = random_int_pair;

    let (duration, _) = cluster_fixture[src]
        .inner()
        .ping(&cluster_fixture[dst].peer_id())
        .await
        .expect("failed to ping peer");

    assert!(duration > Duration::ZERO);

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_ping_self_should_fail(
    #[future(awt)] cluster_fixture: &Vec<HoprTester>,
    random_int: usize,
) -> anyhow::Result<()> {
    let res = cluster_fixture[random_int]
        .inner()
        .ping(&cluster_fixture[random_int].peer_id())
        .await;

    assert!(res.is_err());
    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_open_close_channel(
    #[future(awt)] cluster_fixture: &Vec<HoprTester>,
    random_int_pair: (usize, usize),
) -> anyhow::Result<()> {
    use hopr_lib::ChannelStatus;
    use tokio::time::sleep;

    let (src, dst) = random_int_pair;

    assert!(
        cluster_fixture[src]
            .outgoing_channels_by_status(Some(ChannelStatus::Open))
            .await
            .expect("failed to get channels from src node")
            .is_empty()
    );

    let channel = cluster_fixture[src]
        .inner()
        .open_channel(&(cluster_fixture[dst].address()), FUNDING_AMOUNT.into())
        .await
        .expect("failed to open channel");

    assert_eq!(
        cluster_fixture[src]
            .outgoing_channels_by_status(Some(ChannelStatus::Open))
            .await
            .expect("failed to get channels from src node")
            .len(),
        1
    );

    cluster_fixture[src]
        .inner()
        .close_channel_by_id(channel.channel_id, false)
        .await
        .expect("failed to put channel in PendingToClose state");

    sleep(Duration::from_secs(2)).await;

    assert!(
        cluster_fixture[src]
            .outgoing_channels_by_status(Some(ChannelStatus::Open))
            .await
            .expect("failed to get channels from src node")
            .is_empty()
    );

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_channel_funding_should_be_visible_in_channel_stake(
    #[future(awt)] cluster_fixture: &Vec<HoprTester>,
    random_int_pair: (usize, usize),
) -> anyhow::Result<()> {
    let (src, dst) = random_int_pair;
    let funding_amount: HoprBalance = FUNDING_AMOUNT.parse()?;

    let channel = cluster_fixture[src]
        .inner()
        .open_channel(&(cluster_fixture[dst].address()), funding_amount)
        .await
        .expect("failed to open channel");

    let _ = cluster_fixture[src]
        .inner()
        .fund_channel(&channel.channel_id, funding_amount)
        .await;

    let updated_channel = cluster_fixture[src]
        .channel_from_hash(&channel.channel_id)
        .await
        .expect("failed to retrieve channel by id");

    assert_eq!(updated_channel.balance, funding_amount.mul(2));

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_send_0_hop_without_open_channels(
    #[future(awt)] cluster_fixture: &Vec<HoprTester>,
    random_int_pair: (usize, usize),
) -> anyhow::Result<()> {
    use hopr_lib::{DestinationRouting, RoutingOptions, Tag};

    let (src, dst) = random_int_pair;

    cluster_fixture[src]
        .inner()
        .send_message(
            b"Hello, HOPR!".to_vec().into(),
            DestinationRouting::forward_only(cluster_fixture[dst].address(), RoutingOptions::Hops(0.try_into()?)),
            Tag::Application(1024),
        )
        .await
        .expect("failed to send 0-hop message");

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_reset_ticket_statistics(
    #[future(awt)] cluster_fixture: &Vec<HoprTester>,
    random_int_triple: (usize, usize, usize),
) -> anyhow::Result<()> {
    let funding_amount: HoprBalance = FUNDING_AMOUNT.parse()?;

    let (src, mid, dst) = random_int_triple;

    let _ = cluster_fixture[src]
        .inner()
        .open_channel(&(cluster_fixture[mid].address()), funding_amount)
        .await
        .expect("failed to open channel");

    cluster_fixture[src]
        .inner()
        .send_message(
            b"Hello, HOPR!".to_vec().into(),
            DestinationRouting::forward_only(
                cluster_fixture[dst].address(),
                RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
                    cluster_fixture[mid].address(),
                ))),
            ),
            Tag::Application(1024),
        )
        .await
        .expect("failed to send 1-hop message");

    sleep(Duration::from_secs(1)).await;

    let stats_before = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .expect("failed to get ticket statistics");

    info!("Ticket stats before reset: {:?}", stats_before);

    assert_ne!(stats_before.winning_count, 0);

    cluster_fixture[mid]
        .inner()
        .reset_ticket_statistics()
        .await
        .expect("failed to reset ticket statistics");

    let stats_after = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .expect("failed to get ticket statistics");

    assert_ne!(stats_before, stats_after);

    Ok(())
}

#[rstest]
#[cfg(all(feature = "runtime-tokio", feature = "session-client"))]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_create_0_hop_session(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
    use hopr_lib::{RoutingOptions, SessionClientConfig, SessionTarget};
    use hopr_network_types::udp::{ConnectedUdpStream, UdpStreamParallelism};
    use hopr_transport_session::{Capabilities, Capability, IpOrHost, SealedHost};
    use tokio::{io::AsyncReadExt, net::UdpSocket};

    let ip = IpOrHost::from_str(":0").expect("invalid IpOrHost");

    let mut session = cluster_fixture[0]
        .inner
        .connect_to(
            cluster_fixture[1].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: RoutingOptions::Hops(0_u32.try_into()?),
                return_path_options: RoutingOptions::Hops(0_u32.try_into()?),
                capabilities: Capabilities::from(Capability::Segmentation),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: true,
            },
        )
        .await
        .expect("creating a session must succeed");

    const BUF_LEN: usize = 16384;

    let listener = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_receiver_parallelism(UdpStreamParallelism::Auto)
        .build(("127.0.0.1", 0))?;

    let addr = *listener.bound_address();

    let msg: [u8; 9183] = hopr_crypto_random::random_bytes();
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg[..8192], addr).await?;
    assert_eq!(8192, w);

    let w = sender.send_to(&msg[8192..], addr).await?;
    assert_eq!(991, w);

    let mut recv_msg = [0u8; 9183];
    session.read_exact(&mut recv_msg).await?;

    assert_eq!(recv_msg, msg);

    Ok(())
}

#[rstest]
#[cfg(all(feature = "runtime-tokio", feature = "session-client"))]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_create_1_hop_session(
    #[future(awt)] cluster_fixture: &Vec<HoprTester>,
    random_int_triple: (usize, usize, usize),
) -> anyhow::Result<()> {
    use hopr_lib::{SessionClientConfig, SessionTarget};
    use hopr_network_types::udp::{ConnectedUdpStream, UdpStreamParallelism};
    use hopr_transport_session::{Capabilities, Capability, IpOrHost, SealedHost};
    use tokio::{io::AsyncReadExt, net::UdpSocket};

    let (src, mid, dst) = random_int_triple;

    let ip = IpOrHost::from_str(":0").expect("invalid IpOrHost");

    cluster_fixture[src]
        .inner
        .open_channel(&(cluster_fixture[mid].address()), FUNDING_AMOUNT.into())
        .await
        .expect("failed to open channel");

    let mut session = cluster_fixture[src]
        .inner
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: hopr_lib::RoutingOptions::Hops(1_u32.try_into()?),
                return_path_options: hopr_lib::RoutingOptions::Hops(1_u32.try_into()?),
                capabilities: Capabilities::from(Capability::Segmentation),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: true,
            },
        )
        .await
        .expect("creating a session must succeed");

    const BUF_LEN: usize = 16384;

    let listener = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_receiver_parallelism(UdpStreamParallelism::Auto)
        .build(("127.0.0.1", 0))?;

    let addr = *listener.bound_address();

    let msg: [u8; 9183] = hopr_crypto_random::random_bytes();
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg[..8192], addr).await?;
    assert_eq!(8192, w);

    let w = sender.send_to(&msg[8192..], addr).await?;
    assert_eq!(991, w);

    let mut recv_msg = [0u8; 9183];
    session.read_exact(&mut recv_msg).await?;
    assert_eq!(recv_msg, msg);

    Ok(())
}
