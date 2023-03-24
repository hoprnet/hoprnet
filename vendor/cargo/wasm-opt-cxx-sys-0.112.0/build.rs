fn main() -> anyhow::Result<()> {
    let mut builder = cxx_build::bridge("src/lib.rs");

    {
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV")?;

        let flags: &[_] = if target_env != "msvc" {
            &["-std=c++17", "-Wno-unused-parameter"]
        } else {
            &["/std:c++17"]
        };

        for flag in flags {
            builder.flag(flag);
        }
    }

    builder.include("src").compile("wasm-opt-cxx");

    Ok(())
}
