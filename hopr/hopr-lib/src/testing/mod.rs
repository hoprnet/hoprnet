use hopr_utils_chain_connector::{
    HoprBlockchainSafeConnector,
    testing::{BlokliTestClient, FullStateEmulator},
};

pub mod dummies;
pub mod fixtures;
pub mod hopr;

type TestingConnector = std::sync::Arc<HoprBlockchainSafeConnector<BlokliTestClient<FullStateEmulator>>>;
