use anyhow::Context;
use utoipa::OpenApi;

use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn openapi_spec_should_validate() -> anyhow::Result<()> {
    let mut spec = NamedTempFile::new()?;
    spec.write_all(hoprd_api::ApiDoc::openapi().to_pretty_json()?.as_bytes())?;

    let command = Command::new("vacuum")
        .args([
            "lint",
            "-b",
            spec.path().to_str().context("should be a valid tmp path")?,
        ])
        .output()?;

    assert!(command.status.success());

    Ok(())
}
