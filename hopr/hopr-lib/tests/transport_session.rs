use std::time::Duration;

use anyhow::Context;
use futures::AsyncWriteExt;
use hopr_lib::{Address, HoprBalance, HoprSession, SurbBalancerConfig};
use rstest::rstest;
use serial_test::serial;
use tokio::time::sleep;

use hopr_lib::testing::fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes};

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_create_0_hop_session(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
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
#[test_log::test]
async fn test_create_1_hop_session(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
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
