use std::{str::FromStr, sync::atomic::Ordering::Relaxed};

use axum::{
    extract::{OriginalUri, Request, State},
    http::{
        HeaderMap,
        header::{AUTHORIZATION, HeaderName},
        status::StatusCode,
    },
    middleware::Next,
    response::IntoResponse,
};
use urlencoding::decode;

use crate::{ApiErrorStatus, Auth, BASE_PATH, InternalState};

fn is_a_websocket_uri(uri: &OriginalUri) -> bool {
    const SESSION_PATH: &str = const_format::formatcp!("{BASE_PATH}/session/websocket");

    uri.path().starts_with(SESSION_PATH)
}

pub(crate) async fn cap_websockets(
    State(state): State<InternalState>,
    uri: OriginalUri,
    _headers: HeaderMap,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let max_websocket_count = std::env::var("HOPR_INTERNAL_REST_API_MAX_CONCURRENT_WEBSOCKET_COUNT")
        .and_then(|v| v.parse::<u16>().map_err(|_e| std::env::VarError::NotPresent))
        .unwrap_or(10);

    if is_a_websocket_uri(&uri) {
        let ws_count = state.websocket_active_count;

        if ws_count.fetch_add(1, Relaxed) > max_websocket_count {
            ws_count.fetch_sub(1, Relaxed);

            return (
                StatusCode::TOO_MANY_REQUESTS,
                ApiErrorStatus::TooManyOpenWebsocketConnections,
            )
                .into_response();
        }
    }

    // Go forward to the next middleware or request handler
    next.run(request).await
}

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

            let is_ws_auth = if is_a_websocket_uri(&uri) {
                uri.query()
                    .map(|q| {
                        // Reasonable limit for query string
                        if q.len() > 2048 {
                            return false;
                        }
                        match decode(q) {
                            Ok(decoded) => decoded.into_owned().contains(&format!("apiToken={expected_token}")),
                            Err(_) => false,
                        }
                    })
                    .unwrap_or(false)
            } else {
                false
            };
            // Use "Authorization Bearer <token>" and "X-Auth-Token <token>" headers and "Sec-Websocket-Protocol"
            (!auth_headers.is_empty()
                    && (auth_headers.contains(&(&AUTHORIZATION, &format!("Bearer {expected_token}")))
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
