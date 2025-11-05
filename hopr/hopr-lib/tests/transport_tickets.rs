use std::{ops::Mul, str::FromStr, time::Duration};

use anyhow::Context;
use futures::AsyncWriteExt;
use hopr_lib::{
    ChannelId, HoprBalance, RoutingOptions, SessionClientConfig, SessionTarget,
    testing::fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes},
};
use hopr_primitive_types::bounded::BoundedVec;
use hopr_transport::session::{IpOrHost, SealedHost};
use rstest::rstest;
use serial_test::serial;
use tokio::time::sleep;

const FUNDING_AMOUNT: &str = "1 wxHOPR";

#[rstest]
#[tokio::test]
#[serial]
#[test_log::test]
async fn ticket_statistics_should_reset_when_cleaned(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [src, mid, dst] = exclusive_indexes::<3>();

    let fw_channel = cluster_fixture[src]
        .inner()
        .open_channel(
            &(cluster_fixture[mid].address()),
            FUNDING_AMOUNT.parse::<HoprBalance>()?,
        )
        .await
        .context("failed to open forward channel")?;

    let bw_channel = cluster_fixture[dst]
        .inner()
        .open_channel(
            &(cluster_fixture[mid].address()),
            FUNDING_AMOUNT.parse::<HoprBalance>()?,
        )
        .await
        .context("failed to open return channel")?;

    sleep(std::time::Duration::from_secs(3)).await;

    let ip = IpOrHost::from_str(":0")?;
    let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
        cluster_fixture[mid].address().into(),
    )));

    let mut session = cluster_fixture[src]
        .inner()
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: routing.clone(),
                return_path_options: routing,
                capabilities: Default::default(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .await
        .context("creating a session must succeed")?;

    const BUF_LEN: usize = 5000;
    let sent_data = hopr_crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let _ = cluster_fixture[mid]
        .inner()
        .tickets_in_channel(&fw_channel.channel_id)
        .await
        .context("failed to list tickets")?
        .into_iter()
        .count()
        .ne(&0);

    let _ = cluster_fixture[mid]
        .inner()
        .tickets_in_channel(&bw_channel.channel_id)
        .await
        .context("failed to list tickets")?
        .into_iter()
        .count()
        .ne(&0);

    let channels_with_pending_tickets = cluster_fixture[mid]
        .inner()
        .all_tickets()
        .await
        .context("failed to get all tickets")?
        .into_iter()
        .map(|t| t.channel_id)
        .collect::<Vec<ChannelId>>();

    assert!(channels_with_pending_tickets.contains(&fw_channel.channel_id));
    assert!(channels_with_pending_tickets.contains(&bw_channel.channel_id));

    let stats_before = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    assert!(stats_before.winning_count > 0); // As winning prob is set to 1

    cluster_fixture[mid]
        .inner()
        .reset_ticket_statistics()
        .await
        .context("failed to reset ticket statistics")?;

    let stats_after = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    assert_ne!(stats_before, stats_after);
    assert_eq!(stats_after.winning_count, 0);

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_reject_relaying_a_message_when_the_channel_is_out_of_funding(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [src, mid, dst] = exclusive_indexes::<3>();

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    let _fw_channel = cluster_fixture[src]
        .inner()
        .open_channel(&(cluster_fixture[mid].address()), ticket_price)
        .await
        .context("failed to open forward channel")?;

    let _bw_channel = cluster_fixture[dst]
        .inner()
        .open_channel(&(cluster_fixture[mid].address()), ticket_price)
        .await
        .context("failed to open return channel")?;

    sleep(std::time::Duration::from_secs(3)).await;

    let ip = IpOrHost::from_str(":0")?;
    let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
        cluster_fixture[mid].address().into(),
    )));

    let mut session = cluster_fixture[src]
        .inner()
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: routing.clone(),
                return_path_options: routing,
                capabilities: Default::default(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .await
        .context("creating a session must succeed")?;

    const BUF_LEN: usize = 500;
    let sent_data = hopr_crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    let stats = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;
    assert_eq!(stats.rejected_value, HoprBalance::zero());

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    sleep(std::time::Duration::from_secs(2)).await;

    let stats = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;
    assert!(stats.rejected_value > HoprBalance::zero());

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_redeem_ticket_on_request(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, mid, dst] = exclusive_indexes::<3>();
    let message_count = 3;

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count);

    let _fw_channel = cluster_fixture[src]
        .inner()
        .open_channel(&(cluster_fixture[mid].address()), funding_amount)
        .await
        .context("failed to open forward channel")?;

    let _bw_channel = cluster_fixture[dst]
        .inner()
        .open_channel(&(cluster_fixture[mid].address()), funding_amount)
        .await
        .context("failed to open return channel")?;

    sleep(std::time::Duration::from_secs(3)).await;

    let ip = IpOrHost::from_str(":0")?;
    let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
        cluster_fixture[mid].address().into(),
    )));

    let mut session = cluster_fixture[src]
        .inner()
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: routing.clone(),
                return_path_options: routing,
                capabilities: Default::default(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .await
        .context("creating a session must succeed")?;

    const BUF_LEN: usize = 400;
    let sent_data = hopr_crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    for _ in 1..=message_count {
        tokio::time::timeout(Duration::from_millis(1000), session.write_all(&sent_data))
            .await
            .context("write failed")??;
    }

    sleep(std::time::Duration::from_secs(15)).await;

    let stats_before = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;
    assert_eq!(
        stats_before.unredeemed_value,
        funding_amount.mul(2) + ticket_price.mul(2)
    ); // both ways + 2 packets for session initiation

    cluster_fixture[mid]
        .inner()
        .redeem_all_tickets(0, false)
        .await
        .context("failed to redeem tickets")?;

    sleep(std::time::Duration::from_secs(15)).await;

    let stats_after = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    assert_eq!(stats_after.unredeemed_value, HoprBalance::zero());

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_neglect_ticket_on_closing(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, mid, dst] = exclusive_indexes::<3>();
    let message_count = 3;

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count);

    let fw_channel = cluster_fixture[src]
        .inner()
        .open_channel(&(cluster_fixture[mid].address()), funding_amount)
        .await
        .context("failed to open forward channel")?;

    let bw_channel = cluster_fixture[dst]
        .inner()
        .open_channel(&(cluster_fixture[mid].address()), funding_amount)
        .await
        .context("failed to open return channel")?;

    sleep(std::time::Duration::from_secs(3)).await;

    let ip = IpOrHost::from_str(":0")?;
    let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
        cluster_fixture[mid].address().into(),
    )));

    let mut session = cluster_fixture[src]
        .inner()
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: routing.clone(),
                return_path_options: routing,
                capabilities: Default::default(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .await
        .context("creating a session must succeed")?;

    const BUF_LEN: usize = 400;
    let sent_data = hopr_crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    for _ in 1..=message_count {
        tokio::time::timeout(Duration::from_millis(1000), session.write_all(&sent_data))
            .await
            .context("write failed")??;
    }

    sleep(std::time::Duration::from_secs(15)).await;

    let stats_before = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;
    assert!(stats_before.unredeemed_value > HoprBalance::zero());
    assert_eq!(stats_before.neglected_value, HoprBalance::zero());

    cluster_fixture[src]
        .inner()
        .close_channel_by_id(&fw_channel.channel_id)
        .await
        .context("failed to close forward channel")?;

    cluster_fixture[dst]
        .inner()
        .close_channel_by_id(&bw_channel.channel_id)
        .await
        .context("failed to close forward channel")?;

    let stats_after = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    assert_eq!(stats_after.unredeemed_value, HoprBalance::zero());
    assert!(stats_after.neglected_value > HoprBalance::zero());

    Ok(())
}
