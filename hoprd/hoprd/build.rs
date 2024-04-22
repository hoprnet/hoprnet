use anyhow::Result;
use vergen_git2::{Emitter, Git2Builder};

pub fn main() -> Result<()> {
    // Add short SHA hash of the commit as `VERGEN_GIT_SHA` env variable
    let git = Git2Builder::default().sha(true).build()?;

    Emitter::default().add_instructions(&git)?.emit()
}
