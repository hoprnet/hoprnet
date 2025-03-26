pub mod processor;

pub type AckCodec = asynchronous_codec::CborCodec<
    hopr_internal_types::protocol::Acknowledgement,
    hopr_internal_types::protocol::Acknowledgement,
>;
