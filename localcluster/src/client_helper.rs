use std::process::Child;

use anyhow::{Context, Result};
use futures::future::try_join_all;
use hopr_lib::HoprBalance;
use hoprd_rest_api_client as hoprd_api_client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

#[derive(Debug, Clone)]
pub struct HoprdApiClient {
    inner: hoprd_api_client::Client,
}

impl HoprdApiClient {
    pub fn new(base_url: String, token: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(token) = token {
            let value = format!("Bearer {token}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&value).context("invalid api token")?,
            );
        }

        let http_client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(10))
            .default_headers(headers)
            .build()
            .context("failed to build http client")?;

        Ok(Self {
            inner: hoprd_api_client::Client::new_with_client(base_url.as_ref(), http_client),
        })
    }

    pub async fn wait_started(&self, timeout: std::time::Duration) -> Result<()> {
        self.wait_status("/startedz", timeout).await
    }

    pub async fn wait_ready(&self, timeout: std::time::Duration) -> Result<()> {
        self.wait_status("/readyz", timeout).await
    }

    async fn wait_status(&self, path: &str, timeout: std::time::Duration) -> Result<()> {
        let start = std::time::Instant::now();
        loop {
            let ready = match path {
                "/startedz" => self.inner.startedz().await,
                "/readyz" => self.inner.readyz().await,
                _ => anyhow::bail!("unknown status path: {path}"),
            };
            if ready.is_ok() {
                return Ok(());
            }

            if start.elapsed() > timeout {
                anyhow::bail!("timeout while waiting for {}", path);
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    pub async fn addresses(&self) -> Result<String> {
        let response = self.inner.addresses().await?;
        Ok(response.into_inner().native)
    }

    pub async fn open_channel(&self, destination: &str, amount: &str) -> Result<()> {
        let req = hoprd_api_client::types::OpenChannelBodyRequest {
            amount: amount.to_string(),
            destination: destination.to_string(),
        };
        let _ = self.inner.open_channel(&req).await?;
        Ok(())
    }
}

pub struct NodeProcess {
    pub id: usize,
    pub api_port: u16,
    pub p2p_port: u16,
    pub api: HoprdApiClient,
    pub child: Child,
    pub address: Option<String>,
}

pub async fn open_full_mesh_channels(nodes: &[NodeProcess], amount: &HoprBalance) -> Result<()> {
    let amount = amount.to_string();
    let mut tasks = Vec::new();
    for src in nodes {
        let Some(src_addr) = src.address.clone() else {
            anyhow::bail!("node {} address missing", src.id);
        };
        for dst in nodes {
            let Some(dst_addr) = dst.address.clone() else {
                anyhow::bail!("node {} address missing", dst.id);
            };
            if src_addr == dst_addr {
                continue;
            }
            let api = src.api.clone();
            let amount = amount.clone();
            tasks.push(async move { api.open_channel(&dst_addr, &amount).await });
        }
    }

    try_join_all(tasks).await.context("failed to open channels")?;
    Ok(())
}
