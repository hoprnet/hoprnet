use std::sync::Arc;
use tracing::{error, warn};
use hopr_api::chain::{ChainKeyOperations, ChainReadChannelOperations, ChainValues};
use hopr_api::db::{HoprDbTicketOperations, TicketSelector};
use hopr_protocol_hopr::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::channels::ChannelId;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_primitive_types::balance::HoprBalance;
use hopr_protocol_app::prelude::{ApplicationDataIn, ApplicationDataOut};
use hopr_transport_protocol::{run_packet_pipeline, PacketPipelineProcesses, TicketEvent};
use crate::capture;

#[derive(Clone, Debug, Default)]
pub struct HoprPacketPipelineConfig {
    pub codec_cfg: HoprCodecConfig,
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

pub fn run_hopr_packet_pipeline<WIn, WOut, R, Db, TEvt, AppOut, AppIn>(
    (packet_key, chain_key): (OffchainKeypair, ChainKeypair),
    wire_msg: (WOut, WIn),
    api: (AppOut, AppIn),
    ticket_events: TEvt,
    provider: R,
    db: Db,
    cfg: HoprPacketPipelineConfig,
) -> std::collections::HashMap<PacketPipelineProcesses, hopr_async_runtime::AbortHandle>
where
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::fmt::Display,
    WIn: futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + 'static,
    Db: HoprDbTicketOperations + Send + Sync + 'static,
    R: ChainKeyOperations + ChainReadChannelOperations + ChainValues + Send + Sync + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::fmt::Display,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::fmt::Display,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting, ApplicationDataOut)> + Send + 'static,
{
    let surb_store = MemorySurbStore::new(cfg.surb_cfg);
    let tracker = TicketIndexTracker(db.into());

    let encoder = HoprEncoder::new(provider.clone(), surb_store.clone(), tracker.clone(), chain_key.clone(), cfg.codec_cfg);
    let decoder = HoprDecoder::new(provider.clone(), surb_store, tracker, (packet_key.clone(), chain_key.clone()), cfg.codec_cfg);
    let ticket_proc = HoprTicketProcessor::new(provider.clone(), chain_key.clone(), cfg.ticket_proc_cfg);

    #[cfg(feature = "capture")]
    let (encoder, decoder) = {
        let writer: Box<dyn capture::PacketWriter + Send + 'static> =
            if let Ok(desc) = std::env::var("HOPR_CAPTURE_PACKETS") {
                if let Ok(pcap_writer) = std::fs::File::create(&desc).and_then(capture::PcapPacketWriter::new) {
                    warn!("pcap file packet capture initialized to {desc}");
                    Box::new(pcap_writer)
                } else {
                    error!(desc, "failed to create packet capture: invalid socket address or file");
                    Box::new(capture::NullWriter)
                }
            } else {
                warn!("no packet capture specified");
                Box::new(capture::NullWriter)
            };

        let (sender, ah) = capture::packet_capture_channel(writer);
        (
            capture::CapturePacketCodec::new(encoder, *packet_key.public(), sender.clone()),
            capture::CapturePacketCodec::new(decoder, *packet_key.public(), sender.clone()),
        )
    };

    run_packet_pipeline(
        packet_key.clone(),
        wire_msg,
        (encoder,decoder),
        ticket_proc,
        ticket_events,
        api
    )
}