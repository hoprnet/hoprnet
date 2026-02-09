use anyhow::Result;
use futures::future::try_join_all;
use hopr_lib::{
    HoprBalance,
    testing::{fixtures::ClusterGuard, hopr::ChannelGuard},
};
use tokio::task::JoinHandle;

pub struct ApiHandle {
    pub host: String,
    pub port: u16,
    pub token: String,
    pub join: JoinHandle<()>,
}

impl ApiHandle {
    pub fn host_with_port(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub async fn open_full_mesh_channels(cluster: &ClusterGuard, funding: HoprBalance) -> Result<Vec<ChannelGuard>> {
    let mut futures = Vec::new();

    for (src_idx, src) in cluster.iter().enumerate() {
        for (dst_idx, dst) in cluster.iter().enumerate() {
            if src_idx == dst_idx {
                continue;
            }

            let src = src.instance.clone();
            let dst = dst.instance.clone();
            futures.push(async move { ChannelGuard::open_channel_between_nodes(src, dst, funding).await });
        }
    }

    try_join_all(futures).await
}

pub fn print_node_summary(cluster: &ClusterGuard, api_handles: &[ApiHandle]) {
    println!("\n\n");

    for (idx, node) in cluster.iter().enumerate() {
        let api = api_handles.get(idx);
        let api_host = api.map(|h| h.host_with_port()).unwrap_or_else(|| "N/A".to_string());
        let api_key = api.map(|h| h.token.clone()).unwrap_or_else(|| "N/A".to_string());
        let node_admin = format!(
            "http://localhost:4677/node/info?apiToken={}&apiEndpoint=http://{}",
            api_key, api_host
        );

        println!(
            "Node {}:\n\tAddress: {}\n\tPeer ID: {}\n\tAPI Host: {}\n\tAPI Key: {}\n\tNode admin: {}\n\n",
            idx,
            node.address(),
            node.peer_id(),
            api_host,
            api_key,
            node_admin
        );
    }
}
