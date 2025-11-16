use std::{ops::Mul, time::Duration};

use anyhow::Context;
use hopr_internal_types::prelude::WinningProbability;
use hopr_lib::{
    ChannelId,
    testing::{
        fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes, exclusive_indexes_not_auto_redeeming},
        hopr::TestedHopr,
    },
};
use rstest::rstest;
use serial_test::serial;

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn test_get_balance(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::{HoprBalance, WxHOPR, XDai, XDaiBalance};

    let node: &TestedHopr = &cluster_fixture[0];
    let safe_native = node
        .inner()
        .get_safe_balance::<XDai>()
        .await
        .context("should get safe xdai balance")?;
    let native = node
        .inner()
        .get_balance::<XDai>()
        .await
        .context("should get node xdai balance")?;
    let safe_hopr = node
        .inner()
        .get_safe_balance::<WxHOPR>()
        .await
        .context("should get safe hopr balance")?;
    let hopr = node
        .inner()
        .get_balance::<WxHOPR>()
        .await
        .context("should get node hopr balance")?;
    let safe_allowance = node
        .inner()
        .safe_allowance()
        .await
        .context("should get safe hopr allowance")?;

    assert_ne!(safe_native, XDaiBalance::zero());
    assert_ne!(native, XDaiBalance::zero());
    assert_ne!(safe_hopr, HoprBalance::zero());
    assert_eq!(hopr, HoprBalance::zero());
    assert_ne!(safe_allowance, HoprBalance::zero());

    Ok(())
}

// #[rstest]
// #[tokio::test]
// #[serial]
// async fn test_safe_and_module_shouldnt_change(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
// let [idx] = exclusive_indexes::<1>();
// let safe_address = cluster_fixture[idx].inner().get_safe_config();
//
// assert_eq!(
// safe_address.module_address,
// cluster_fixture[idx].safe_config.module_address
// );
// assert_eq!(safe_address.safe_address, cluster_fixture[idx].safe_config.safe_address);
//
// Ok(())
// }

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn test_open_close_channel(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::{ChannelStatus, HoprBalance};
    use tokio::time::sleep;

    let [src, dst] = exclusive_indexes::<2>();

    assert!(
        cluster_fixture[src]
            .outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .context("failed to get initial channels from src node")?
            .is_empty()
    );

    let channel = cluster_fixture[src]
        .inner()
        .open_channel(
            &(cluster_fixture[dst].address()),
            FUNDING_AMOUNT.parse::<HoprBalance>()?,
        )
        .await
        .context("failed to open channel")?;

    assert_eq!(
        cluster_fixture[src]
            .outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .context("failed to get channels from src node")?
            .len(),
        1
    );

    cluster_fixture[src]
        .inner()
        .close_channel_by_id(&channel.channel_id)
        .await
        .context("failed to put channel in PendingToClose state")?;

    sleep(Duration::from_secs(2)).await;

    match cluster_fixture[src]
        .channel_from_hash(&channel.channel_id)
        .await
        .context("failed to get channel from id")?
        .status
    {
        ChannelStatus::PendingToClose(_) => (),
        _ => panic!("channel {} should be in PendingToClose state", channel.channel_id),
    }

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn channel_funding_should_be_visible_in_channel_stake(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    use hopr_lib::HoprBalance;

    let [src, dst] = exclusive_indexes::<2>();
    let funding_amount: HoprBalance = FUNDING_AMOUNT.parse()?;

    let channel = cluster_fixture[src]
        .inner()
        .open_channel(&(cluster_fixture[dst].address()), funding_amount)
        .await
        .context("failed to open channel")?;

    let _ = cluster_fixture[src]
        .inner()
        .fund_channel(&channel.channel_id, funding_amount)
        .await;

    let updated_channel = cluster_fixture[src]
        .channel_from_hash(&channel.channel_id)
        .await
        .context("failed to retrieve channel by id")?;

    assert_eq!(updated_channel.balance, funding_amount.mul(2));

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn test_channel_retrieval(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, ext, dst] = exclusive_indexes::<3>();

    let channel = cluster_fixture[src]
        .inner()
        .open_channel(&(cluster_fixture[dst].address()), FUNDING_AMOUNT.parse()?)
        .await
        .context("failed to open channel")?;

    let channel_by_parties = cluster_fixture[ext]
        .inner()
        .channel(&(cluster_fixture[src].address()), &cluster_fixture[dst].address())
        .await
        .context("failed to get channel by parties")?
        .context("channel not found")?;

    let channel_from_ids = cluster_fixture[ext]
        .inner()
        .channels_from(&(cluster_fixture[src].address()))
        .await
        .context("failed to get channels from src")?
        .into_iter()
        .map(|c| c.get_id())
        .collect::<Vec<ChannelId>>();

    let channel_to_ids = cluster_fixture[ext]
        .inner()
        .channels_to(&(cluster_fixture[dst].address()))
        .await
        .context("failed to get channels to dst")?
        .into_iter()
        .map(|c| c.get_id())
        .collect::<Vec<ChannelId>>();

    assert_eq!(channel_by_parties.get_id(), channel.channel_id);
    assert!(channel_from_ids.contains(&channel.channel_id));
    assert!(channel_to_ids.contains(&channel.channel_id));

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn test_withdraw_native(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = exclusive_indexes::<2>();

    let withdrawn_amount = "0.005 xDai".parse::<hopr_lib::XDaiBalance>()?;

    let initial_balance_src = cluster_fixture[src]
        .inner()
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let initial_balance_dst = cluster_fixture[dst]
        .inner()
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let _ = cluster_fixture[src]
        .inner()
        .withdraw_native(cluster_fixture[dst].address(), withdrawn_amount)
        .await
        .context("failed to withdraw native")?;

    let final_balance_src = cluster_fixture[src]
        .inner()
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let final_balance_dst = cluster_fixture[dst]
        .inner()
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    assert_eq!(final_balance_dst, initial_balance_dst + withdrawn_amount);
    assert!(final_balance_src < initial_balance_src - withdrawn_amount); // account for gas
    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn ticket_price_is_set_to_non_zero_value_on_start(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [node] = exclusive_indexes::<1>();

    let ticket_price = cluster_fixture[node]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    assert!(ticket_price > hopr_lib::HoprBalance::zero());

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn ticket_price_is_equal_to_oracle_value(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [node] = exclusive_indexes::<1>();
    let oracle_price = cluster_fixture.get_oracle_ticket_price().await?;

    let ticket_price = cluster_fixture[node]
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    assert_eq!(ticket_price, oracle_price);

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn test_check_winn_prob_is_default(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [node] = exclusive_indexes_not_auto_redeeming::<1>();

    let winning_prob = cluster_fixture[node]
        .inner()
        .get_minimum_incoming_ticket_win_probability()
        .await
        .context("failed to get winning probability")?;

    assert!(winning_prob.approx_eq(&WinningProbability::ALWAYS));

    Ok(())
}
