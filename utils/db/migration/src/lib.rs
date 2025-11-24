pub use sea_orm_migration::prelude::*;

mod m20240226_000009_peers_create_peer_store;
mod m20240301_000010_tickets_create_ticket;
mod m20240301_000011_tickets_create_ticket_stats;
mod m20240301_000012_tickets_create_outgoing_ticket_index;
mod m20240404_000013_tickets_recreate_ticket;
mod m20240926_000016_peers_create_peer_store_with_new_sea_orm;
mod m20250528_000023_peers_reset;
mod m20250603_000025_peers_reset;
mod m20250701_000028_peers_deprecate_fields;
mod m20250909_000031_peer_store_add_indices;

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
            Box::new(m20240926_000016_peers_create_peer_store_with_new_sea_orm::Migration(
                BackendType::Postgres,
            )),
            Box::new(m20250528_000023_peers_reset::Migration),
            Box::new(m20250603_000025_peers_reset::Migration),
            Box::new(m20250909_000031_peer_store_add_indices::Migration),
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
            Box::new(m20250909_000031_peer_store_add_indices::Migration),
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
