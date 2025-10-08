mod common;

use std::{ops::Mul, time::Duration};

use anyhow::Context;
use common::{
    fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes},
    hopr_tester::HoprTester,
};
use hopr_api::Address;
use hopr_lib::ChannelId;
use rstest::rstest;
use serial_test::serial;

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[tokio::test]
#[serial]
async fn test_get_balance(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::{HoprBalance, WxHOPR, XDai, XDaiBalance};

    let node: &HoprTester = &cluster_fixture[0];
    let safe_native = node
        .get_safe_balance::<XDai>()
        .await
        .context("should get safe xdai balance")?;
    let native = node
        .get_balance::<XDai>()
        .await
        .context("should get node xdai balance")?;
    let safe_hopr = node
        .get_safe_balance::<WxHOPR>()
        .await
        .context("should get safe hopr balance")?;
    let hopr = node
        .get_balance::<WxHOPR>()
        .await
        .context("should get node hopr balance")?;
    let safe_allowance = node.safe_allowance().await.context("should get safe hopr allowance")?;

    assert_ne!(safe_native, XDaiBalance::zero());
    assert_ne!(native, XDaiBalance::zero());
    assert_ne!(safe_hopr, HoprBalance::zero());
    assert_eq!(hopr, HoprBalance::zero());
    assert_ne!(safe_allowance, HoprBalance::zero());

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_safe_and_module_shouldnt_change(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [idx] = exclusive_indexes::<1>();
    let safe_address = cluster_fixture[idx].inner().get_safe_config();

    assert_eq!(
        safe_address.module_address,
        cluster_fixture[idx].safe_config.module_address
    );
    assert_eq!(safe_address.safe_address, cluster_fixture[idx].safe_config.safe_address);
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_get_public_node_is_not_empty(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [idx] = exclusive_indexes::<1>();

    let config = cluster_fixture[idx]
        .inner()
        .get_public_nodes()
        .await
        .context("should get public nodes")?;

    assert!(!config.is_empty());

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]

async fn test_ping_peer_inside_cluster(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = exclusive_indexes::<2>();

    let _ = cluster_fixture[src]
        .inner()
        .ping(&cluster_fixture[dst].peer_id())
        .await
        .context("failed to ping peer")?;

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]

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
#[serial]

async fn test_open_close_channel(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::{ChannelStatus, HoprBalance};
    use tokio::time::sleep;

    let [src, dst] = exclusive_indexes::<2>();

    assert!(
        cluster_fixture[src]
            .outgoing_channels_by_status(ChannelStatus::Open)
            .await
            .context("failed to get channels from src node")?
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
        _ => panic!("channel should be in PendingToClose state"),
    }

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
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
#[tokio::test]
#[serial]
async fn test_reset_ticket_statistics(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
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

    let mut session: HoprSession = cluster_fixture[src]
        .create_1_hop_session(&cluster_fixture[mid], &cluster_fixture[dst], None, None)
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

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_create_0_hop_session(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::HoprSession;

    let [src, dst] = exclusive_indexes::<2>();
    let _session: HoprSession = cluster_fixture[src]
        .create_raw_0_hop_session(&cluster_fixture[dst])
        .await?;

    // TODO: check here that the destination sees the new session created

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_create_1_hop_session(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::HoprSession;

    let [src, mid, dst] = exclusive_indexes::<3>();
    let _session: HoprSession = cluster_fixture[src]
        .create_1_hop_session(&cluster_fixture[mid], &cluster_fixture[dst], None, None)
        .await?;

    // TODO: check here that the destination sees the new session created

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_keep_alive_session(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use futures::AsyncWriteExt;
    use hopr_lib::HoprSession;
    use tokio::time::sleep;

    let [src, dst] = exclusive_indexes::<2>();
    let mut session: HoprSession = cluster_fixture[src]
        .create_raw_0_hop_session(&cluster_fixture[dst])
        .await?;

    sleep(Duration::from_secs(2)).await;

    cluster_fixture[src]
        .inner()
        .keep_alive_session(&session.id())
        .await
        .context("failed to keep alive session")?;

    sleep(Duration::from_secs(2)).await;

    session
        .write_all(b"ping")
        .await
        .context("failed to write to session before session sunsets")?;

    sleep(Duration::from_secs(2)).await;

    let _ = session.write_all(b"ping").await.is_err();

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_session_surb_balancer_config(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_lib::{HoprBalance, HoprSession, SurbBalancerConfig};

    let [src, mid, dst] = exclusive_indexes::<3>();
    let exp_config = SurbBalancerConfig {
        target_surb_buffer_size: 10,
        max_surbs_per_sec: 100,
        ..Default::default()
    };

    let _ = cluster_fixture[src]
        .inner()
        .open_channel(
            &(cluster_fixture[mid].address()),
            FUNDING_AMOUNT.parse::<HoprBalance>()?,
        )
        .await
        .context("failed to open forward channel")?;

    let _ = cluster_fixture[dst]
        .inner()
        .open_channel(
            &(cluster_fixture[mid].address()),
            FUNDING_AMOUNT.parse::<HoprBalance>()?,
        )
        .await
        .context("failed to open return channel")?;

    let session: HoprSession = cluster_fixture[src]
        .create_1_hop_session(&cluster_fixture[mid], &cluster_fixture[dst], None, Some(exp_config))
        .await?;

    let config = cluster_fixture[src]
        .inner()
        .get_session_surb_balancer_config(&session.id())
        .await
        .context("failed to get surb balancer config")?;

    assert_eq!(config, Some(exp_config));

    cluster_fixture[src]
        .inner()
        .update_session_surb_balancer_config(&session.id(), SurbBalancerConfig::default())
        .await
        .context("failed to update surb balancer config")?;

    let config = cluster_fixture[src]
        .inner()
        .get_session_surb_balancer_config(&session.id())
        .await
        .context("failed to get surb balancer config")?;

    assert_eq!(config, Some(SurbBalancerConfig::default()));

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_announced_accounts(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [idx1, idx2] = exclusive_indexes::<2>();

    let accounts_addresses_1 = cluster_fixture[idx1]
        .inner()
        .accounts_announced_on_chain()
        .await
        .context("failed to get announced accounts")?
        .into_iter()
        .map(|acc| acc.chain_addr)
        .collect::<Vec<Address>>();

    let accounts_addresses_2 = cluster_fixture[idx2]
        .inner()
        .accounts_announced_on_chain()
        .await
        .context("failed to get announced accounts")?
        .into_iter()
        .map(|acc| acc.chain_addr)
        .collect::<Vec<Address>>();

    assert!(accounts_addresses_1.contains(&cluster_fixture[idx1].address()));
    assert!(accounts_addresses_1.contains(&cluster_fixture[idx2].address()));

    assert_eq!(accounts_addresses_1, accounts_addresses_2);
    Ok(())
}

#[rstest]
#[tokio::test]
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

// #[rstest]
// #[tokio::test]
// async fn test_corrupted_channels_TODO(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
//     // TODO
//     Ok(())
// }

#[rstest]
#[tokio::test]
#[serial]
async fn test_withdraw_native(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = exclusive_indexes::<2>();

    let withdrawn_amount = "0.005 xDai".parse::<hopr_lib::XDaiBalance>()?;

    let initial_balance_src = cluster_fixture[src]
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let initial_balance_dst = cluster_fixture[dst]
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let _ = cluster_fixture[src]
        .inner()
        .withdraw_native(cluster_fixture[dst].address(), withdrawn_amount)
        .await
        .context("failed to withdraw native")?;

    let final_balance_src = cluster_fixture[src]
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    let final_balance_dst = cluster_fixture[dst]
        .get_balance::<hopr_lib::XDai>()
        .await
        .context("should get node xdai balance")?;

    assert_eq!(final_balance_dst, initial_balance_dst + withdrawn_amount);
    assert!(final_balance_src < initial_balance_src - withdrawn_amount); // account for gas
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_peerid_and_chain_key_conversion(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [candidate, tester] = exclusive_indexes::<2>();

    let peer_id = cluster_fixture[candidate].peer_id();
    let chain_key = cluster_fixture[candidate].address();

    let derived_chain_key = cluster_fixture[tester]
        .inner()
        .peerid_to_chain_key(&peer_id)
        .await
        .context("failed to convert peer id to chain key")?
        .context("no chain key found for peer id")?;

    let derived_peer_id = cluster_fixture[tester]
        .inner()
        .chain_key_to_peerid(&chain_key)
        .await
        .context("failed to convert chain key to peer id")?
        .context("no peer id found for chain key")?;

    assert_eq!(chain_key, derived_chain_key);
    assert_eq!(peer_id, derived_peer_id);

    Ok(())
}
