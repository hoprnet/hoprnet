use std::str::FromStr;

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

use crate::{ApiErrorStatus, Auth, BlokliClientLike, InternalState};

pub(crate) async fn authenticate<C: BlokliClientLike>(
    State(state): State<InternalState<C>>,
    _uri: OriginalUri,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let auth = state.auth.clone();

    let x_auth_header = HeaderName::from_str("x-auth-token").expect("Invalid header name: x-auth-token");

    let is_authorized = match auth.as_ref() {
        Auth::Token(expected_token) => {
            let auth_headers = headers
                .iter()
                .filter_map(|(n, v)| {
                    (AUTHORIZATION.eq(n) || x_auth_header.eq(n))
                        .then_some((n, v.to_str().expect("Invalid header value")))
                })
                .collect::<Vec<_>>();

            // Use "Authorization Bearer <token>" and "X-Auth-Token <token>" headers
            !auth_headers.is_empty()
                && (auth_headers.contains(&(&AUTHORIZATION, &format!("Bearer {expected_token}")))
                    || auth_headers.contains(&(&x_auth_header, expected_token)))
        }
        Auth::None => true,
    };

    if !is_authorized {
        return (StatusCode::UNAUTHORIZED, ApiErrorStatus::Unauthorized).into_response();
    }

    // Go forward to the next middleware or request handler
    next.run(request).await
}
