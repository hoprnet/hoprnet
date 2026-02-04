// Re-export channel operation result types from hopr_api::node
pub use hopr_api::node::{CloseChannelResult, OpenChannelResult};

#[async_trait::async_trait]
pub trait HoprChainApi {}
