use std::{env, fs, path::Path};

use anyhow::Context;

const CONFIG_FILE_PATH: &str = "../../hopr/chain-config/data/protocol-config.json";

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").context("OUT_DIR environment variable should be set")?;
    let dest_path = Path::new(&out_dir).join("protocol-config.json");

    let config_path = Path::new(CONFIG_FILE_PATH);

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "protocol-config.json not found at expected path: {:?}",
            config_path
        ));
    } else {
        fs::copy(config_path, &dest_path).context("Failed to copy protocol-config.json to OUT_DIR")?;
    }

    // Tell Cargo to rerun this build script if the config file changes
    println!("cargo:rerun-if-changed={CONFIG_FILE_PATH}");

    Ok(())
}
