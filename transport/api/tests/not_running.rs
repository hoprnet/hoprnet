/// Integration tests for `HoprTransport` using lightweight in-memory test doubles.
///
/// These tests exercise construction, pre-run behavior, address filtering, and
/// utility functions without requiring a real database or chain connector.
mod stubs;

use std::str::FromStr;

use anyhow::Context;
use hex_literal::hex;
use hopr_api::types::crypto::prelude::{ChainKeypair, Keypair, OffchainKeypair};
use hopr_network_graph::ChannelGraph;
use hopr_transport::{HoprProtocolConfig, HoprTransport, Multiaddr, PeerId, peer_id_to_public_key};

use crate::stubs::StubNet;

lazy_static::lazy_static! {
    static ref PEER_KEYS: Vec<(OffchainKeypair, ChainKeypair)> = [
        (hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"),
         hex!("4db3ac225fdcc7e20bf887cd90bbd62dc6bd41ce8ba5c23cc9ae0bf56e20d056")),
        (hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca"),
         hex!("1d40c69c179528bbdf49c2254e93400b485f47d7d2fa84aae280af5a31c1918b")),
        (hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"),
         hex!("99facd2cd33664d65826ad220920a6b356e31d18c1ce1734303b70a962664d71")),
    ]
    .iter()
    .map(|(off_priv, chain_priv)| {
        (
            OffchainKeypair::from_secret(off_priv).expect("keypair"),
            ChainKeypair::from_secret(chain_priv).expect("keypair"),
        )
    })
    .collect();
}

type TestTransport = HoprTransport<stubs::StubChain, ChannelGraph, StubNet>;

fn create_stubbed_transport(peer_index: usize) -> anyhow::Result<TestTransport> {
    create_stubbed_transport_with_addrs(peer_index, vec![])
}

fn create_stubbed_transport_with_addrs(
    peer_index: usize,
    my_multiaddresses: Vec<Multiaddr>,
) -> anyhow::Result<TestTransport> {
    create_stubbed_transport_with_cfg(peer_index, my_multiaddresses, HoprProtocolConfig::default())
}

fn create_stubbed_transport_with_cfg(
    peer_index: usize,
    my_multiaddresses: Vec<Multiaddr>,
    cfg: HoprProtocolConfig,
) -> anyhow::Result<TestTransport> {
    let (offchain, chain_kp) = &PEER_KEYS[peer_index];
    let graph = ChannelGraph::new(*offchain.public());
    let chain = stubs::StubChain::new(offchain, chain_kp);

    Ok(HoprTransport::new(
        (chain_kp, offchain),
        chain,
        graph,
        my_multiaddresses,
        cfg,
    )?)
}

// --- Construction tests ---

#[tokio::test]
async fn transport_constructs_with_default_config() -> anyhow::Result<()> {
    let _transport = create_stubbed_transport(0)?;
    Ok(())
}

#[tokio::test]
async fn transport_constructs_for_all_peers() -> anyhow::Result<()> {
    for i in 0..PEER_KEYS.len() {
        create_stubbed_transport(i)?;
    }
    Ok(())
}

#[tokio::test]
async fn transport_constructs_with_multiaddresses() -> anyhow::Result<()> {
    let addrs = vec![
        Multiaddr::from_str("/ip4/1.2.3.4/tcp/9000")?,
        Multiaddr::from_str("/ip4/10.0.0.1/tcp/9001")?,
    ];
    let _transport = create_stubbed_transport_with_addrs(0, addrs)?;
    Ok(())
}

// --- Pre-run behavior tests ---

#[tokio::test]
async fn local_multiaddresses_returns_configured_before_run() -> anyhow::Result<()> {
    let addrs = vec![Multiaddr::from_str("/ip4/1.2.3.4/tcp/9000")?];
    let transport = create_stubbed_transport_with_addrs(0, addrs)?;

    let local = transport.local_multiaddresses();
    assert_eq!(local.len(), 1);
    insta::assert_yaml_snapshot!(local.iter().map(|a| a.to_string()).collect::<Vec<_>>());
    Ok(())
}

#[tokio::test]
async fn listening_multiaddresses_empty_before_run() -> anyhow::Result<()> {
    let transport = create_stubbed_transport(0)?;
    let addrs = transport.listening_multiaddresses().await;
    assert!(addrs.is_empty());
    Ok(())
}

#[tokio::test]
async fn network_health_red_before_run() -> anyhow::Result<()> {
    let transport = create_stubbed_transport(0)?;
    let health = transport.network_health().await;
    insta::assert_yaml_snapshot!(format!("{health:?}"));
    Ok(())
}

#[tokio::test]
async fn network_connected_peers_errors_before_run() -> anyhow::Result<()> {
    let transport = create_stubbed_transport(0)?;
    assert!(transport.network_connected_peers().await.is_err());
    Ok(())
}

