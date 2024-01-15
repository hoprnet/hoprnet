use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoprTransportError {
    #[error("API error: {0}")]
    Api(String),

    #[error("General error: {0}")]
    General(#[from] hopr_primitive_types::errors::GeneralError),

    #[error("Db error: {0}")]
    Db(#[from] utils_db::errors::DbError),

    #[error("Path error: {0}")]
    Path(#[from] core_path::errors::PathError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] core_protocol::errors::ProtocolError),

    #[error("Packet error: {0}")]
    Packet(#[from] core_packet::errors::PacketError),

    #[error("Type error: {0}")]
    Types(#[from] hopr_internal_types::errors::CoreTypesError),
}

pub type Result<T> = core::result::Result<T, HoprTransportError>;
