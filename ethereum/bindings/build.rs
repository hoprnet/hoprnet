//! Creates a build specification for the smart contract codegen.

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let cargo_manifest_dir = &env::var("CARGO_MANIFEST_DIR").expect("Points to a valid manifest dir");

    let contracts_package_path = Path::new(&cargo_manifest_dir).parent().unwrap().join("contracts");

    println!("cargo:rerun-if-changed={}", contracts_package_path.to_str().unwrap());

    // Ideally this would run a `forge bind` command directly
    assert!(Command::new("make")
        .args(["generate-bindings"])
        .current_dir(contracts_package_path.clone())
        .status()
        .unwrap()
        .success());
}
