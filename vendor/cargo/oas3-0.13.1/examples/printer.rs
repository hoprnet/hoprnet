//! Demonstrates reading an OpenAPI spec file and printing back to stdout.

use std::env;

fn main() -> eyre::Result<()> {
    if let Some(path) = env::args().nth(1) {
        let spec = oas3::from_path(path)?;
        println!("{}", oas3::to_yaml(&spec).unwrap());
    }

    Ok(())
}
