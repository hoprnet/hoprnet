mod common;

use std::{ops::Mul, time::Duration};

use common::{
    fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes},
    hopr_tester::HoprTester,
};
use rstest::rstest;

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[tokio::test]
async fn test_get_balance(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
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

    assert_ne!(safe_native, XDaiBalance::zero());
    assert_ne!(native, XDaiBalance::zero());
    assert_ne!(safe_hopr, HoprBalance::zero());
    assert_eq!(hopr, HoprBalance::zero());
    assert_ne!(safe_allowance, HoprBalance::zero());

    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_ping_peer_inside_cluster(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = exclusive_indexes::<2>();

    cluster_fixture[src]
        .inner()
        .ping(&cluster_fixture[dst].peer_id())
        .await
        .expect("failed to ping peer");

    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_ping_self_should_fail(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [random_int] = exclusive_indexes::<1>();
    let res = cluster_fixture[random_int]
        .inner()
        .ping(&cluster_fixture[random_int].peer_id())
        .await;

    assert!(res.is_err());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_open_close_channel(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::{ChannelStatus, HoprBalance};
    use tokio::time::sleep;

    let [src, dst] = exclusive_indexes::<2>();

    assert!(
        cluster_fixture[src]
            .outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .expect("failed to get channels from src node")
            .is_empty()
    );

    let channel = cluster_fixture[src]
        .inner()
        .open_channel(
            &(cluster_fixture[dst].address()),
            FUNDING_AMOUNT.parse::<HoprBalance>()?,
        )
        .await
        .expect("failed to open channel");

    assert_eq!(
        cluster_fixture[src]
            .outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .expect("failed to get channels from src node")
            .len(),
        1
    );

    cluster_fixture[src]
        .inner()
        .close_channel_by_id(&channel.channel_id)
        .await
        .expect("failed to put channel in PendingToClose state");

    sleep(Duration::from_secs(2)).await;

    match cluster_fixture[src]
        .channel_from_hash(&channel.channel_id)
        .await
        .expect("failed to get channel from id")
        .status
    {
        ChannelStatus::PendingToClose(_) => (),
        _ => panic!("channel should be in PendingToClose state"),
    }

    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_channel_funding_should_be_visible_in_channel_stake(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    use hopr_lib::HoprBalance;

    let [src, dst] = exclusive_indexes::<2>();
    let funding_amount = FUNDING_AMOUNT.parse::<HoprBalance>()?;

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
#[tokio::test]
async fn test_reset_ticket_statistics(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::{DestinationRouting, HoprBalance, RoutingOptions, Tag};
    use hopr_primitive_types::bounded::BoundedVec;

    let [src, mid, dst] = exclusive_indexes::<3>();

    let _ = cluster_fixture[src]
        .inner()
        .open_channel(
            &(cluster_fixture[mid].address()),
            FUNDING_AMOUNT.parse::<HoprBalance>()?,
        )
        .await
        .expect("failed to open channel");

    // TODO: replace with session
    // cluster_fixture[src]
    //     .inner()
    //     .send_message(
    //         b"Hello, HOPR!".to_vec().into(),
    //         DestinationRouting::forward_only(
    //             cluster_fixture[dst].address(),
    //             RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
    //                 cluster_fixture[mid].address(),
    //             ))),
    //         ),
    //         Tag::Application(1024),
    //     )
    //     .await
    //     .expect("failed to send 1-hop message");

    let stats_before = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .expect("failed to get ticket statistics");

    assert_eq!(stats_before.winning_count, 1); // As winning prob is set to 1

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
    assert_eq!(stats_after.winning_count, 0);

    Ok(())
}

// #[rstest]
// #[tokio::test]
// #[cfg(feature = "session-client")]
// async fn test_create_0_hop_session(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
//     use std::str::FromStr;

//     use futures::AsyncReadExt;
//     use hopr_lib::{
//         RoutingOptions, SessionClientConfig, SessionTarget,
//         prelude::{ConnectedUdpStream, UdpStreamParallelism},
//     };
//     use hopr_transport_session::{Capabilities, Capability, IpOrHost, SealedHost};
//     use tokio::net::UdpSocket;
//     let [src, dst] = exclusive_indexess::<2>();

//     let ip = IpOrHost::from_str(":0").expect("invalid IpOrHost");

//     let mut session = cluster_fixture[src]
//         .inner()
//         .connect_to(
//             cluster_fixture[dst].address(),
//             SessionTarget::UdpStream(SealedHost::Plain(ip)),
//             SessionClientConfig {
//                 forward_path_options: RoutingOptions::Hops(0_u32.try_into()?),
//                 return_path_options: RoutingOptions::Hops(0_u32.try_into()?),
//                 capabilities: Capabilities::from(Capability::Segmentation),
//                 pseudonym: None,
//                 surb_management: None,
//                 always_max_out_surbs: true,
//             },
//         )
//         .await
//         .expect("creating a session must succeed")
//

//     const BUF_LEN: usize = 16384;

//     let listener = ConnectedUdpStream::builder()
//         .with_buffer_size(BUF_LEN)
//         .with_queue_size(512)
//         .with_receiver_parallelism(UdpStreamParallelism::Auto)
//         .build(("127.0.0.1", 0))?;

//     let addr = *listener.bound_address();

//     let msg: [u8; 9183] = hopr_crypto_random::random_bytes();
//     let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

//     let w = sender.send_to(&msg[..8192], addr).await?;
//     assert_eq!(8192, w);

//     let w = sender.send_to(&msg[8192..], addr).await?;
//     assert_eq!(991, w);

//     let mut recv_msg = [0u8; 9183];
//     session.read_exact(&mut recv_msg).await?;

//     assert_eq!(recv_msg, msg);

//     Ok(())
// }

// #[rstest]
// #[tokio::test]
// #[cfg(feature = "session-client")]
// async fn test_create_1_hop_session(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
//     let [src, mid, dst] = exclusive_indexes::<3>();

//     let ip = IpOrHost::from_str(":0").expect("invalid IpOrHost");

//     cluster_fixture[src]
//         .inner()
//         .open_channel(
//             &(cluster_fixture[mid].address()),
//             FUNDING_AMOUNT.parse::<HoprBalance>()?,
//         )
//         .await
//         .expect("failed to open channel");

//     let mut session = cluster_fixture[src]
//         .inner()
//         .connect_to(
//             cluster_fixture[dst].address(),
//             SessionTarget::UdpStream(SealedHost::Plain(ip)),
//             SessionClientConfig {
//                 forward_path_options: hopr_lib::RoutingOptions::Hops(1_u32.try_into()?),
//                 return_path_options: hopr_lib::RoutingOptions::Hops(1_u32.try_into()?),
//                 capabilities: Capabilities::from(Capability::Segmentation),
//                 pseudonym: None,
//                 surb_management: None,
//                 always_max_out_surbs: true,
//             },
//         )
//         .await
//         .expect("creating a session must succeed");

//     const BUF_LEN: usize = 16384;

//     let listener = ConnectedUdpStream::builder()
//         .with_buffer_size(BUF_LEN)
//         .with_queue_size(512)
//         .with_receiver_parallelism(UdpStreamParallelism::Auto)
//         .build(("127.0.0.1", 0))?;

//     let addr = *listener.bound_address();

//     let msg: [u8; 9183] = hopr_crypto_random::random_bytes();
//     let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

//     let w = sender.send_to(&msg[..8192], addr).await?;
//     assert_eq!(8192, w);

//     let w = sender.send_to(&msg[8192..], addr).await?;
//     assert_eq!(991, w);

//     let mut recv_msg = [0u8; 9183];
//     session.read_exact(&mut recv_msg).await?;
//     assert_eq!(recv_msg, msg);

//     Ok(())
// }
