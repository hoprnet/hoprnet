use tide_tracing::TraceMiddleware;

use tide::{Error, Response, Result, StatusCode};

#[async_std::main]
async fn main() -> tide::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut app = tide::Server::new();

    app.with(TraceMiddleware::new());

    app.at("/working_endpoint")
        .get(|_| async { Ok(Response::new(StatusCode::Ok)) });
    app.at("/client_error")
        .get(|_| async { Ok(Response::new(StatusCode::NotFound)) });
    app.at("/internal_error").get(|_| async {
        Result::<Response>::Err(Error::from_str(
            StatusCode::ServiceUnavailable,
            "This message will be displayed",
        ))
    });

    app.listen("localhost:8081").await?;

    Ok(())
}
