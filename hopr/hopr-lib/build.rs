use std::{env, fs, path::PathBuf};

/// Generates a static slice from the manifest, with each feature gated by its compiler cfg.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=Cargo.toml");
    let manifest = fs::read_to_string("Cargo.toml")?.parse::<toml_edit::DocumentMut>()?;
    let features = manifest["features"].as_table().ok_or("missing feature table")?;
    let mut names: Vec<_> = features.iter().map(|(name, _)| name).collect();
    names.sort_unstable();

    let mut source = String::from("&[\n");
    for name in names {
        source.push_str(&format!("    #[cfg(feature = {name:?})] {name:?},\n"));
    }
    source.push_str("    #[cfg(debug_assertions)] \"debug-assertions\",\n]\n");
    fs::write(PathBuf::from(env::var("OUT_DIR")?).join("enabled_features.rs"), source)?;
    Ok(())
}
