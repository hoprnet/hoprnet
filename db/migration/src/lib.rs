pub use sea_orm_migration::prelude::*;

mod m20240226_000001_index_create_channel;
mod m20240226_000002_index_create_account;
mod m20240226_000003_index_create_network_registry;
mod m20240226_000004_index_create_node_info;
mod m20240226_000005_index_create_chain_info;
mod m20240226_000006_index_create_network_eligibility;
mod m20240226_000007_index_initial_seed;
mod m20240226_000008_node_create_settings;
mod m20240226_000009_peers_create_peer_store;
mod m20240301_000010_tickets_create_ticket;
mod m20240301_000011_tickets_create_ticket_stats;
mod m20240301_000012_tickets_create_outgoing_ticket_index;
mod m20240404_000013_tickets_recreate_ticket;
mod m20240810_000014_index_extend_chain_info_with_pre_checksum_block;
mod m20240917_000015_add_minimum_incoming_ticket_win_prob_column;
mod m20240926_000016_peers_create_peer_store_with_new_sea_orm;
mod m20240930_000017_logs_create_log;
mod m20241112_000018_logs_add_index;
mod m20250107_000019_logs_meta_table;
mod m20250219_000020_logs_add_index;
mod m20250219_000021_channels_add_index;
mod m20250419_000022_account_add_published_block;
mod m20250528_000023_peers_reset;
mod m20250603_000024_index_reset;
mod m20250603_000025_peers_reset;
mod m20250603_000026_logs_reset;
mod m20250604_000027_index_initial_seed;
mod m20250701_000028_peers_deprecate_fields;
mod m20250709_000029_channels_add_corrupted_state;
mod m20250808_000030_index_create_corrupted_channel;

#[derive(PartialEq)]
pub enum BackendType {
    SQLite,
    Postgres,
}

pub struct Migrator;

/// Used to instantiate all tables to generate the corresponding entities in
/// a non-SQLite database (such as Postgres).
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240226_000001_index_create_channel::Migration),
            Box::new(m20240226_000002_index_create_account::Migration),
            Box::new(m20240226_000003_index_create_network_registry::Migration),
            Box::new(m20240226_000004_index_create_node_info::Migration),
            Box::new(m20240226_000005_index_create_chain_info::Migration),
            Box::new(m20240226_000006_index_create_network_eligibility::Migration),
            Box::new(m20240226_000007_index_initial_seed::Migration),
            Box::new(m20240226_000008_node_create_settings::Migration),
            Box::new(m20240226_000009_peers_create_peer_store::Migration),
            Box::new(m20240301_000010_tickets_create_ticket::Migration(BackendType::Postgres)),
            Box::new(m20240301_000011_tickets_create_ticket_stats::Migration(
                BackendType::Postgres,
            )),
            Box::new(m20240301_000012_tickets_create_outgoing_ticket_index::Migration(
                BackendType::Postgres,
            )),
            Box::new(m20240404_000013_tickets_recreate_ticket::Migration(
                BackendType::Postgres,
            )),
            Box::new(m20240810_000014_index_extend_chain_info_with_pre_checksum_block::Migration),
            Box::new(m20240917_000015_add_minimum_incoming_ticket_win_prob_column::Migration),
            Box::new(m20240926_000016_peers_create_peer_store_with_new_sea_orm::Migration(
                BackendType::Postgres,
            )),
            Box::new(m20240930_000017_logs_create_log::Migration),
            Box::new(m20241112_000018_logs_add_index::Migration),
            Box::new(m20250107_000019_logs_meta_table::Migration),
            Box::new(m20250219_000020_logs_add_index::Migration),
            Box::new(m20250219_000021_channels_add_index::Migration),
            Box::new(m20250419_000022_account_add_published_block::Migration),
            Box::new(m20250528_000023_peers_reset::Migration),
            Box::new(m20250603_000024_index_reset::Migration),
            Box::new(m20250603_000025_peers_reset::Migration),
            Box::new(m20250603_000026_logs_reset::Migration),
            Box::new(m20250604_000027_index_initial_seed::Migration),
            Box::new(m20250709_000029_channels_add_corrupted_state::Migration),
            Box::new(m20250808_000030_index_create_corrupted_channel::Migration),
        ]
    }
}

/// SQLite does not allow writing lock tables only, and the write lock
/// will apply to the entire database file. It is therefore beneficial
/// to separate the exclusive concurrently accessing components into
/// separate database files to benefit from multiple write locks over
/// different parts of the database.
pub struct MigratorIndex;

#[async_trait::async_trait]
impl MigratorTrait for MigratorIndex {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240226_000001_index_create_channel::Migration),
            Box::new(m20240226_000002_index_create_account::Migration),
            Box::new(m20240226_000003_index_create_network_registry::Migration),
            Box::new(m20240226_000004_index_create_node_info::Migration),
            Box::new(m20240226_000005_index_create_chain_info::Migration),
            Box::new(m20240226_000006_index_create_network_eligibility::Migration),
            Box::new(m20240226_000008_node_create_settings::Migration),
            Box::new(m20240226_000007_index_initial_seed::Migration),
            Box::new(m20240810_000014_index_extend_chain_info_with_pre_checksum_block::Migration),
            Box::new(m20240917_000015_add_minimum_incoming_ticket_win_prob_column::Migration),
            Box::new(m20250219_000021_channels_add_index::Migration),
            Box::new(m20250419_000022_account_add_published_block::Migration),
            Box::new(m20250603_000024_index_reset::Migration),
            Box::new(m20250604_000027_index_initial_seed::Migration),
            Box::new(m20250709_000029_channels_add_corrupted_state::Migration),
            Box::new(m20250808_000030_index_create_corrupted_channel::Migration),
        ]
    }
}

pub struct MigratorPeers;

#[async_trait::async_trait]
impl MigratorTrait for MigratorPeers {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240226_000009_peers_create_peer_store::Migration),
            Box::new(m20240926_000016_peers_create_peer_store_with_new_sea_orm::Migration(
                BackendType::SQLite,
            )),
            Box::new(m20250528_000023_peers_reset::Migration),
            Box::new(m20250603_000025_peers_reset::Migration),
            Box::new(m20250701_000028_peers_deprecate_fields::Migration(BackendType::SQLite)),
        ]
    }
}

pub struct MigratorTickets;

#[async_trait::async_trait]
impl MigratorTrait for MigratorTickets {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240301_000010_tickets_create_ticket::Migration(BackendType::SQLite)),
            Box::new(m20240301_000011_tickets_create_ticket_stats::Migration(
                BackendType::SQLite,
            )),
            Box::new(m20240301_000012_tickets_create_outgoing_ticket_index::Migration(
                BackendType::SQLite,
            )),
            Box::new(m20240404_000013_tickets_recreate_ticket::Migration(BackendType::SQLite)),
        ]
    }
}

/// The logs are kept separate from the rest of the database to allow for
/// easier export of the logs themselves and also to not block any other database operations
/// made by the node at runtime.
pub struct MigratorChainLogs;

#[async_trait::async_trait]
impl MigratorTrait for MigratorChainLogs {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240930_000017_logs_create_log::Migration),
            Box::new(m20241112_000018_logs_add_index::Migration),
            Box::new(m20250107_000019_logs_meta_table::Migration),
            Box::new(m20250219_000020_logs_add_index::Migration),
            Box::new(m20250603_000026_logs_reset::Migration),
        ]
    }
}
