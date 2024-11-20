use anyhow::Result;
use vergen_gix::{Emitter, GixBuilder};

pub fn main() -> Result<()> {
    // Adds a short SHA hash of the commit as `VERGEN_GIT_SHA` env variable
    if let Err(_) = std::env::var("VERGEN_GIT_SHA") {
        let git = GixBuilder::default().sha(true).build()?;
        Emitter::default().add_instructions(&git)?.emit()
    } else {
        Ok(())
    }
}
