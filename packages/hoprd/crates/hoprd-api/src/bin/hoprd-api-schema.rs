use utoipa::OpenApi;

/// Schema generator executable
/// 
/// This executable statically generates the OpenAPI schema from the available API
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", hoprd_api::ApiDoc::openapi().to_pretty_json().unwrap());

    Ok(())
}