use utoipa::OpenApi;

#[test]
fn openapi_spec_should_validate_basic() -> anyhow::Result<()> {
    oas3::from_json(hoprd_api::ApiDoc::openapi().to_pretty_json()?)?;

    Ok(())
}
