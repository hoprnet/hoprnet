//! Creates a build specification for the ORM codegen.

use std::env::{self, temp_dir};
use std::path::Path;

use clap::Parser;

async fn execute_sea_orm_cli_command<I, T>(itr: I)
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    use sea_orm::{ConnectOptions, Database};
    use sea_orm_cli::*;

    let cli = sea_orm_cli::Cli::try_parse_from(itr).expect("should be able to parse a sea-orm-cli command");

    match cli.command {
        Commands::Generate { command } => {
            run_generate_command(command, true).await.unwrap();
        }
        Commands::Migrate {
            database_schema,
            database_url,
            command,
            ..
        } => {
            let connect_options = ConnectOptions::new(database_url.unwrap())
                .set_schema_search_path(database_schema.unwrap_or_else(|| "public".to_owned()))
                .to_owned();
            let db = &Database::connect(connect_options)
                .await
                .expect("Fail to acquire database connection");

            sea_orm_migration::cli::run_migrate(hoprd_migration::Migrator {}, db, command, cli.verbose)
                .await
                .unwrap_or_else(handle_error);
        }
    }
}

#[tokio::main]
async fn main() {
    let cargo_manifest_dir = &env::var("CARGO_MANIFEST_DIR").expect("Points to a valid manifest dir");
    let db_migration_package_path = Path::new(&cargo_manifest_dir).parent().unwrap().join("migration");

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
        .expect("should contain valid temporary db path");

    let tmp_db = temp_dir().join("tmp_migration.db");

    let _ = std::fs::remove_file(&tmp_db);

    execute_sea_orm_cli_command([
        "sea-orm-cli",
        "migrate",
        "refresh",
        "-u",
        format!(
            "sqlite://{}?mode=rwc",
            tmp_db.to_str().expect("should contain valid temporary db path")
        )
        .as_str(),
        "-d",
        db_migration_package_path.to_string_lossy().as_ref(),
    ])
    .await;

    execute_sea_orm_cli_command([
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
                .expect("should contain valid temporary db path")
        )
        .as_str(),
    ])
    .await;
}
