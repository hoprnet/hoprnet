pub use sea_orm_migration::prelude::*;

mod m20240226_000001_create_channel;
mod m20240226_000002_create_ticket;
mod m20240226_000003_create_account;
mod m20240226_000004_create_network_registry;
mod m20240226_000005_create_node_info;
mod m20240226_000006_create_peer_store;
mod m20240226_000007_create_settings;
mod m20240226_000008_create_stats;
mod m20240301_000010_initial_seed;
mod m20240301_000009_create_network_eligiblity;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240226_000001_create_channel::Migration),
            Box::new(m20240226_000002_create_ticket::Migration),
            Box::new(m20240226_000003_create_account::Migration),
            Box::new(m20240226_000004_create_network_registry::Migration),
            Box::new(m20240226_000005_create_node_info::Migration),
            Box::new(m20240226_000006_create_peer_store::Migration),
            Box::new(m20240226_000007_create_settings::Migration),
            Box::new(m20240226_000008_create_stats::Migration),
            Box::new(m20240301_000010_initial_seed::Migration),
            Box::new(m20240301_000009_create_network_eligiblity::Migration),
        ]
    }
}
