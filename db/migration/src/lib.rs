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
        ]
    }
}

/// SQLite does not allow writing lock tables only, and the write lock
/// will apple to the entire database file. It is therefore beneficial
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
        ]
    }
}

pub struct MigratorPeers;

#[async_trait::async_trait]
impl MigratorTrait for MigratorPeers {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240226_000009_peers_create_peer_store::Migration)]
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
        ]
    }
}
