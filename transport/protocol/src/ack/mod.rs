pub mod processor;

pub mod codec;

pub type AckCodec = codec::CborCodec<hopr_internal_types::protocol::Acknowledgement>;
pub const CURRENT_HOPR_ACK_PROTOCOL: &str = "/hopr/ack/1.0.0";
