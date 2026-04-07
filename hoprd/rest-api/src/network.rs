use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Json, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{
    Address, HoprBalance, Multiaddr,
    api::graph::{
        EdgeLinkObservable,
        traits::{EdgeNetworkObservableRead, EdgeObservableRead},
    },
};
use serde_with::{DisplayFromStr, serde_as};

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState, checksum_address_serializer};

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "price": "0.03 wxHOPR"
}))]
#[serde(rename_all = "camelCase")]
/// Contains the ticket price in HOPR tokens.
pub(crate) struct TicketPriceResponse {
    /// Price of the ticket in HOPR tokens.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0.03 wxHOPR")]
    price: HoprBalance,
}

/// Gets the current ticket price.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/network/price"),
        description = "Get the current ticket price",
        responses(
            (status = 200, description = "Current ticket price", body = TicketPriceResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Network"
    )]
pub(super) async fn price(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.get_ticket_price().await {
        Ok(price) => (StatusCode::OK, Json(TicketPriceResponse { price })).into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "probability": 0.5
}))]
#[serde(rename_all = "camelCase")]
/// Contains the winning probability of a ticket.
pub(crate) struct TicketProbabilityResponse {
    #[schema(example = 0.5)]
    /// Winning probability of a ticket.
    probability: f64,
}

/// Gets the current minimum incoming ticket winning probability defined by the network.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/network/probability"),
        description = "Get the current minimum incoming ticket winning probability defined by the network",
        responses(
            (status = 200, description = "Minimum incoming ticket winning probability defined by the network", body = TicketProbabilityResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Network"
    )]
pub(super) async fn probability(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.get_minimum_incoming_ticket_win_probability().await {
        Ok(p) => (
            StatusCode::OK,
            Json(TicketProbabilityResponse { probability: p.into() }),
        )
            .into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

// ── Connected peers endpoint ────────────────────────────────────────────────

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    "probeRate": 0.476,
    "lastUpdate": 1690000000000_u128,
    "averageLatency": 100,
    "score": 0.7
}))]
/// Immediate observation data for a connected peer.
pub(crate) struct ConnectedPeerResponse {
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    address: Address,
    #[schema(example = 0.476)]
    probe_rate: f64,
    /// Epoch milliseconds of the last observation update.
    #[schema(example = 1690000000000_u128)]
    last_update: u128,
    /// Average latency in milliseconds, if available.
    #[schema(example = 100)]
    average_latency: Option<u128>,
    #[schema(example = 0.7)]
    score: f64,
}

/// Lists peers with immediate observation data from the network graph.
///
/// Returns only peers that have at least one edge with immediate QoS data,
/// representing nodes the current node has direct transport observations for.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/network/connected"),
    description = "List connected peers with immediate observation data from the network graph",
    responses(
        (status = 200, description = "Connected peers with immediate observations", body = Vec<ConnectedPeerResponse>),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
        (status = 422, description = "Unknown failure", body = ApiError)
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Network"
)]
pub(super) async fn connected(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();
    let graph = hopr.graph();
    let edges = graph.connected_edges();

    let me_key = graph.me();

    // Collect peers that are connected (is_connected == true) with immediate QoS data.
    let mut peers = Vec::new();
    for (src, dst, obs) in &edges {
        if src != me_key {
            continue;
        }
        let Some(imm) = obs.immediate_qos() else {
            continue;
        };
        if !imm.is_connected() {
            continue;
        }

        let address = match hopr.peerid_to_chain_key(&(*dst).into()) {
            Ok(Some(addr)) => addr,
            _ => continue,
        };

        peers.push(ConnectedPeerResponse {
            address,
            probe_rate: imm.average_probe_rate(),
            last_update: obs.last_update().as_millis(),
            average_latency: imm.average_latency().map(|l| l.as_millis()),
            score: obs.score(),
        });
    }

    (StatusCode::OK, Json(peers)).into_response()
}

// ── Announced peers endpoint ────────────────────────────────────────────────

/// How a peer announcement was discovered.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    utoipa::ToSchema,
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[serde(rename_all = "lowercase")]
#[schema(example = "chain")]
pub(crate) enum AnnouncementOriginResponse {
    /// Announced via on-chain registration.
    Chain,
    /// Discovered via DHT.
    Dht,
}

impl From<hopr_lib::AnnouncementOrigin> for AnnouncementOriginResponse {
    fn from(origin: hopr_lib::AnnouncementOrigin) -> Self {
        match origin {
            hopr_lib::AnnouncementOrigin::Chain => Self::Chain,
            hopr_lib::AnnouncementOrigin::DHT => Self::Dht,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    "multiaddrs": ["/ip4/178.12.1.9/tcp/19092"],
    "origin": "chain"
}))]
#[serde(rename_all = "camelCase")]
/// A peer that has been announced.
pub(crate) struct AnnouncedPeerResponse {
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    address: Address,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/178.12.1.9/tcp/19092"]))]
    multiaddrs: Vec<Multiaddr>,
    #[schema(example = "chain")]
    origin: AnnouncementOriginResponse,
}

