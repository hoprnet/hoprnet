pub use sea_orm_migration::prelude::*;

mod m20240226_000001_create_channel;
mod m20240226_000002_create_ticket;
mod m20240226_000003_create_account;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240226_000001_create_channel::Migration),
            Box::new(m20240226_000002_create_ticket::Migration),
            Box::new(m20240226_000003_create_account::Migration),
        ]
    }
}
