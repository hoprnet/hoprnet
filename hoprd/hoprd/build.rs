use anyhow::Result;
use vergen_gix::{Emitter, GixBuilder};

pub fn main() -> Result<()> {
    // Adds a short SHA hash of the commit as `VERGEN_GIT_SHA` env variable
    let git = GixBuilder::default().sha(true).build()?;
    Emitter::default().add_instructions(&git)?.emit()
}
