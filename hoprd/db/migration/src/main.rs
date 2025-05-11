use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(hoprd_migration::Migrator).await;
}
