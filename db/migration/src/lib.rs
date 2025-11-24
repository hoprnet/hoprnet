pub use sea_orm_migration::prelude::*;

mod m20251124_00001_tickets_create_ticket_stats;
mod m20251124_00002_tickets_create_outgoing_ticket_index;
mod m20251124_00003_tickets_create_ticket;
mod m20251124_00004_peers_create_peers;

pub struct Migrator;

/// Used to instantiate all tables to generate the corresponding entities in
/// a non-SQLite database (such as Postgres).
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut ret = MigratorTickets::migrations();
        ret.extend(MigratorPeers::migrations());
        ret
    }
}

pub struct MigratorPeers;

#[async_trait::async_trait]
impl MigratorTrait for MigratorPeers {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20251124_00004_peers_create_peers::Migration)]
    }
}

pub struct MigratorTickets;

#[async_trait::async_trait]
impl MigratorTrait for MigratorTickets {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251124_00001_tickets_create_ticket_stats::Migration),
            Box::new(m20251124_00002_tickets_create_outgoing_ticket_index::Migration),
            Box::new(m20251124_00003_tickets_create_ticket::Migration),
        ]
    }
}
