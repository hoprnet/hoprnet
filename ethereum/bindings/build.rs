//! Creates a build specification for the smart contract codegen.

use anyhow::Context;
use glob::glob;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let codegen_dir = out_dir + "/codegen";

    let cargo_manifest_dir = &env::var("CARGO_MANIFEST_DIR")?;

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

    assert!(std::fs::metadata(&contracts_package_path)
        .context(format!("{contracts_package_path:?} must be a path"))?
        .is_dir());

    assert!(Command::new("forge").args(["--version"]).status()?.success());

    assert!(Command::new("forge")
        .args([
            "bind",
            "--offline", // ensure we are not installing any missing solc at this point
            "--bindings-path",
            codegen_dir.as_str(),
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

    //let mods_filename = Path::new(&out_dir.as_str()).join("mods.rs");

    //let mut mods = File::create(mods_filename).unwrap();
    //let pattern = out_dir.clone() + "/hopr*.rs";
    //for entry in glob(&pattern).unwrap() {
    //    writeln!(
    //        &mut mods,
    //        r#"  include!(concat!(env!("OUT_DIR"), "/{}"));"#,
    //        entry.unwrap().file_name().unwrap().to_str().unwrap()
    //    )
    //    .unwrap();
    //}

    Ok(())
}
