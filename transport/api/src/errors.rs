use thiserror::Error;

pub use hopr_transport_network::errors::NetworkingError;
pub use hopr_transport_protocol::errors::ProtocolError;

/// Errors produced by the crate.
#[derive(Error, Debug)]
pub enum HoprTransportError {
    #[error("API error: {0}")]
    Api(String),

    #[error("General error: {0}")]
    General(#[from] hopr_primitive_types::errors::GeneralError),

    #[error("Db error: {0}")]
    Db(#[from] hopr_db_sql::api::errors::DbError),

    // TODO(20250114): Unify all Databse API functionality in the db_api crate and remove this error.
    #[error("Db error: {0}")]
    Database(#[from] hopr_db_sql::errors::DbSqlError),

    #[error("Path error: {0}")]
    Path(#[from] hopr_path::errors::PathError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] hopr_transport_protocol::errors::ProtocolError),

    #[error("Transport session error: {0}")]
    Session(#[from] hopr_transport_session::errors::TransportSessionError),

    #[error("Packet error: {0}")]
    Packet(#[from] hopr_crypto_packet::errors::PacketError),

    #[error("Type error: {0}")]
    Types(#[from] hopr_internal_types::errors::CoreTypesError),

    #[error("Network monitoring error: {0}")]
    NetworkError(#[from] hopr_transport_network::errors::NetworkingError),
}

/// Result produced by the crate, uses the [HoprTransportError] as the error type.
pub type Result<T> = core::result::Result<T, HoprTransportError>;
