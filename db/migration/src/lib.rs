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
        vec![Box::new(m20240930_000017_logs_create_log::Migration)]
    }
}
