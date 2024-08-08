use axum::{
    extract::{Json, Path, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{collections::HashMap, sync::Arc};

use crate::{ApiErrorStatus, InternalState, BASE_PATH};

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "peerId": "12D3KooWRWeTozREYHzWTbuCYskdYhED1MXpDwTrmccwzFrd2mEA"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct PeerIdResponse {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    pub peer_id: PeerId,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "alias": "Alice",
        "peerId": "12D3KooWRWeTozREYHzWTbuCYskdYhED1MXpDwTrmccwzFrd2mEA"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct AliasPeerIdBodyRequest {
    pub alias: String,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    pub peer_id: PeerId,
}

/// Get each previously set alias and its corresponding PeerId.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/aliases"),
        responses(
            (status = 200, description = "Each alias with its corresponding PeerId", body = HashMap<String, String>, example = json!({
                    "alice": "12D3KooWPWD5P5ZzMRDckgfVaicY5JNoo7JywGotoAv17d7iKx1z",
                    "me": "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS"
            })),
            (status = 401, description = "Invalid authorization token.", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Alias",
    )]
pub(super) async fn aliases(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let aliases = state.aliases.clone();

    let aliases = aliases
        .read()
        .await
        .iter()
        .map(|(key, value)| (key.clone(), value.to_string()))
        .collect::<HashMap<String, String>>();

    (StatusCode::OK, Json(aliases)).into_response()
}

/// Set alias for a peer with a specific PeerId.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/aliases"),
        request_body(
            content = AliasPeerIdBodyRequest,
            description = "Alias name along with the PeerId to be aliased",
            content_type = "application/json"),
        responses(
            (status = 201, description = "Alias set successfully.", body = PeerIdResponse),
            (status = 400, description = "Invalid PeerId: The format or length of the peerId is incorrect.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 409, description = "Given PeerId is already aliased.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Alias",
    )]
pub(super) async fn set_alias(
    State(state): State<Arc<InternalState>>,
    Json(args): Json<AliasPeerIdBodyRequest>,
) -> impl IntoResponse {
    let aliases = state.aliases.clone();

    let inserted = aliases.write().await.insert_no_overwrite(args.alias, args.peer_id);
    match inserted {
        Ok(_) => (StatusCode::CREATED, Json(PeerIdResponse { peer_id: args.peer_id })).into_response(),
        Err(_) => (StatusCode::CONFLICT, ApiErrorStatus::AliasAlreadyExists).into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct GetAliasParams {
    alias: String,
}

/// Get alias for the PeerId (Hopr address) that have this alias assigned to it.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/aliases/{{alias}}"),
        params(
            ("alias" = String, Path, description = "Alias to be shown"),
        ),
        responses(
            (status = 200, description = "Get PeerId for an alias", body = PeerIdResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "PeerId not found", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Alias",
    )]
pub(super) async fn get_alias(
    Path(GetAliasParams { alias }): Path<GetAliasParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let aliases = state.aliases.clone();

    let aliases = aliases.read().await;
    if let Some(peer_id) = aliases.get_by_left(&alias) {
        (StatusCode::OK, Json(PeerIdResponse { peer_id: *peer_id })).into_response()
    } else {
        (StatusCode::NOT_FOUND, ApiErrorStatus::InvalidInput).into_response()
    }
}

#[derive(Deserialize)]
pub(crate) struct DeleteAliasParams {
    alias: String,
}

/// Delete an alias.
#[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/aliases/{{alias}}"),
        params(
            ("alias" = String, Path, description = "Alias to be shown"),
        ),
        responses(
            (status = 204, description = "Alias removed successfully"),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)   // This can never happen
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Alias",
    )]
pub(super) async fn delete_alias(
    Path(DeleteAliasParams { alias }): Path<DeleteAliasParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let aliases = state.aliases.clone();

    let _ = aliases.write().await.remove_by_left(&alias);

    (StatusCode::NO_CONTENT, "").into_response()
}