#[tokio::test]
async fn ping_errors_before_run() -> anyhow::Result<()> {
    let transport = create_stubbed_transport(0)?;
    assert!(transport.ping(PEER_KEYS[1].0.public()).await.is_err());
    Ok(())
}

#[tokio::test]
async fn ping_to_self_rejected() -> anyhow::Result<()> {
    let transport = create_stubbed_transport(0)?;
    let err = transport
        .ping(PEER_KEYS[0].0.public())
        .await
        .err()
        .context("pinging self should fail")?;
    let msg = format!("{err}");
    assert!(msg.contains("self"), "error should mention self: {msg}");
    Ok(())
}

#[tokio::test]
async fn network_observed_multiaddresses_empty_before_run() -> anyhow::Result<()> {
    let transport = create_stubbed_transport(0)?;
    let addrs = transport.network_observed_multiaddresses(PEER_KEYS[1].0.public()).await;
    assert!(addrs.is_empty());
    Ok(())
}

#[tokio::test]
async fn network_peer_observations_none_for_unknown_peer() -> anyhow::Result<()> {
    let transport = create_stubbed_transport(0)?;
    assert!(transport.network_peer_observations(PEER_KEYS[1].0.public()).is_none());
    Ok(())
}

// --- Address filtering tests ---

#[tokio::test]
async fn announceable_filters_out_private_addresses() -> anyhow::Result<()> {
    let addrs = vec![
        Multiaddr::from_str("/ip4/1.2.3.4/tcp/9000")?,
        Multiaddr::from_str("/ip4/192.168.1.1/tcp/9001")?,
    ];
    let transport = create_stubbed_transport_with_addrs(0, addrs)?;

    let announceable = transport.announceable_multiaddresses();
    for addr in &announceable {
        let s = addr.to_string();
        assert!(!s.contains("192.168"), "private address leaked: {s}");
    }
    Ok(())
}

#[tokio::test]
async fn announceable_includes_private_when_configured() -> anyhow::Result<()> {
    let addrs = vec![Multiaddr::from_str("/ip4/192.168.1.1/tcp/9001")?];
    let mut cfg = HoprProtocolConfig::default();
    cfg.transport.announce_local_addresses = true;

    let transport = create_stubbed_transport_with_cfg(0, addrs, cfg)?;

    let announceable = transport.announceable_multiaddresses();
    assert!(!announceable.is_empty(), "private should be included");
    Ok(())
}

#[tokio::test]
async fn announceable_multiaddresses_snapshot() -> anyhow::Result<()> {
    let addrs = vec![
        Multiaddr::from_str("/ip4/1.2.3.4/tcp/9000")?,
        Multiaddr::from_str("/dns4/example.com/tcp/443")?,
    ];
    let transport = create_stubbed_transport_with_addrs(0, addrs)?;

    let announceable: Vec<String> = transport
        .announceable_multiaddresses()
        .iter()
        .map(|a| a.to_string())
        .collect();
    insta::assert_yaml_snapshot!(announceable);
    Ok(())
}

// --- peer_id_to_public_key utility ---

#[tokio::test]
async fn peer_id_converts_to_public_key() -> anyhow::Result<()> {
    let peer_id = PeerId::from(*PEER_KEYS[0].0.public());
    let key = peer_id_to_public_key(&peer_id).await?;
    assert_eq!(key, *PEER_KEYS[0].0.public());
    Ok(())
}

#[tokio::test]
async fn peer_id_conversion_is_repeatable() -> anyhow::Result<()> {
    let peer_id = PeerId::from(*PEER_KEYS[1].0.public());
    let k1 = peer_id_to_public_key(&peer_id).await?;
    let k2 = peer_id_to_public_key(&peer_id).await?;
    assert_eq!(k1, k2);
    Ok(())
}

// --- HoprTransportProcess display ---

#[test]
fn transport_process_display_names_are_stable() {
    use hopr_transport::HoprTransportProcess;
    use hopr_transport_protocol::PacketPipelineProcesses;

    let names: Vec<String> = vec![
        HoprTransportProcess::Medium.to_string(),
        HoprTransportProcess::Pipeline(PacketPipelineProcesses::MsgIn).to_string(),
        HoprTransportProcess::Pipeline(PacketPipelineProcesses::MsgOut).to_string(),
        HoprTransportProcess::Pipeline(PacketPipelineProcesses::AckIn).to_string(),
        HoprTransportProcess::Pipeline(PacketPipelineProcesses::AckOut).to_string(),
        HoprTransportProcess::SessionsManagement(0).to_string(),
        HoprTransportProcess::OutgoingIndexSync.to_string(),
    ];
    insta::assert_yaml_snapshot!(names);
}

// --- MixerConfig defaults ---

#[test]
fn mixer_config_default_values_snapshot() {
    let cfg = hopr_transport_mixer::config::MixerConfig::default();
    insta::assert_yaml_snapshot!(cfg);
}
