//! Creates a build specification for the smart contract codegen.

use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let cargo_manifest_dir = &env::var("CARGO_MANIFEST_DIR")?;
    let bindings_codegen_path = Path::new(&cargo_manifest_dir).join("src/codegen");
    std::fs::create_dir_all(bindings_codegen_path.clone())?;

    let contracts_package_path = Path::new(&cargo_manifest_dir)
        .parent()
        .context("must have a parent")?
        .join("contracts");
    let vendor_path = Path::new(&cargo_manifest_dir)
        .parent()
        .context("path should be present")?
        .parent()
        .context("parent path should exist")?
        .join("vendor/solidity");

    println!(
        "cargo:rerun-if-changed={}",
        contracts_package_path
            .join("src")
            .to_str()
            .context("must be convertible to string")?
    );
    println!(
        "cargo:rerun-if-changed={}",
        contracts_package_path
            .join("remappings.txt")
            .to_str()
            .context("must be convertible to string")?
    );
    println!(
        "cargo:rerun-if-changed={}",
        contracts_package_path
            .join("foundry.toml")
            .to_str()
            .context("must be convertible to string")?
    );
    println!(
        "cargo:rerun-if-changed={}",
        vendor_path.to_str().context("must be convertible to string")?
    );

    assert!(std::fs::metadata(&bindings_codegen_path)
        .context(format!("{bindings_codegen_path:?} must be a path"))?
        .is_dir());

    assert!(std::fs::metadata(&contracts_package_path)
        .context(format!("{contracts_package_path:?} must be a path"))?
        .is_dir());

    assert!(Command::new("forge").args(["--version"]).status()?.success());

    let bindings_codegen_path_str = bindings_codegen_path
        .to_str()
        .context("must be convertible to string")?;
    assert!(Command::new("forge")
        .args([
            "bind",
            "--offline", // ensure we are not installing any missing solc at this point
            "--bindings-path",
            bindings_codegen_path_str,
            "--module",
            "--skip-extra-derives",
            "--alloy",
            "--overwrite",
            "--select",
            "^(HoprAnnouncements|HoprAnnouncementsEvents|HoprCapabilityPermissions|HoprChannels|HoprChannelsEvents|HoprCrypto|HoprDummyProxyForNetworkRegistry|HoprBoost|HoprToken|HoprLedger|HoprLedgerevents|HoprMultisig|HoprNetworkRegistry|HoprNetworkRegistryEvents|HoprNodeManagementModule|HoprNodeSafeRegistry|HoprNodeSafeRegistryEvents|HoprNodeStakeFactory|HoprNodeStakeFactoryEvents|HoprSafeProxyForNetworkRegistry|HoprStakingProxyForNetworkRegistry|HoprTicketPriceOracle|HoprTicketPriceOracleEvents|HoprWinningProbabilityOracle|HoprWinningProbabilityOracleEvents)$"
        ])
        .current_dir(contracts_package_path.clone())
        .status()?
        .success());

    Ok(())
}
