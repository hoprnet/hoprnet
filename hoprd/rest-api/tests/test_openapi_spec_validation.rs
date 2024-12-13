// use anyhow::Context;
use utoipa::OpenApi;

// use std::io::Write;
// use std::process::Command;
// use tempfile::NamedTempFile;

#[test]
fn openapi_spec_should_validate_basic() -> anyhow::Result<()> {
    assert!(oas3::from_str(hoprd_api::ApiDoc::openapi().to_pretty_json()?.as_str()).is_ok());

    Ok(())
}

///// The test cannot be run in CI as the temporary file creation leads to a non-existing file error
// #[test]
// fn openapi_spec_should_validate_advanced() -> anyhow::Result<()> {
//     if std::env::var("CI").map_or(true, |v| v != "true") {
//         let mut spec = NamedTempFile::new()?;
//         spec.write_all(hoprd_api::ApiDoc::openapi().to_pretty_json()?.as_bytes())?;

//         let command = Command::new("vacuum")
//             .args([
//                 "lint",
//                 "-b",
//                 spec.path().to_str().context("should be a valid tmp path")?,
//             ])
//             .output()?;

//         assert!(command.status.success());
//     }

//     Ok(())
// }
