//! DOT (Graphviz) rendering for the channel graph.
//!
//! Gated behind the `graph-api` feature.

use std::fmt::Write;

use hopr_api::{
    OffchainPublicKey,
    graph::traits::{EdgeLinkObservable, EdgeObservableRead, EdgeProtocolObservable},
};

use crate::ChannelGraph;

/// Renders the connected subgraph of `graph` as a DOT (Graphviz) digraph.
///
/// Isolated nodes (those with no incoming or outgoing edges) are excluded.
/// Each node is labeled with `OffchainPublicKey` in hex.
/// Edges carry quality annotations: score, latency (ms), and capacity when available.
pub fn render_dot(graph: &ChannelGraph) -> String {
    render_dot_with_labels(graph, |key| format!("{key}"))
}

/// Renders the connected subgraph of `graph` as a DOT digraph, using the
/// provided `label_fn` to produce the node label for each [`OffchainPublicKey`].
///
/// This allows callers to substitute onchain addresses or any other label format
/// while keeping the rendering logic shared.
pub fn render_dot_with_labels(graph: &ChannelGraph, label_fn: impl Fn(&OffchainPublicKey) -> String) -> String {
    render_edges_as_dot(&graph.connected_edges(), &label_fn)
}

/// Like [`render_dot_with_labels`], but only includes edges reachable from the
/// current node via directed BFS. Disconnected subgraphs are excluded.
pub fn render_dot_reachable_with_labels(
    graph: &ChannelGraph,
    label_fn: impl Fn(&OffchainPublicKey) -> String,
) -> String {
    render_edges_as_dot(&graph.reachable_edges(), &label_fn)
}

