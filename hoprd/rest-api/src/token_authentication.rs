/// Token-based authentication middleware
struct TokenAuthenticationLayer;

impl<S> Layer<S> for TokenAuthenticationLayer {
    type Service = TokenAuthenticationLayer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TokenAuthenticationLayer { inner }
    }
}

struct TokenAuthenticationMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for TokenAuthenticationMiddleware<S>
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
            let auth = request.state().auth.clone();

            let x_auth_header = HeaderName::from_str("x-auth-token")?;
            let websocket_proto_header = HeaderName::from_str("Sec-Websocket-Protocol")?;

            let is_authorized = match auth.as_ref() {
                Auth::Token(expected_token) => {
                    let auth_headers = request
                        .iter()
                        .filter_map(|(n, v)| {
                            (AUTHORIZATION.eq(n) || x_auth_header.eq(n) || websocket_proto_header.eq(n))
                                .then_some((n, v.as_str()))
                        })
                        .collect::<Vec<_>>();

                    // Use "Authorization Bearer <token>" and "X-Auth-Token <token>" headers and "Sec-Websocket-Protocol"
                    (!auth_headers.is_empty()
                    && (auth_headers.contains(&(&AUTHORIZATION, &format!("Bearer {}", expected_token)))
                        || auth_headers.contains(&(&x_auth_header, expected_token)))
                )

                // TODO: Replace with proper JS compliant solution
                // The following line would never be needed, if the JavaScript browser was able to properly
                // pass the x-auth-token or Bearer headers.
                || request.url().as_str().contains(format!("messages/websocket?apiToken={expected_token}").as_str())
                }
                Auth::None => true,
            };

            if !is_authorized {
                let reject_response = Response::builder(StatusCode::Unauthorized)
                    .content_type(mime::JSON)
                    .body(ApiErrorStatus::Unauthorized)
                    .build();

                return Ok(reject_response);
            }

            // Go forward to the next middleware or request handler
            return Ok(future.await?);
        })
    }
}
