use anyhow::Result;
use vergen_gitcl::{Emitter, GitclBuilder};

pub fn main() -> Result<()> {
    // Add short SHA hash of the commit as `VERGEN_GIT_SHA` env variable
    let git = GitclBuilder::default().sha(true).build()?;

    Emitter::default().add_instructions(&git)?.emit()
}
