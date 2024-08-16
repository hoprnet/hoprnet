// SPDX-License-Identifier: Apache-2.0 OR MIT

// The rustc-cfg emitted by the build script are *not* public API.

use std::{env, fs, iter, path::PathBuf, process::Command, str};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let version = match rustc_version() {
        Ok(version) => version.print(),
        Err(e) => {
            if env::var_os("CONST_FN_DENY_WARNINGS").unwrap_or_default() == "1" {
                panic!("unable to determine rustc version")
            }
            println!(
                "cargo:warning={}: unable to determine rustc version: {}",
                env!("CARGO_PKG_NAME"),
                e
            );
            return;
        }
    };

    let out_dir: PathBuf = env::var_os("OUT_DIR").expect("OUT_DIR not set").into();
    let out_file = &out_dir.join("version");
    fs::write(out_file, version)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", out_file.display(), e));

    if assume_incomplete_release() {
        println!("cargo:rustc-cfg=const_fn_assume_incomplete_release");
    }

    // Mark as build script has been run successfully.
    println!("cargo:rustc-cfg=const_fn_has_build_script");
}

fn rustc_version() -> Result<Version, String> {
    let rustc = env::var_os("RUSTC").ok_or("RUSTC not set")?;
    let rustc_wrapper = if env::var_os("CARGO_ENCODED_RUSTFLAGS").is_some() {
        env::var_os("RUSTC_WRAPPER").filter(|v| !v.is_empty())
    } else {
        // Cargo sets environment variables for wrappers correctly only since https://github.com/rust-lang/cargo/pull/9601.
        None
    };
    // Do not apply RUSTC_WORKSPACE_WRAPPER: https://github.com/cuviper/autocfg/issues/58#issuecomment-2067625980
    let mut rustc = rustc_wrapper.into_iter().chain(iter::once(rustc));
    let mut cmd = Command::new(rustc.next().unwrap());
    cmd.args(rustc);
    // Use verbose version output because the packagers add extra strings to the normal version output.
    // Do not use long flags (--version --verbose) because clippy-deriver doesn't handle them properly.
    // -vV is also matched with that cargo internally uses: https://github.com/rust-lang/cargo/blob/14b46ecc62aa671d7477beba237ad9c6a209cf5d/src/cargo/util/rustc.rs#L65
    let output =
        cmd.arg("-vV").output().map_err(|e| format!("could not execute {:?}: {}", cmd, e))?;
    let verbose_version = str::from_utf8(&output.stdout)
        .map_err(|e| format!("failed to parse output of {:?}: {}", cmd, e))?;
    Version::parse(verbose_version)
        .ok_or_else(|| format!("unexpected output from {:?}: {}", cmd, verbose_version))
}

struct Version {
    minor: u32,
    nightly: bool,
}

impl Version {
    fn parse(verbose_version: &str) -> Option<Self> {
        let mut release = verbose_version
            .lines()
            .find(|line| line.starts_with("release: "))
            .map(|line| &line["release: ".len()..])?
            .splitn(2, '-');
        let version = release.next().unwrap();
        let channel = release.next().unwrap_or_default();
        let mut digits = version.splitn(3, '.');
        let major = digits.next()?;
        if major != "1" {
            return None;
        }
        let minor = digits.next()?.parse::<u32>().ok()?;
        let _patch = digits.next().unwrap_or("0").parse::<u32>().ok()?;
        let nightly = channel == "nightly" || channel == "dev";

        Some(Self { minor, nightly })
    }

    fn print(&self) -> String {
        format!("Version {{ minor: {}, nightly: {} }}\n", self.minor, self.nightly)
    }
}

// https://github.com/taiki-e/const_fn/issues/27
// https://github.com/rust-lang/rust/pull/81468
fn assume_incomplete_release() -> bool {
    // Recognized formats: -Z( )?assume-incomplete-release

    // https://github.com/rust-lang/cargo/issues/10111
    if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split('\x1f') {
            if flag.starts_with("-Z") {
                flag = &flag["-Z".len()..];
            }
            if flag == "assume-incomplete-release" {
                return true;
            }
        }
    }

    false
}
