use std::{env, fs};

/// Exports the enabled manifest features for startup logging.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=Cargo.toml");
    let manifest = fs::read_to_string("Cargo.toml")?.parse::<toml_edit::DocumentMut>()?;
    let mut active_features: Vec<_> = manifest
        .get("features")
        .and_then(toml_edit::Item::as_table)
        .into_iter()
        .flat_map(|features| features.iter())
        .map(|(name, _)| name)
        .filter(|name| env::var_os(format!("CARGO_FEATURE_{}", name.to_uppercase().replace('-', "_"))).is_some())
        .collect();
    active_features.sort_unstable();
    if env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_some() {
        active_features.push("debug-assertions");
    }
    println!("cargo::rustc-env=ACTIVE_FEATURES={}", active_features.join(","));
    Ok(())
}
