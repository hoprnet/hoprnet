use std::{ops::Mul, time::Duration};

use anyhow::Context;
use futures::AsyncWriteExt;
use hopr_lib::{
    ChannelId, HoprBalance,
    testing::fixtures::{
        ClusterGuard, cluster_fixture, exclusive_indexes_auto_redeeming, exclusive_indexes_not_auto_redeeming,
        exclusive_indexes_with_auto_redeem_intermediaries,
    },
};
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
    let [src, mid, dst] = exclusive_indexes_not_auto_redeeming::<3>();

    let (mut session, fw_channels, bw_channels) = cluster_fixture
        .create_session_between(src, mid, dst, FUNDING_AMOUNT.parse::<HoprBalance>()?)
        .await?;

    const BUF_LEN: usize = 5000;
    let sent_data = hopr_crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    assert!(
        fw_channels
            .try_to_get_all_ticket_counts()
            .await?
            .first()
            .context("no tickets found for the first forward channel")?
            .ne(&0)
    );
    assert!(
        bw_channels
            .try_to_get_all_ticket_counts()
            .await?
            .first()
            .context("no tickets found for the first backward channel")?
            .ne(&0)
    );

    let channels_with_pending_tickets = cluster_fixture[mid]
        .inner()
        .all_tickets()
        .await
        .context("failed to get all tickets")?
        .into_iter()
        .map(|t| t.channel_id)
        .collect::<Vec<ChannelId>>();

    assert!(channels_with_pending_tickets.contains(&fw_channels.channel_id(0)));
    assert!(channels_with_pending_tickets.contains(&bw_channels.channel_id(0)));

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
    let [src, mid, dst] = exclusive_indexes_not_auto_redeeming::<3>();

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    let (mut session, _fw_channel, _bw_channel) = cluster_fixture
        .create_session_between(src, mid, dst, ticket_price)
        .await?;

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
    let [src, mid, dst] = exclusive_indexes_not_auto_redeeming::<3>();
    let message_count = 10;

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count);

    let (mut session, _fw_channel, _bw_channel) = cluster_fixture
        .create_session_between(src, mid, dst, funding_amount)
        .await?;

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

    cluster_fixture[mid]
        .inner()
        .redeem_all_tickets(0)
        .await
        .context("failed to redeem tickets")?;

    sleep(std::time::Duration::from_secs(5)).await;

    let stats_after = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    assert!(stats_after.redeemed_value > stats_before.redeemed_value);

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_neglect_ticket_on_closing(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, mid, dst] = exclusive_indexes_not_auto_redeeming::<3>();

    let message_count = 3;

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    cluster_fixture[mid]
        .inner()
        .reset_ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    let funding_amount = ticket_price.mul(message_count);
    let (mut session, fw_channel, bw_channel) = cluster_fixture
        .create_session_between(src, mid, dst, funding_amount)
        .await?;

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

    fw_channel.try_close_channels_all_channels().await?;
    bw_channel.try_close_channels_all_channels().await?;

    sleep(std::time::Duration::from_secs(5)).await;

    let stats_after = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    assert_eq!(stats_after.unredeemed_value, HoprBalance::zero());
    assert!(stats_after.neglected_value > HoprBalance::zero());

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_relay_with_reduced_winn_prob(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    cluster_fixture.update_winning_probability(0.2).await?; // reduced to match the relayers custom winn prob

    let [src, mid, dst] = exclusive_indexes_with_auto_redeem_intermediaries::<3>();
    let message_count = 10;

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count);

    let (mut session, _fw_channel, _bw_channel) = cluster_fixture
        .create_session_between(src, mid, dst, funding_amount)
        .await?;

    const BUF_LEN: usize = 400;
    let sent_data = hopr_crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    let stats_before = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    for _ in 1..=message_count {
        tokio::time::timeout(Duration::from_millis(500), session.write_all(&sent_data))
            .await
            .context("write failed")??;
    }

    sleep(std::time::Duration::from_secs(5)).await;

    let stats_after = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    assert!(stats_after.winning_count < stats_before.winning_count + message_count as u128);
    assert!(stats_after.redeemed_value > stats_before.redeemed_value);

    cluster_fixture.update_winning_probability(1.0).await?;

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_relay_with_winn_prob_lower_than_min_win_prob_should_fail(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    cluster_fixture.update_winning_probability(0.5).await?;

    let [src, mid, dst] = exclusive_indexes_auto_redeeming::<3>();
    let message_count = 20;

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count);

    assert!(
        cluster_fixture
            .create_session_between(src, mid, dst, funding_amount)
            .await
            .is_err()
    );

    cluster_fixture.update_winning_probability(1.0).await?;

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn relay_with_winn_prob_higher_than_min_win_prob_should_succeed(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    cluster_fixture.update_winning_probability(0.1).await?;

    let [src, mid, dst] = exclusive_indexes_with_auto_redeem_intermediaries::<3>();
    let message_count = 20;

    let ticket_price = cluster_fixture[src]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count);

    let (mut session, _fw_channel, _bw_channel) = cluster_fixture
        .create_session_between(src, mid, dst, funding_amount)
        .await?;

    const BUF_LEN: usize = 400;
    let sent_data = hopr_crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    let stats_before = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    for _ in 1..=message_count {
        tokio::time::timeout(Duration::from_millis(500), session.write_all(&sent_data))
            .await
            .context("write failed")??;
    }
    sleep(std::time::Duration::from_secs(5)).await;

    let stats_after = cluster_fixture[mid]
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    assert!(stats_after.winning_count > stats_before.winning_count);
    assert!(stats_after.redeemed_value > stats_before.redeemed_value);

    cluster_fixture.update_winning_probability(1.0).await?;

    Ok(())
}
