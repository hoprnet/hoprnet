use std::{ops::Mul, time::Duration};

use anyhow::Context;
use hopr_internal_types::prelude::WinningProbability;
use hopr_lib::{
    ChannelId,
    testing::{
        fixtures::{
            ClusterGuard, DEFAULT_SAFE_ALLOWANCE, INITIAL_NODE_NATIVE, INITIAL_NODE_TOKEN, INITIAL_SAFE_NATIVE,
            INITIAL_SAFE_TOKEN, MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, cluster_fixture,
        },
        hopr::TestedHopr,
    },
};
use rstest::*;
use serial_test::serial;

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn test_get_balance(#[with(2)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
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

    assert_eq!(safe_native, XDaiBalance::new_base(INITIAL_SAFE_NATIVE));
    assert_ne!(native, XDaiBalance::new_base(INITIAL_NODE_NATIVE)); // Node made some TXs
    assert_eq!(safe_hopr, HoprBalance::new_base(INITIAL_SAFE_TOKEN));
    assert_eq!(hopr, HoprBalance::new_base(INITIAL_NODE_TOKEN));
    assert_eq!(safe_allowance, HoprBalance::new_base(DEFAULT_SAFE_ALLOWANCE));

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn test_open_close_channel(cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::{ChannelStatus, HoprBalance};
    use tokio::time::sleep;

    let [src, dst] = cluster_fixture.sample_nodes::<2>();

    assert!(
        src.outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .context("failed to get initial channels from src node")?
            .is_empty()
    );

    let channel = src
        .inner()
        .open_channel(&dst.address(), FUNDING_AMOUNT.parse::<HoprBalance>()?)
        .await
        .context("failed to open channel")?;

    assert_eq!(
        src.outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .context("failed to get channels from src node")?
            .len(),
        1
    );

    src.inner()
        .close_channel_by_id(&channel.channel_id)
        .await
        .context("failed to put channel in PendingToClose state")?;

    sleep(Duration::from_secs(2)).await;

    match src
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
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn channel_funding_should_be_visible_in_channel_stake(cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::HoprBalance;

    let [src, dst] = cluster_fixture.sample_nodes::<2>();
    let funding_amount: HoprBalance = FUNDING_AMOUNT.parse()?;

    let channel = src
        .inner()
        .open_channel(&dst.address(), funding_amount)
        .await
        .context("failed to open channel")?;

    let _ = src.inner().fund_channel(&channel.channel_id, funding_amount).await;

    let updated_channel = src
        .channel_from_hash(&channel.channel_id)
        .await
        .context("failed to retrieve channel by id")?;

    assert_eq!(updated_channel.balance, funding_amount.mul(2));

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn test_channel_retrieval(cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, ext, dst] = cluster_fixture.sample_nodes::<3>();

    let channel = src
        .inner()
        .open_channel(&dst.address(), FUNDING_AMOUNT.parse()?)
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
        .map(|c| c.get_id())
        .collect::<Vec<ChannelId>>();

    let channel_to_ids = ext
        .inner()
        .channels_to(&dst.address())
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
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn test_withdraw_native(cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster_fixture.sample_nodes::<2>();

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

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn ticket_price_is_set_to_non_zero_value_on_start(
    #[with(2)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [node] = cluster_fixture.sample_nodes::<1>();

    let ticket_price = node
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    assert!(ticket_price > hopr_lib::HoprBalance::zero());

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_mins(1))]
#[serial]
async fn ticket_price_is_equal_to_oracle_value(#[with(2)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster_fixture.sample_nodes::<1>();
    let oracle_price = cluster_fixture.get_oracle_ticket_price().await?;

    let ticket_price = node
        .inner()
        .get_ticket_price()
        .await
        .context("failed to get ticket price")?;

    assert_eq!(ticket_price, oracle_price);

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_mins(1))]
#[serial]
async fn test_check_win_prob_is_default(#[with(2)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster_fixture.sample_nodes_with_win_prob_1::<1>();

    let winning_prob = node
        .inner()
        .get_minimum_incoming_ticket_win_probability()
        .await
        .context("failed to get winning probability")?;

    assert!(winning_prob.approx_eq(&WinningProbability::try_from_f64(MINIMUM_INCOMING_WIN_PROB)?));

    Ok(())
}
