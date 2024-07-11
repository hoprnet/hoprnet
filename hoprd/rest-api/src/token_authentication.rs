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

use crate::{ApiErrorStatus, Auth, InternalState};

pub(crate) async fn authenticate(
    State(state): State<InternalState>,
    uri: OriginalUri,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let auth = state.auth.clone();

    let x_auth_header = HeaderName::from_str("x-auth-token").unwrap();
    let websocket_proto_header = HeaderName::from_str("Sec-Websocket-Protocol").unwrap();

    let is_authorized = match auth.as_ref() {
        Auth::Token(expected_token) => {
            let auth_headers = headers
                .iter()
                .filter_map(|(n, v)| {
                    (AUTHORIZATION.eq(n) || x_auth_header.eq(n) || websocket_proto_header.eq(n))
                        .then_some((n, v.to_str().unwrap()))
                })
                .collect::<Vec<_>>();

            let is_ws_auth = if let Some(s) = uri.path_and_query() {
                s.as_str()
                    .contains(format!("messages/websocket?apiToken={expected_token}").as_str())
            } else {
                false
            };
            // Use "Authorization Bearer <token>" and "X-Auth-Token <token>" headers and "Sec-Websocket-Protocol"
            (!auth_headers.is_empty()
                    && (auth_headers.contains(&(&AUTHORIZATION, &format!("Bearer {}", expected_token)))
                        || auth_headers.contains(&(&x_auth_header, &expected_token)))
                )
                // TODO: Replace with proper JS compliant solution
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
    return next.run(request).await;
}
