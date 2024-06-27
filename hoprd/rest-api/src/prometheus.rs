use futures::FutureExt;
use std::future::Future;
use tower_layer::Layer;
use tower_service::Service;

/// Custom prometheus recording middleware
#[cfg(all(feature = "prometheus", not(test)))]
struct PrometheusMetricsLayer;

#[cfg(all(feature = "prometheus", not(test)))]
#[async_trait]
impl<S> Layer<S> for PrometheusMetricsLayer {
    type Service = PrometheusMetricsLayer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PrometheusMetricsLayer { inner }
    }
}

struct PrometheusMetricsMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for PrometheusMetricsMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            let path = req.url().path().to_owned();
            let method = req.method().to_string();

            let start = std::time::Instant::now();
            let response: Response = future.await?;
            let response_duration = start.elapsed();

            let status = response.status();

            // We're not interested on metrics for non-functional
            if path.starts_with("/api/v3/") && !path.contains("node/metrics") {
                METRIC_COUNT_API_CALLS.increment(&[&path, &method, &status.to_string()]);
                METRIC_COUNT_API_CALLS_TIMING.observe(&[&path, &method], response_duration.as_secs_f64());
            }

            return Ok(response);
        })
    }
}
