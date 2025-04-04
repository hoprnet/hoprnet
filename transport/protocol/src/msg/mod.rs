mod codec;
pub mod packet;
pub mod processor;

pub use codec::v1::MsgCodec;
pub const CURRENT_HOPR_MSG_PROTOCOL: &str = "/hopr/msg/1.0.0";
