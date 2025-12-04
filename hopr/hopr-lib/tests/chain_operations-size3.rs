use std::{ops::Mul, time::Duration};

use anyhow::Context;
use hopr_lib::{
    ChannelId, ChannelStatus, HoprBalance,
    testing::{
        fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, size_3_cluster_fixture as cluster},
        hopr::ChannelGuard,
    },
};
use rstest::*;
use serial_test::serial;
use tokio::time::sleep;

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Opens and then closes a channel between two nodes to ensure lifecycle APIs
/// transition through Open and PendingToClose states as expected.
async fn test_open_close_channel(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster.sample_nodes::<2>();

    assert!(
        src.outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .context("failed to get initial channels from src node")?
            .is_empty()
    );

    let channel = ChannelGuard::open_channel_between_nodes(
        src.instance.clone(),
        dst.instance.clone(),
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await
    .context("failed to open channel between nodes")?;

    assert_eq!(
        src.outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .context("failed to get channels from src node")?
            .len(),
        1
    );

    src.inner()
        .close_channel_by_id(&channel.channel_id(0))
        .await
        .context("failed to put channel in PendingToClose state")?;

    sleep(Duration::from_secs(2)).await;

    match src
        .channel_from_hash(&channel.channel_id(0))
        .await
        .context("failed to get channel from id")?
        .status
    {
        ChannelStatus::PendingToClose(_) => (),
        _ => panic!("channel {} should be in PendingToClose state", channel.channel_id(0)),
    }

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Funds a freshly opened channel and asserts the stake reflects the deposit by
/// re-reading the channel and comparing its balance against the funding amount.
async fn channel_funding_should_be_visible_in_channel_stake(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster.sample_nodes::<2>();
    let funding_amount: HoprBalance = FUNDING_AMOUNT.parse()?;

    let channel = ChannelGuard::open_channel_between_nodes(src.instance.clone(), dst.instance.clone(), funding_amount)
        .await
        .context("failed to open channel")?;

    let _ = src.inner().fund_channel(&channel.channel_id(0), funding_amount).await;

    let updated_channel = src
        .channel_from_hash(&channel.channel_id(0))
        .await
        .context("failed to retrieve channel by id")?;

    assert_eq!(updated_channel.balance, funding_amount.mul(2));

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Confirms different channel-lookup APIs return the same channel identifier by
/// having a third node query the channel via parties, sources and destinations.
async fn test_channel_retrieval(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, ext, dst] = cluster.sample_nodes::<3>();

    let channel =
        ChannelGuard::open_channel_between_nodes(src.instance.clone(), dst.instance.clone(), FUNDING_AMOUNT.parse()?)
            .await
            .context("failed to open channel")?;

    let channel_by_parties = ext
        .inner()
        .channel(&src.address(), &dst.address())
        .await
        .context("failed to get channel by parties")?
        .context("channel not found")?;

    let channel_from_ids = ext
        .inner()
        .channels_from(&src.address())
        .await
        .context("failed to get channels from src")?
        .into_iter()
        .map(|c| *c.get_id())
        .collect::<Vec<ChannelId>>();

    let channel_to_ids = ext
        .inner()
        .channels_to(&dst.address())
        .await
        .context("failed to get channels to dst")?
        .into_iter()
        .map(|c| *c.get_id())
        .collect::<Vec<ChannelId>>();

    assert_eq!(channel_by_parties.get_id(), channel.channel_id(0));
    assert!(channel_from_ids.contains(&channel.channel_id(0)));
    assert!(channel_to_ids.contains(&channel.channel_id(0)));

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Exercises the native withdrawal path by sending xDai from one node to another
/// and asserting recipient balance increases while sender balance decreases.
async fn test_withdraw_native(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster.sample_nodes::<2>();

    let withdrawn_amount = "0.005 xDai".parse::<hopr_lib::XDaiBalance>()?;

    let initial_balance_src = src
        .inner()
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let initial_balance_dst = dst
        .inner()
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let _ = src
        .inner()
        .withdraw_native(dst.address(), withdrawn_amount)
        .await
        .context("failed to withdraw native")?;

    let final_balance_src = src
        .inner()
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let final_balance_dst = dst
        .inner()
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    assert_eq!(final_balance_dst, initial_balance_dst + withdrawn_amount);
    assert!(final_balance_src < initial_balance_src - withdrawn_amount); // account for gas
    Ok(())
}
