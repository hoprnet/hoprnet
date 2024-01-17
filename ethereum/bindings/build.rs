use std::process::Command;
use std::env;
use std::path::Path;


fn main() {
    let contracts_package_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("contracts");

    Command::new("make")
        .args(&["generate-bindings"])
        .current_dir(contracts_package_path.clone())
        .status()
        .unwrap();

    println!("cargo:rerun-if-changed={}", contracts_package_path.to_str().unwrap());
}