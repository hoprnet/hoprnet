//! DOT (Graphviz) rendering for the channel graph.
//!
//! Gated behind the `graph-api` feature.

use std::fmt::Write;

use hopr_api::graph::traits::{EdgeLinkObservable, EdgeObservableRead, EdgeProtocolObservable};

use crate::ChannelGraph;

/// Renders the connected subgraph of `graph` as a DOT (Graphviz) digraph.
///
/// Isolated nodes (those with no incoming or outgoing edges) are excluded.
/// Each node is labeled with `OffchainPublicKey` in hex.
/// Edges carry quality annotations: score, latency (ms), and capacity when available.
pub fn render_dot(graph: &ChannelGraph) -> String {
    let edges = graph.connected_edges();

    let mut out = String::from("digraph hopr {\n");

    for (src, dst, obs) in &edges {
        let src_label = format!("{src}");
        let dst_label = format!("{dst}");

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
    use hopr_api::{
        graph::{
            NetworkGraphWrite,
            traits::{EdgeObservableWrite, EdgeWeightType},
        },
        types::crypto::prelude::{Keypair, OffchainKeypair},
    };

    use super::*;
    use crate::ChannelGraph;

    const SECRET_0: [u8; 32] = hex_literal::hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d");
    const SECRET_1: [u8; 32] = hex_literal::hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a");
    const SECRET_2: [u8; 32] = hex_literal::hex!("c24bd833704dd2abdae3933fcc9962c2ac404f84132224c474147382d4db2299");
    const SECRET_3: [u8; 32] = hex_literal::hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e");

    fn pubkey(secret: &[u8; 32]) -> hopr_api::OffchainPublicKey {
        *OffchainKeypair::from_secret(secret).expect("valid secret").public()
    }

    #[test]
    fn empty_graph_renders_empty_digraph() {
        let me = pubkey(&SECRET_0);
        let graph = ChannelGraph::new(me);
        let dot = render_dot(&graph);
        assert_eq!(dot, "digraph hopr {\n}\n");
    }

    #[test]
    fn isolated_nodes_excluded_from_dot() {
        let me = pubkey(&SECRET_0);
        let graph = ChannelGraph::new(me);
        graph.add_node(pubkey(&SECRET_1));
        // No edges — only isolated nodes.
        let dot = render_dot(&graph);
        assert_eq!(dot, "digraph hopr {\n}\n");
    }

    #[test]
    fn diamond_topology_renders_four_edges() -> anyhow::Result<()> {
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
    fn observations_appear_as_edge_labels() {
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
}
