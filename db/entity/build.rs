//! Creates a build specification for the ORM codegen.

use std::{
    env::{self, temp_dir},
    path::Path,
};

use anyhow::Context;
use clap::Parser;
use migration::MigrationTrait;
// use sea_orm::sqlx::migrate::Migration;

async fn execute_sea_orm_cli_command<I, T>(itr: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    use sea_orm::{ConnectOptions, Database};
    use sea_orm_cli::*;

    let cli = sea_orm_cli::Cli::try_parse_from(itr).context("should be able to parse a sea-orm-cli command")?;

    match cli.command {
        Commands::Generate { command } => run_generate_command(command, true)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string())),
        Commands::Migrate {
            database_schema,
            database_url,
            command,
            ..
        } => {
            let connect_options: ConnectOptions =
                ConnectOptions::new(database_url.unwrap_or("/tmp/sea_orm_cli.db".into()))
                    .set_schema_search_path(database_schema.unwrap_or_else(|| "public".to_owned()))
                    .to_owned();
            let is_sqlite = connect_options.get_url().starts_with("sqlite");
            let db = &Database::connect(connect_options).await?;

            if is_sqlite {
                struct TempMigrator;

                impl migration::MigratorTrait for TempMigrator {
                    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
                        let mut migrations = migration::MigratorIndex::migrations();
                        migrations.extend(migration::MigratorPeers::migrations());
                        migrations.extend(migration::MigratorTickets::migrations());
                        migrations.extend(migration::MigratorChainLogs::migrations());

                        migrations
                    }
                }

                sea_orm_migration::cli::run_migrate(TempMigrator {}, db, command, cli.verbose)
                    .await
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            } else {
                sea_orm_migration::cli::run_migrate(migration::Migrator {}, db, command, cli.verbose)
                    .await
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cargo_manifest_dir = &env::var("CARGO_MANIFEST_DIR").context("should point to a valid manifest dir")?;
    let db_migration_package_path = Path::new(&cargo_manifest_dir)
        .parent()
        .context("should have a parent dir")?
        .join("migration");

    println!(
        "cargo:rerun-if-changed={}",
        db_migration_package_path.join("src").to_string_lossy()
    );
    println!(
        "cargo:rerun-if-changed={}",
        db_migration_package_path.join("Cargo.toml").to_str().unwrap()
    );

    let codegen_path = Path::new(&cargo_manifest_dir)
        .join("src/codegen/sqlite")
        .into_os_string()
        .into_string()
        .map_err(|e| anyhow::anyhow!(e.to_str().unwrap_or("illegible error").to_string()))?;

    let tmp_db = temp_dir().join("tmp_migration.db");

    let _ = std::fs::remove_file(
        tmp_db
            .clone()
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow::anyhow!(e.to_str().unwrap_or("illegible error").to_string()))?
            .as_str(),
    );

    tokio::runtime::Runtime::new()?.block_on(execute_sea_orm_cli_command([
        "sea-orm-cli",
        "migrate",
        "refresh",
        "-u",
        format!(
            "sqlite://{}?mode=rwc",
            tmp_db
                .clone()
                .into_os_string()
                .into_string()
                .map_err(|e| anyhow::anyhow!(e.to_str().unwrap_or("illegible error").to_string()))?
        )
        .as_str(),
        "-d",
        db_migration_package_path
            .clone()
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow::anyhow!(e.to_str().unwrap_or("illegible error").to_string()))?
            .as_str(),
    ]))?;

    tokio::runtime::Runtime::new()?.block_on(execute_sea_orm_cli_command([
        "sea-orm-cli",
        "generate",
        "entity",
        "-o",
        &codegen_path,
        "-u",
        format!(
            "sqlite://{}?mode=rwc",
            tmp_db
                .into_os_string()
                .into_string()
                .map_err(|e| anyhow::anyhow!(e.to_str().unwrap_or("illegible error").to_string()))?
        )
        .as_str(),
    ]))?;

    Ok(())
}
