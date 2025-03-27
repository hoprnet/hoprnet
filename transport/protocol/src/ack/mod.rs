pub mod processor;

pub mod codec;

pub type AckCodec = codec::CborCodec<hopr_internal_types::protocol::Acknowledgement>;
