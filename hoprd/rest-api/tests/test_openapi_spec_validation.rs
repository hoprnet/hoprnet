use utoipa::OpenApi;

#[test]
fn openapi_spec_should_validate_basic() -> anyhow::Result<()> {
    assert!(oas3::from_json(hoprd_api::ApiDoc::openapi().to_pretty_json()?).is_ok());
    Ok(())
}
