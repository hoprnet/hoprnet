use axum::{
    extract::{OriginalUri, Request, State},
    http::{
        header::{HeaderName, AUTHORIZATION},
        status::StatusCode,
        HeaderMap,
    },
    middleware::Next,
    response::IntoResponse,
};
use std::str::FromStr;
use urlencoding::decode;

use crate::{ApiErrorStatus, Auth, InternalState};

pub(crate) async fn authenticate(
    State(state): State<InternalState>,
    uri: OriginalUri,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let auth = state.auth.clone();

    let x_auth_header = HeaderName::from_str("x-auth-token").expect("Invalid header name: x-auth-token");
    let websocket_proto_header =
        HeaderName::from_str("Sec-Websocket-Protocol").expect("Invalid header name: Sec-Websocket-Protocol");

    let is_authorized = match auth.as_ref() {
        Auth::Token(expected_token) => {
            let auth_headers = headers
                .iter()
                .filter_map(|(n, v)| {
                    (AUTHORIZATION.eq(n) || x_auth_header.eq(n) || websocket_proto_header.eq(n))
                        .then_some((n, v.to_str().expect("Invalid header value")))
                })
                .collect::<Vec<_>>();

            // We have multiple websocket routes that need authentication checks
            let is_ws_auth = if uri.path().starts_with("/api/v3/messages/websocket")
                || uri.path().starts_with("/api/v3/session/websocket")
            {
                uri.query()
                    .map(|q| match decode(q) {
                        Ok(decoded) => decoded.into_owned().contains(&format!("apiToken={}", expected_token)),
                        Err(_) => false,
                    })
                    .unwrap_or(false)
            } else {
                false
            };
            // Use "Authorization Bearer <token>" and "X-Auth-Token <token>" headers and "Sec-Websocket-Protocol"
            (!auth_headers.is_empty()
                    && (auth_headers.contains(&(&AUTHORIZATION, &format!("Bearer {}", expected_token)))
                        || auth_headers.contains(&(&x_auth_header, expected_token)))
                )
                // The following line would never be needed, if the JavaScript browser was able to properly
                // pass the x-auth-token or Bearer headers.
                || is_ws_auth
        }
        Auth::None => true,
    };

    if !is_authorized {
        return (StatusCode::UNAUTHORIZED, ApiErrorStatus::Unauthorized).into_response();
    }

    // Go forward to the next middleware or request handler
    next.run(request).await
}