pub fn render_edges_as_dot(
    edges: &[(OffchainPublicKey, OffchainPublicKey, crate::Observations)],
    label_fn: &impl Fn(&OffchainPublicKey) -> String,
) -> String {
    let mut out = String::from("digraph hopr {\n");

    for (src, dst, obs) in edges {
        let src_label = label_fn(src);
        let dst_label = label_fn(dst);

        let mut attrs = Vec::new();
        let score = obs.score();
        attrs.push(format!("score={score:.2}"));

        if let Some(imm) = obs.immediate_qos()
            && let Some(latency) = imm.average_latency()
        {
            attrs.push(format!("lat={}ms", latency.as_millis()));
        }

        if let Some(inter) = obs.intermediate_qos()
            && let Some(cap) = inter.capacity()
        {
            attrs.push(format!("cap={cap}"));
        }

        let label = attrs.join(" ");
        let _ = writeln!(out, "  \"{src_label}\" -> \"{dst_label}\" [label=\"{label}\"];");
    }

    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use hopr_api::{
        graph::{
            NetworkGraphWrite,
            traits::{EdgeObservableWrite, EdgeWeightType},
        },
        types::crypto::prelude::{Keypair, OffchainKeypair},
    };

    use super::*;
    use crate::ChannelGraph;

    fn label_from_map<'a>(
        addr_map: &'a HashMap<hopr_api::OffchainPublicKey, String>,
    ) -> impl Fn(&hopr_api::OffchainPublicKey) -> String + 'a {
        |key| addr_map.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    const SECRET_0: [u8; 32] = hex_literal::hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d");
    const SECRET_1: [u8; 32] = hex_literal::hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a");
    const SECRET_2: [u8; 32] = hex_literal::hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299");
    const SECRET_3: [u8; 32] = hex_literal::hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e");

    fn pubkey(secret: &[u8; 32]) -> hopr_api::OffchainPublicKey {
        *OffchainKeypair::from_secret(secret).expect("valid secret").public()
    }

    #[test]
    fn empty_graph_should_render_empty_digraph() {
        let me = pubkey(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let dot = render_dot(&graph);
        assert_eq!(dot, "digraph hopr {\n}\n");
    }

    #[test]
    fn isolated_nodes_should_be_excluded_from_dot() {
        let me = pubkey(&SECRET_0);
        let graph = ChannelGraph::new(me);
        graph.add_node(pubkey(&SECRET_1));
        // No edges — only isolated nodes.
        let dot = render_dot(&graph);
        assert_eq!(dot, "digraph hopr {\n}\n");
    }

    #[test]
    fn diamond_topology_should_render_four_edges() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let dest = pubkey(&SECRET_3);
        let graph = ChannelGraph::new(me);
        for n in [a, b, dest] {
            graph.add_node(n);
        }
        graph.add_edge(&me, &a)?;
        graph.add_edge(&me, &b)?;
        graph.add_edge(&a, &dest)?;
        graph.add_edge(&b, &dest)?;

        let dot = render_dot(&graph);
        // Four "->" occurrences, one per edge.
        assert_eq!(dot.matches("->").count(), 4);
        assert!(dot.starts_with("digraph hopr {"));
        assert!(dot.ends_with("}\n"));
        Ok(())
    }

    #[test]
    fn observations_should_appear_as_edge_labels() {
        let me = pubkey(&SECRET_0);
        let peer = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.upsert_edge(&me, &peer, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(50))));
            obs.record(EdgeWeightType::Capacity(Some(1000)));
        });

        let dot = render_dot(&graph);
        assert!(dot.contains("lat=50ms"), "should contain latency: {dot}");
        assert!(dot.contains("cap=1000"), "should contain capacity: {dot}");
        assert!(dot.contains("score="), "should contain score: {dot}");
    }

    #[test]
    fn custom_labels_should_replace_offchain_keys() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let peer = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        let mut addr_map: HashMap<hopr_api::OffchainPublicKey, String> = HashMap::new();
        addr_map.insert(me, "0xaaaa000000000000000000000000000000000001".into());
        addr_map.insert(peer, "0xbbbb000000000000000000000000000000000002".into());

        let dot = render_dot_with_labels(&graph, label_from_map(&addr_map));

        assert!(
            dot.contains("0xaaaa000000000000000000000000000000000001"),
            "source should use onchain address: {dot}"
        );
        assert!(
            dot.contains("0xbbbb000000000000000000000000000000000002"),
            "destination should use onchain address: {dot}"
        );
        // Offchain keys should NOT appear
        assert!(
            !dot.contains(&format!("{me}")),
            "offchain key for 'me' should be replaced: {dot}"
        );
        assert!(
            !dot.contains(&format!("{peer}")),
            "offchain key for 'peer' should be replaced: {dot}"
        );
        Ok(())
    }

    #[test]
    fn custom_labels_should_fall_back_to_offchain_key_when_unmapped() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let peer = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        let mut addr_map: HashMap<hopr_api::OffchainPublicKey, String> = HashMap::new();
        addr_map.insert(me, "0xcccc000000000000000000000000000000000003".into());

        let dot = render_dot_with_labels(&graph, label_from_map(&addr_map));

        assert!(
            dot.contains("0xcccc000000000000000000000000000000000003"),
            "mapped node should use onchain address: {dot}"
        );
        assert!(
            dot.contains(&format!("{peer}")),
            "unmapped node should fall back to offchain key: {dot}"
        );
        Ok(())
    }

    #[test]
    fn custom_labels_should_preserve_edge_attributes() {
        let me = pubkey(&SECRET_0);
        let peer = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.upsert_edge(&me, &peer, |obs| {
            obs.record(EdgeWeightType::Connected(true));
            obs.record(EdgeWeightType::Immediate(Ok(std::time::Duration::from_millis(120))));
            obs.record(EdgeWeightType::Capacity(Some(500)));
        });

        let mut addr_map: HashMap<hopr_api::OffchainPublicKey, String> = HashMap::new();
        addr_map.insert(me, "0x1111111111111111111111111111111111111111".into());
        addr_map.insert(peer, "0x2222222222222222222222222222222222222222".into());

        let dot = render_dot_with_labels(&graph, label_from_map(&addr_map));

        assert!(dot.contains("lat=120ms"), "latency should be preserved: {dot}");
        assert!(dot.contains("cap=500"), "capacity should be preserved: {dot}");
        assert!(dot.contains("score="), "score should be preserved: {dot}");
        assert!(
            dot.contains(
                "\"0x1111111111111111111111111111111111111111\" -> \"0x2222222222222222222222222222222222222222\""
            ),
            "edge should use mapped addresses: {dot}"
        );
    }

    #[test]
    fn render_dot_should_be_unchanged_when_identity_label() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let peer = pubkey(&SECRET_1);
        let graph = ChannelGraph::new(me);
        graph.add_node(peer);
        graph.add_edge(&me, &peer)?;

        let dot_original = render_dot(&graph);
        let dot_identity = render_dot_with_labels(&graph, |key| format!("{key}"));

        assert_eq!(
            dot_original, dot_identity,
            "identity label_fn should produce identical output"
        );
        Ok(())
    }

    #[test]
    fn reachable_should_exclude_disconnected_subgraph() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let c = pubkey(&SECRET_3);
        let graph = ChannelGraph::new(me);
        for n in [a, b, c] {
            graph.add_node(n);
        }

        // me -> a (reachable)
        graph.add_edge(&me, &a)?;
        // b -> c (disconnected from me)
        graph.add_edge(&b, &c)?;

        let all_dot = render_dot(&graph);
        let reachable_dot = render_dot_reachable_with_labels(&graph, |key| format!("{key}"));

        // All edges should appear in the full graph
        assert_eq!(
            all_dot.matches("->").count(),
            2,
            "full graph should have 2 edges: {all_dot}"
        );

        // Only me -> a should appear in the reachable graph
        assert_eq!(
            reachable_dot.matches("->").count(),
            1,
            "reachable graph should have 1 edge: {reachable_dot}"
        );
        assert!(
            reachable_dot.contains(&format!("{a}")),
            "reachable peer 'a' should be present: {reachable_dot}"
        );
        assert!(
            !reachable_dot.contains(&format!("{b}")),
            "unreachable peer 'b' should be absent: {reachable_dot}"
        );
        assert!(
            !reachable_dot.contains(&format!("{c}")),
            "unreachable peer 'c' should be absent: {reachable_dot}"
        );
        Ok(())
    }

    #[test]
    fn reachable_should_include_transitive_peers() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        for n in [a, b] {
            graph.add_node(n);
        }

        // me -> a -> b (b reachable transitively)
        graph.add_edge(&me, &a)?;
        graph.add_edge(&a, &b)?;

        let reachable_dot = render_dot_reachable_with_labels(&graph, |key| format!("{key}"));

        assert_eq!(
            reachable_dot.matches("->").count(),
            2,
            "both edges should be reachable: {reachable_dot}"
        );
        assert!(
            reachable_dot.contains(&format!("{b}")),
            "transitively reachable peer should be present: {reachable_dot}"
        );
        Ok(())
    }

    #[test]
    fn reachable_should_be_empty_when_node_has_no_outgoing_edges() -> anyhow::Result<()> {
        let me = pubkey(&SECRET_0);
        let a = pubkey(&SECRET_1);
        let b = pubkey(&SECRET_2);
        let graph = ChannelGraph::new(me);
        for n in [a, b] {
            graph.add_node(n);
        }

        // a -> b (me has no outgoing edges)
        graph.add_edge(&a, &b)?;

        let reachable_dot = render_dot_reachable_with_labels(&graph, |key| format!("{key}"));

        assert_eq!(
            reachable_dot.matches("->").count(),
            0,
            "no edges should be reachable from isolated 'me': {reachable_dot}"
        );
        Ok(())
    }
}
