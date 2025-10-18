use hopr_api::db::{HoprDbTicketOperations, TicketSelector};
use hopr_protocol_hopr::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::channels::ChannelId;
use hopr_primitive_types::balance::HoprBalance;
use hopr_transport_protocol::PacketPipelineProcesses;

pub struct HoprPacketPipelineConfig {
    pub encoder_cfg: HoprEncoderConfig,
    pub decoder_cfg: HoprDecoderConfig,
    pub ticket_proc_cfg: HoprTicketProcessorConfig,
    pub surb_cfg: SurbStoreConfig,
}

pub struct TicketIndexTracker<Db>(std::sync::Arc<Db>);

impl<Db> Clone for TicketIndexTracker<Db> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


#[async_trait::async_trait]
impl<Db: HoprDbTicketOperations + Send + Sync> TicketTracker for TicketIndexTracker<Db> {
    type Error = Db::Error;

    async fn next_outgoing_ticket_index(&self, channel_id: &ChannelId) -> Result<u64, Self::Error> {
        self.0.increment_outgoing_ticket_index(channel_id).await
    }

    async fn incoming_channel_unrealized_balance(&self, channel_id: &ChannelId, epoch: u32) -> Result<HoprBalance, Self::Error> {
        self.0.unrealized_value(TicketSelector::new(*channel_id, epoch)).await
    }
}

pub async fn run_hopr_packet_pipeline<WIn, WOut, R, Db, TEvt, AppOut, AppIn>(
    (packet_key, chain_key): (OffchainKeypair, ChainKeypair),
    wire_msg: (WOut, WIn),
    provider: R,
    db: Db,
    cfg: HoprPacketPipelineConfig,
    ticket_events: TEvt,
    api: (AppOut, AppIn),
) -> std::collections::HashMap<PacketPipelineProcesses, hopr_async_runtime::AbortHandle> {
    let surb_store = MemorySurbStore::new(cfg.surb_cfg);
    let tracker = TicketIndexTracker(db.into());

    let encoder = HoprEncoder::new(provider.clone(), surb_store.clone(), tracker.clone(), chain_key.clone(), cfg.encoder_cfg);
    let decoder = HoprDecoder::new(provider.clone(), surb_store, tracker, (packet_key.clone(), chain_key.clone()), cfg.decoder_cfg);
    let ticket_proc = HoprTicketProcessor::new(provider.clone(), chain_key.clone(), cfg.ticket_proc_cfg);
    
}