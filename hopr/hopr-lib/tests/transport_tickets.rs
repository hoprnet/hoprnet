use std::time::Duration;

use anyhow::Context;
use rstest::rstest;
use serial_test::serial;

use hopr_lib::testing::{
    fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes},
    hopr::create_1_hop_session,
};

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[tokio::test]
#[serial]
#[test_log::test]
async fn ticket_statistics_should_reset_when_cleaned(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    use futures::AsyncWriteExt;
    use hopr_lib::{ChannelId, HoprBalance, HoprSession};

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

    let mut session: HoprSession = create_1_hop_session(
        &cluster_fixture[src],
        &cluster_fixture[mid],
        &cluster_fixture[dst],
        None,
        None,
    )
    .await?;

    const BUF_LEN: usize = 5000;
    let sent_data = hopr_crypto_random::random_bytes::<BUF_LEN>();

    let _ = tokio::time::timeout(Duration::from_secs(1), session.write_all(&sent_data))
        .await
        .context("write failed")?;

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

    assert_eq!(stats_before.winning_count, 1); // As winning prob is set to 1

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
