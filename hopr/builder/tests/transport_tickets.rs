use std::{ops::Mul, time::Duration};

use anyhow::Context;
use futures::{AsyncWriteExt, StreamExt, pin_mut};
use futures_time::future::FutureExt as _;
use hopr_builder::{
    hopr_lib::{HoprBalance, HoprLibError, UnitaryFloatOps},
    testing::{
        fixtures::{ClusterGuard, MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, cluster_fixture},
        hopr::ChannelGuard,
        wait_until,
    },
};
use rstest::*;
use serial_test::serial;
use tokio::time::sleep;

/// Extra funding per channel to absorb background loopback-probe drain.
/// 3 nodes probing at 1s intervals over ~30s test windows can consume
/// tickets per channel via multi-hop loopback probes.
const PROBING_OVERHEAD: u64 = 30;

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
/// Funds a channel with a small budget and verifies that the relay eventually
/// rejects tickets once the channel is exhausted by a combination of test
/// traffic and background probing.
async fn relaying_message_rejected_when_channel_out_of_funding(
    #[with(vec![TestNodeConfig::default(); 3])] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [src, mid, dst] = cluster_fixture.sample_nodes::<3>();

    let ticket_price = src
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    // Fund with just enough for session creation and a few writes;
    // background probing will drain the remainder over time.
    let funding_amount = ticket_price.mul(PROBING_OVERHEAD + 5);

    let [_fw_channel, _bw_channel, _telemetry_channels]: [ChannelGuard; 3] = cluster_fixture
        .open_channels(&[&[src, mid, dst], &[dst, mid, src], &[src, dst]], funding_amount)
        .await?
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 channel guards"))?;

    let mut session = cluster_fixture.create_session(&[src, mid, dst]).await?;

    const BUF_LEN: usize = 500;
    let sent_data = hopr_api::types::crypto_random::random_bytes::<BUF_LEN>();

    // Confirm the session works initially
    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("initial write failed")??;

    // Continuously send until rejected_value increases (channel funds exhausted).
    // Background probing speeds up fund depletion alongside our writes.
    let write_succeeded_at_least_once = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let write_succeeded_at_least_once_1 = write_succeeded_at_least_once.clone();
    let write_fut = std::pin::pin!(async move {
        loop {
            match session
                .write_all(&sent_data)
                .timeout(futures_time::time::Duration::from_millis(500))
                .await
            {
                Ok(Ok(())) => write_succeeded_at_least_once_1.store(true, std::sync::atomic::Ordering::Release),
                Ok(Err(_)) | Err(_) => {} // write failed or timed out — channel may be drained
            }
            sleep(Duration::from_millis(500)).await;

            // This future never completes
        }
    });

    let ticket_event_stream = mid
        .inner()
        .subscribe_ticket_events()
        .filter_map(|evt| futures::future::ready(evt.try_as_rejected_ticket()));

    pin_mut!(ticket_event_stream);

    let wait_for_rejection_fut = ticket_event_stream.next();

    // Keep on writing and wait for the rejected ticket
    let _ = futures::future::select(write_fut, wait_for_rejection_fut)
        .timeout(futures_time::time::Duration::from_secs(120))
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for ticket rejection after fund exhaustion"))?;

    assert!(
        write_succeeded_at_least_once.load(std::sync::atomic::Ordering::Acquire),
        "at least one write should have succeeded before the channel was exhausted"
    );

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
/// Sends a fixed number of messages so tickets accumulate, then calls
/// `redeem_all_tickets` and asserts unredeemed value shrinks while redeemed grows.
async fn redeem_ticket_on_request(
    #[with(vec![TestNodeConfig::default(); 3])] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [src, mid, dst] = cluster_fixture.sample_nodes_with_win_prob_1::<3>();
    let message_count = 10;

    let ticket_price = src
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count + PROBING_OVERHEAD);

    let [_fw_channel, _bw_channel, _telemetry_channels]: [ChannelGuard; 3] = cluster_fixture
        .open_channels(&[&[src, mid, dst], &[dst, mid, src], &[src, dst]], funding_amount)
        .await?
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 channel guards"))?;

    let mut session = cluster_fixture.create_session(&[src, mid, dst]).await?;

    const BUF_LEN: usize = 400;
    let sent_data = hopr_api::types::crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    for _ in 1..=message_count {
        tokio::time::timeout(Duration::from_millis(1000), session.write_all(&sent_data))
            .await
            .context("write failed")??;
    }

    wait_until(
        || async {
            let stats_before = mid.inner().ticket_statistics().await?;
            Ok::<_, HoprLibError>(stats_before.unredeemed_value >= message_count.into())
        },
        Duration::from_secs(15),
    )
    .await
    .context("failed to wait for: `stats_before.unredeemed_value > message_count.into()`")?;

    let stats_before = mid.inner().ticket_statistics().await?;

    mid.inner()
        .redeem_all_tickets(0)
        .await
        .context("failed to redeem tickets")?;

    #[allow(deprecated)] // TODO: remove once blokli#237 is merged
    wait_until(
        || async {
            let stats_after = mid.inner().ticket_statistics().await?;
            Ok::<_, HoprLibError>(stats_after.redeemed_value > stats_before.redeemed_value)
        },
        Duration::from_secs(5),
    )
    .await
    .context("failed to wait for: `stats_after.redeemed_value() > stats_before.redeemed_value()`")?;

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
/// Demonstrates that closing channels without redeeming moves ticket value into
/// the neglected bucket by closing both paths after traffic has flowed.
async fn neglect_ticket_on_closing(
    #[with(vec![TestNodeConfig::default(); 3])] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [src, mid, dst] = cluster_fixture.sample_nodes_with_win_prob_1::<3>();

    let message_count = 3;

    let ticket_price = src
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    // Snapshot stats right after reset to use as baseline for delta checks
    let stats_after_reset = mid
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    let funding_amount = ticket_price.mul(message_count + PROBING_OVERHEAD);
    let [fw_channel, bw_channel, _telemetry_channels]: [ChannelGuard; 3] = cluster_fixture
        .open_channels(&[&[src, mid, dst], &[dst, mid, src], &[src, dst]], funding_amount)
        .await?
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 channel guards"))?;

    let mut session = cluster_fixture.create_session(&[src, mid, dst]).await?;

    const BUF_LEN: usize = 400;
    let sent_data = hopr_api::types::crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    for _ in 1..=message_count {
        tokio::time::timeout(Duration::from_millis(1000), session.write_all(&sent_data))
            .await
            .context("write failed")??;
    }

    // Wait for unredeemed_value to increase by at least message_count relative
    // to the post-reset snapshot, ensuring our test writes are counted (not just
    // background probing).
    wait_until(
        || async {
            let stats = mid.inner().ticket_statistics().await?;
            Ok::<_, HoprLibError>(
                stats.unredeemed_value >= stats_after_reset.unredeemed_value + HoprBalance::from(message_count),
            )
        },
        Duration::from_secs(5),
    )
    .await
    .context("failed to wait for: `stats.unredeemed_value >= stats_after_reset.unredeemed_value + message_count`")?;

    // Snapshot stats right before closing so we can measure the delta
    let stats_before_close = mid
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    fw_channel.try_close_channels_all_channels().await?;
    bw_channel.try_close_channels_all_channels().await?;

    wait_until(
        || async {
            let stats_after = mid.inner().ticket_statistics().await?;
            // After closing the test channels, neglected value must increase.
            // We use a delta check because background probing may have created
            // unredeemed tickets on other channels that remain open.
            Ok::<_, HoprLibError>(stats_after.neglected_value > stats_before_close.neglected_value)
        },
        Duration::from_secs(5),
    )
    .await
    .context("failed to wait for: `stats_after.neglected_value() > stats_before_close.neglected_value()`")?;

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
/// Lowers the sender's win probability and confirms the relay receives fewer
/// winning tickets by comparing statistics before and after traffic relay.
async fn relay_gets_less_tickets_if_sender_has_lower_win_prob(
    #[with(vec![
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB),
        TestNodeConfig::default(),
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB),
    ])]
    cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [src, mid, dst] = cluster_fixture.sample_nodes_with_win_prob_1_intermediaries::<3>();

    let message_count = 20;

    let ticket_price = src
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price
        .mul(message_count + PROBING_OVERHEAD)
        .div_f64(MINIMUM_INCOMING_WIN_PROB)?;

    let [_fw_channel, _bw_channel, _telemetry_channels]: [ChannelGuard; 3] = cluster_fixture
        .open_channels(&[&[src, mid, dst], &[dst, mid, src], &[src, dst]], funding_amount)
        .await?
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 channel guards"))?;

    let mut session = cluster_fixture.create_session(&[src, mid, dst]).await?;

    const BUF_LEN: usize = 400;
    let sent_data = hopr_api::types::crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    let stats_before = mid
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    for _ in 1..=message_count {
        tokio::time::timeout(Duration::from_millis(500), session.write_all(&sent_data))
            .await
            .context("write failed")??;
    }

    sleep(Duration::from_secs(5)).await;

    mid.inner()
        .redeem_all_tickets(0)
        .await
        .context("failed to redeem tickets")?;

    #[allow(deprecated)] // TODO: remove once blokli#237 is merged
    wait_until(
        || async {
            let stats_after = mid.inner().ticket_statistics().await?;
            Ok::<_, HoprLibError>(
                stats_after.winning_tickets < stats_before.winning_tickets + message_count as u128
                    && stats_after.redeemed_value > stats_before.redeemed_value,
            )
        },
        Duration::from_secs(5),
    )
    .await
    .context(
        "failed to wait for: `stats_after.winning_tickets < stats_before.winning_tickets + message_count as u128 && \
         stats_after.redeemed_value() > stats_before.redeemed_value()`",
    )?;

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
/// Drops the cluster-wide minimum win probability threshold and asserts session
/// creation fails when relay win probability is insufficient.
async fn ticket_with_win_prob_lower_than_min_win_prob_should_be_rejected(
    #[with(vec![
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB),
        TestNodeConfig::default(),
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB),
    ])]
    cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    cluster_fixture.update_winning_probability(0.5).await?;

    let [src, mid, dst] = cluster_fixture.sample_nodes_with_win_prob_1_intermediaries::<3>();
    let message_count = 20;

    let ticket_price = src
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count + 2 + PROBING_OVERHEAD);

    let _channel_guards = cluster_fixture
        .open_channels(&[&[src, mid, dst], &[dst, mid, src], &[src, dst]], funding_amount)
        .await?;

    assert!(cluster_fixture.create_session(&[src, mid, dst]).await.is_err());

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
/// Keeps relay win probability above the minimum, relays traffic, redeems all
/// tickets and asserts the statistics reflect the successful redemptions.
async fn relay_with_win_prob_higher_than_min_win_prob_should_succeed(
    #[with(vec![
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB),
        TestNodeConfig::default(),
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB),
    ])]
    cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [src, mid, dst] = cluster_fixture.sample_nodes_with_win_prob_1_intermediaries::<3>();
    let message_count = 20;

    let ticket_price = src
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;
    let funding_amount = ticket_price.mul(message_count + 2 + PROBING_OVERHEAD);

    let [_fw_channel, _bw_channel, _telemetry_channel]: [ChannelGuard; 3] = cluster_fixture
        .open_channels(&[&[src, mid, dst], &[dst, mid, src], &[src, dst]], funding_amount)
        .await?
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 3 channel guards"))?;

    let mut session = cluster_fixture.create_session(&[src, mid, dst]).await?;

    const BUF_LEN: usize = 400;
    let sent_data = hopr_api::types::crypto_random::random_bytes::<BUF_LEN>();

    tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")??;

    let stats_before = mid
        .inner()
        .ticket_statistics()
        .await
        .context("failed to get ticket statistics")?;

    for _ in 1..=message_count {
        tokio::time::timeout(Duration::from_millis(500), session.write_all(&sent_data))
            .await
            .context("write failed")??;
    }
    sleep(Duration::from_secs(5)).await;

    mid.inner()
        .redeem_all_tickets(0)
        .await
        .context("failed to redeem tickets")?;

    #[allow(deprecated)] // TODO: remove once blokli#237 is merged
    wait_until(
        || async {
            let stats_after = mid.inner().ticket_statistics().await?;
            Ok::<_, HoprLibError>(
                stats_after.redeemed_value > stats_before.redeemed_value
                    && stats_after.winning_tickets > stats_before.winning_tickets,
            )
        },
        Duration::from_secs(5),
    )
    .await
    .context(
        "failed to wait for: `stats_after.redeemed_value() > stats_before.redeemed_value() && \
         stats_after.winning_tickets > stats_before.winning_tickets`",
    )?;

    Ok(())
}
