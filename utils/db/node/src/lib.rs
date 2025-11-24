mod db;

mod peers;

mod protocol;

mod tickets;

mod cache;
pub mod errors;
mod ticket_manager;

pub use db::{HoprNodeDb, HoprNodeDbConfig};
pub use hopr_api::chain::AcknowledgedTicket;
pub use hopr_api::db::*;
use hopr_crypto_types::keypairs::ChainKeypair;

use std::path::PathBuf;

use futures::channel::mpsc::channel;

pub async fn init_db(
    chain_key: &ChainKeypair,
    db_data_path: &str,
    initialize: bool,
    force_initialize: bool,
) -> anyhow::Result<(HoprNodeDb, futures::channel::mpsc::Receiver<AcknowledgedTicket>)> {
    let db_path: PathBuf = [&db_data_path, "node_db"].iter().collect();
    tracing::info!(path = ?db_path, "initiating DB");

    let mut create_if_missing = initialize;

    if force_initialize {
        tracing::info!("Force cleaning up existing database");
        std::fs::remove_dir_all(db_path.as_path())?;
        create_if_missing = true;
    }

    // create DB dir if it does not exist
    if let Some(parent_dir_path) = db_path.as_path().parent() {
        if !parent_dir_path.is_dir() {
            std::fs::create_dir_all(parent_dir_path)
                .map_err(|e| anyhow::anyhow!("Failed to create DB parent directory at '{parent_dir_path:?}': {e}"))?
        }
    }

    let db_cfg = HoprNodeDbConfig {
        create_if_missing,
        force_create: force_initialize,
        log_slow_queries: std::time::Duration::from_millis(150),
        surb_ring_buffer_size: std::env::var("HOPR_PROTOCOL_SURB_RB_SIZE")
            .ok()
            .and_then(|s| s.parse::<u64>().map(|v| v as usize).ok())
            .unwrap_or_else(|| HoprNodeDbConfig::default().surb_ring_buffer_size),
        surb_distress_threshold: std::env::var("HOPR_PROTOCOL_SURB_RB_DISTRESS")
            .ok()
            .and_then(|s| s.parse::<u64>().map(|v| v as usize).ok())
            .unwrap_or_else(|| HoprNodeDbConfig::default().surb_distress_threshold),
    };
    let node_db = HoprNodeDb::new(db_path.as_path(), chain_key.clone(), db_cfg).await?;

    let ack_ticket_channel_capacity = std::env::var("HOPR_INTERNAL_ACKED_TICKET_CHANNEL_CAPACITY")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&c| c > 0)
        .unwrap_or(2048);

    tracing::debug!(
        capacity = ack_ticket_channel_capacity,
        "starting winning ticket processing"
    );
    let (on_ack_tkt_tx, on_ack_tkt_rx) = channel::<AcknowledgedTicket>(ack_ticket_channel_capacity);
    node_db.start_ticket_processing(Some(on_ack_tkt_tx))?;

    Ok((node_db, on_ack_tkt_rx))
}
