use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

// This build script is copied from
// [anyhow](https://github.com/dtolnay/anyhow/blob/master/build.rs),
// and is a type of feature detection to determine if the current rust
// toolchain has backtraces available.
//
// It exercises the surface area that we expect of the std Backtrace
// type. If the current toolchain is able to compile it, we enable a
// backtrace compiler configuration flag in http-types. We then
// conditionally require the compiler feature in src/lib.rs with
// `#![cfg_attr(backtrace, feature(backtrace))]`
// and gate our backtrace code behind `#[cfg(backtrace)]`

const PROBE: &str = r#"
    #![feature(backtrace)]
    #![allow(dead_code)]

    use std::backtrace::{Backtrace, BacktraceStatus};
    use std::error::Error;
    use std::fmt::{self, Display};

    #[derive(Debug)]
    struct E;

    impl Display for E {
        fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }

    impl Error for E {
        fn backtrace(&self) -> Option<&Backtrace> {
            let backtrace = Backtrace::capture();
            match backtrace.status() {
                BacktraceStatus::Captured | BacktraceStatus::Disabled | _ => {}
            }
            unimplemented!()
        }
    }
"#;

fn main() {
    match compile_probe() {
        Some(status) if status.success() => println!("cargo:rustc-cfg=backtrace"),
        _ => {}
    }
}

fn compile_probe() -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR")?;
    let probefile = Path::new(&out_dir).join("probe.rs");
    fs::write(&probefile, PROBE).ok()?;
    Command::new(rustc)
        .stderr(Stdio::null())
        .arg("--edition=2018")
        .arg("--crate-name=http_types_build")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile)
        .status()
        .ok()
}
