//! Creates a build specification for the ORM codegen.

use std::{env, path::Path};

use anyhow::Context;
use clap::Parser;

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
            let db = &Database::connect(connect_options).await?;

            sea_orm_migration::cli::run_migrate(migration::Migrator {}, db, command, cli.verbose)
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))
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

    // Always generate to OUT_DIR for both Nix and regular builds
    let out_dir = env::var("OUT_DIR").context("OUT_DIR environment variable should be set")?;
    let codegen_path = Path::new(&out_dir).join("codegen").join("sqlite");

    // Ensure the codegen directory exists
    std::fs::create_dir_all(&codegen_path).context("Failed to create codegen directory in OUT_DIR")?;

    let codegen_path_str = codegen_path.to_string_lossy().to_string();

    let tmp_db = Path::new(&out_dir).join("tmp_migration.db");

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
        &codegen_path_str,
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

    // Generate a mod.rs file that re-exports all the generated entities
    let mod_rs_path = codegen_path.join("mod.rs");
    let mut mod_content = String::new();

    // Read the generated files and create module declarations
    if let Ok(entries) = std::fs::read_dir(&codegen_path) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str()
                && file_name.ends_with(".rs")
                && file_name != "mod.rs"
            {
                let module_name = file_name.trim_end_matches(".rs");
                mod_content.push_str(&format!("pub mod {};\n", module_name));
            }
        }
    }

    std::fs::write(&mod_rs_path, mod_content).context("Failed to write mod.rs file")?;

    Ok(())
}
