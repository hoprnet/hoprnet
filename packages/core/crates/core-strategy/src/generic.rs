pub struct ChannelEntry {

}

pub type PeerQualityEvaluator = dyn Fn(&str) -> f32;

pub trait ChannelStrategy {
    fn name(&self) -> &str;

    fn tick(&self,
            balance: u64,
            network_size: u32,
            current_channels: Vec<ChannelEntry>,
            quality_of: PeerQualityEvaluator,
            peer_ids: Vec<String>) -> StrategyTickResult;
}

pub struct ChannelOpenRequest {
    pub peer_id: String,
    pub stake: f64
}

pub struct StrategyTickResult {
    to_open: Vec<ChannelOpenRequest>,
    to_close: Vec<String>
}