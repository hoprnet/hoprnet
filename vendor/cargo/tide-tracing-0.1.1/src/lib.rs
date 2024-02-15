#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::time::Instant;

use tide::{Middleware, Next, Request};
use tracing::{error, error_span, field, info, info_span, warn, warn_span};
use tracing_futures::Instrument;
#[cfg(feature = "request_id")]
use uuid::Uuid;

/// Log all incoming requests and responses with tracing spans.
///
/// ```
/// let mut app = tide::Server::new();
/// app.with(tide_tracing::TraceMiddleware::new());
/// ```
#[derive(Debug, Default, Clone)]
pub struct TraceMiddleware;

impl TraceMiddleware {
    /// Create a new instance of `TraceMiddleware`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Log a request and a response.
    async fn log<'a, State: Clone + Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> tide::Result {
        let path = ctx.url().path().to_owned();
        let method = ctx.method();

        Ok(async {
            info!("received");
            let start = Instant::now();
            let response = next.run(ctx).await;
            let duration = start.elapsed();
            let status = response.status();

            info_span!("Response", http.status_code = status as u16, http.duration = ?duration)
                .in_scope(|| {
                    if status.is_server_error() {
                        let span = error_span!(
                            "Internal error",
                            detail = field::Empty,
                            error = field::Empty
                        );
                        if let Some(error) = response.error() {
                            span.record("error", &field::display(error));
                            span.record("detail", &field::debug(error));
                        }
                        span.in_scope(|| error!("sent"));
                    } else if status.is_client_error() {
                        warn_span!("Client error").in_scope(|| warn!("sent"));
                    } else {
                        info!("sent")
                    }
                });
            response
        }
        .instrument({
            let span = info_span!(
                "Request",
                http.method = %method,
                http.target = %path,
                request_id = field::Empty,
            );

            // If the request_id feature is enabled, add a new request_id record to the span.
            #[cfg(feature = "request_id")]
            span.record("request_id", Uuid::new_v4().to_string());

            span
        })
        .await)
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for TraceMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        self.log(req, next).await
    }
}
