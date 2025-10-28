use std::{str::FromStr, time::Duration};

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
