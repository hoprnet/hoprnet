mod db;
pub mod errors;
mod tickets;

use std::path::PathBuf;

pub use db::{HoprNodeDb, HoprNodeDbConfig};
pub use hopr_api::{chain::RedeemableTicket, db::*};

/// Convenience function to initialize the HOPR node database.
pub async fn init_hopr_node_db(
    db_data_path: &str,
    initialize: bool,
    force_initialize: bool,
) -> anyhow::Result<HoprNodeDb> {
    let db_path: PathBuf = [db_data_path, "node_db"].iter().collect();
    tracing::info!(path = ?db_path, "initiating DB");

    let mut create_if_missing = initialize;

    if force_initialize {
        tracing::info!("Force cleaning up existing database");
        std::fs::remove_dir_all(db_path.as_path())?;
        create_if_missing = true;
    }

    // create DB dir if it does not exist
    if let Some(parent_dir_path) = db_path.as_path().parent()
        && !parent_dir_path.is_dir() {
            std::fs::create_dir_all(parent_dir_path)
                .map_err(|e| anyhow::anyhow!("Failed to create DB parent directory at '{parent_dir_path:?}': {e}"))?
        }

    let db_cfg = HoprNodeDbConfig {
        create_if_missing,
        force_create: force_initialize,
        log_slow_queries: std::time::Duration::from_millis(150),
    };
    let node_db = HoprNodeDb::new(db_path.as_path(), db_cfg).await?;

    Ok(node_db)
}
