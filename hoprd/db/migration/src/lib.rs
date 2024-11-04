pub use sea_orm_migration::prelude::*;

mod m20240814_000001_metadata_create_db;

pub struct Migrator;

/// Used to instantiate all tables to generate the corresponding entities in
/// a non-SQLite database (such as Postgres).
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240814_000001_metadata_create_db::Migration)]
    }
}

/// SQLite does not allow writing lock tables only, and the write lock
/// will apply to the entire database file. It is therefore beneficial
/// to separate the exclusive concurrently accessing components into
/// separate database files to benefit from multiple write locks over
/// different parts of the database.
pub struct MigratorMetadata;

#[async_trait::async_trait]
impl MigratorTrait for MigratorMetadata {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240814_000001_metadata_create_db::Migration)]
    }
}
