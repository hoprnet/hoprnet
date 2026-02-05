use std::future::Future;

use futures::{StreamExt, TryFutureExt, TryStreamExt};
use futures_time::future::FutureExt as FuturesTimeExt;
use hopr_chain_connector::{
    HoprBlockchainSafeConnector,
    testing::{BlokliTestClient, FullStateEmulator},
};
use hopr_db_node::HoprNodeDb;
use hopr_network_graph::immediate::ImmediateNeighborChannelGraph;
use hopr_transport_p2p::{HoprNetwork, UninitializedPeerStore};

pub mod dummies;
pub mod fixtures;
pub mod hopr;

pub(crate) type TestingConnector = std::sync::Arc<HoprBlockchainSafeConnector<BlokliTestClient<FullStateEmulator>>>;
pub(crate) type TestingGraph = ImmediateNeighborChannelGraph<UninitializedPeerStore>;
pub(crate) type TestingHopr = crate::Hopr<TestingConnector, HoprNodeDb, TestingGraph, HoprNetwork>;

/// Waits until either the given async `predicate` returns true or the `timeout` is reached.
///
/// The predicate is sampled 10 times inside the `timeout` period, but not faster than every 100ms (for timeouts < 1s).
pub async fn wait_until<F, Fut, E>(predicate: F, timeout: std::time::Duration) -> anyhow::Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<bool, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    if !predicate().await? {
        futures_time::stream::interval(futures_time::time::Duration::from(
            (timeout / 10).max(std::time::Duration::from_millis(100)),
        ))
        .map(Ok)
        .try_skip_while(|_| predicate().and_then(|v| futures::future::ok(!v)))
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(timeout))
        .await?;
    }

    Ok(())
}
