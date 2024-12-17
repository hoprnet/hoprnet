use utoipa::OpenApi;

#[test]
fn openapi_spec_should_validate_basic() -> anyhow::Result<()> {
    assert!(oas3::from_str(hoprd_api::ApiDoc::openapi().to_pretty_json()?.as_str()).is_ok());

    Ok(())
}