/// Lists all announced peers.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/network/announced"),
    description = "List all announced peers",
    responses(
        (status = 200, description = "Announced peers", body = Vec<AnnouncedPeerResponse>),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
        (status = 422, description = "Unknown failure", body = ApiError)
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Network"
)]
pub(super) async fn announced(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.announced_peers().await {
        Ok(peers) => {
            let response: Vec<AnnouncedPeerResponse> = peers
                .into_iter()
                .map(|peer| AnnouncedPeerResponse {
                    address: peer.address,
                    multiaddrs: peer.multiaddresses,
                    origin: peer.origin.into(),
                })
                .collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

// ── Graph DOT endpoint ──────────────────────────────────────────────────────

#[derive(Debug, Default, Copy, Clone, serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
#[serde(default, rename_all = "camelCase")]
#[schema(example = json!({ "reachableOnly": false }))]
/// Parameters for the network graph endpoint.
pub(crate) struct GraphQueryRequest {
    /// When true, only include edges reachable from this node via directed
    /// traversal. Disconnected subgraphs that cannot be routed through are excluded.
    #[schema(required = false)]
    #[serde(default)]
    reachable_only: bool,
}

/// Returns the network graph in DOT (Graphviz) format.
///
/// Only connected nodes (those with at least one edge) are included.
/// Nodes are labeled by their on-chain (Ethereum) address when resolvable,
/// falling back to the offchain public key hex representation.
/// Edges carry quality annotations: score, latency (ms), and capacity when available.
///
/// Pass `?reachableOnly=true` to limit the output to edges reachable from this node.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/network/graph"),
    description = "Get the network graph in DOT (Graphviz) format",
    params(GraphQueryRequest),
    responses(
        (status = 200, description = "DOT representation of the network graph", body = String, content_type = "text/plain"),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Network"
)]
pub(super) async fn graph(
    State(state): State<Arc<InternalState>>,
    Query(query): Query<GraphQueryRequest>,
) -> impl IntoResponse {
    let hopr = &state.hopr;
    let graph = hopr.graph();

    let edges = if query.reachable_only {
        graph.reachable_edges()
    } else {
        graph.connected_edges()
    };

    // Build offchain key → onchain address mapping for all nodes in the graph.
    let mut unique_keys = std::collections::HashSet::new();
    for (src, dst, _) in &edges {
        unique_keys.insert(*src);
        unique_keys.insert(*dst);
    }

    let mut key_to_addr: HashMap<hopr_lib::OffchainPublicKey, String> = HashMap::new();
    for key in &unique_keys {
        let label = match hopr.peerid_to_chain_key(&(*key).into()) {
            Ok(Some(addr)) => addr.to_string(),
            _ => key.to_string(),
        };
        key_to_addr.insert(*key, label);
    }

    let label_fn = |key: &hopr_lib::OffchainPublicKey| key_to_addr.get(key).cloned().unwrap_or_else(|| key.to_string());

    let dot = hopr_network_graph::render::render_edges_as_dot(&edges, &label_fn);

    (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "text/plain")], dot).into_response()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn announcement_origin_response_should_serialize_as_lowercase_string() {
        assert_eq!(
            serde_json::to_string(&AnnouncementOriginResponse::Chain).unwrap(),
            "\"chain\""
        );
        assert_eq!(
            serde_json::to_string(&AnnouncementOriginResponse::Dht).unwrap(),
            "\"dht\""
        );
    }

    #[test]
    fn announcement_origin_response_should_deserialize_from_lowercase_string() {
        assert_eq!(
            serde_json::from_str::<AnnouncementOriginResponse>("\"chain\"").unwrap(),
            AnnouncementOriginResponse::Chain
        );
        assert_eq!(
            serde_json::from_str::<AnnouncementOriginResponse>("\"dht\"").unwrap(),
            AnnouncementOriginResponse::Dht
        );
    }

    #[test]
    fn announcement_origin_response_should_deserialize_case_insensitively_via_strum() {
        assert_eq!(
            AnnouncementOriginResponse::from_str("Chain").unwrap(),
            AnnouncementOriginResponse::Chain
        );
        assert_eq!(
            AnnouncementOriginResponse::from_str("CHAIN").unwrap(),
            AnnouncementOriginResponse::Chain
        );
        assert_eq!(
            AnnouncementOriginResponse::from_str("DHT").unwrap(),
            AnnouncementOriginResponse::Dht
        );
    }

    #[test]
    fn announcement_origin_response_should_display_as_lowercase() {
        assert_eq!(AnnouncementOriginResponse::Chain.to_string(), "chain");
        assert_eq!(AnnouncementOriginResponse::Dht.to_string(), "dht");
    }

    #[test]
    fn announcement_origin_response_should_reject_invalid_string() {
        assert!(AnnouncementOriginResponse::from_str("invalid").is_err());
    }

    #[test]
    fn chain_origin_should_convert_from_domain_type() {
        assert_eq!(
            AnnouncementOriginResponse::from(hopr_lib::AnnouncementOrigin::Chain),
            AnnouncementOriginResponse::Chain
        );
    }

    #[test]
    fn dht_origin_should_convert_from_domain_type() {
        assert_eq!(
            AnnouncementOriginResponse::from(hopr_lib::AnnouncementOrigin::DHT),
            AnnouncementOriginResponse::Dht
        );
    }

    #[test]
    fn announced_peer_response_should_serialize_with_origin() -> anyhow::Result<()> {
        let response = AnnouncedPeerResponse {
            address: Address::default(),
            multiaddrs: vec!["/ip4/1.2.3.4/tcp/9091".parse()?],
            origin: AnnouncementOriginResponse::Chain,
        };

        let json = serde_json::to_value(&response)?;
        assert_eq!(json["origin"], "chain");
        assert!(json["multiaddrs"].is_array());
        assert!(json["address"].is_string());
        Ok(())
    }
}
