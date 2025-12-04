use anyhow::Context;
use hopr_internal_types::prelude::WinningProbability;
use hopr_lib::{
    HoprBalance, WxHOPR, XDai, XDaiBalance,
    testing::fixtures::{
        ClusterGuard, DEFAULT_SAFE_ALLOWANCE, INITIAL_NODE_NATIVE, INITIAL_NODE_TOKEN, INITIAL_SAFE_NATIVE,
        INITIAL_SAFE_TOKEN, MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, size_2_cluster_fixture as cluster,
    },
};
use rstest::*;

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Validates a node reports the expected initial balances and safe allowance by
/// reading every wallet component and comparing against the constants seeded in
/// the 2-node cluster fixture.
async fn test_get_balance(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let node = &cluster[0];
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
/// Verifies that startup automatically sets a non-zero ticket price by querying
/// a random node immediately after bootstrapping the 2-node cluster.
async fn ticket_price_is_set_to_non_zero_value_on_start(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();

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
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Confirms ticket price derivation matches the oracle by comparing the on-chain
/// value fetched from a node with the oracle price read from a node.
async fn ticket_price_is_equal_to_oracle_value(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();
    let oracle_price = cluster.get_oracle_ticket_price().await?;

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
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Ensures participating nodes honour the configured minimum incoming win probability
/// by sampling a win-prob-1 node and asserting the reported value matches the constant.
async fn test_check_win_prob_is_default(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes_with_win_prob_1::<1>();

    let winning_prob = node
        .inner()
        .get_minimum_incoming_ticket_win_probability()
        .await
        .context("failed to get winning probability")?;

    assert!(winning_prob.approx_eq(&WinningProbability::try_from_f64(MINIMUM_INCOMING_WIN_PROB)?));

    Ok(())
}
