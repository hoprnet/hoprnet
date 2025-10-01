use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("protocol-config.json");

    // Copy the protocol-config.json from the chain-config crate
    let config_path = Path::new("../../hopr/chain-config/data/protocol-config.json");

    if config_path.exists() {
        fs::copy(config_path, &dest_path).expect("Failed to copy protocol-config.json to OUT_DIR");
    } else {
        // Fallback: create a minimal config if the file doesn't exist
        // This handles the case where the relative path structure is different
        println!("cargo:warning=protocol-config.json not found at expected path, creating minimal config");

        let minimal_config = r#"{
            "networks": {},
            "chains": {}
        }"#;

        fs::write(&dest_path, minimal_config).expect("Failed to write minimal protocol-config.json");
    }

    // Tell Cargo to rerun this build script if the config file changes
    println!("cargo:rerun-if-changed=../../hopr/chain-config/data/protocol-config.json");
}
