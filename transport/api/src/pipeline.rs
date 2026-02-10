use hopr_api::{
    chain::{ChainKeyOperations, ChainReadChannelOperations, ChainValues},
    db::HoprDbTicketOperations,
};
use hopr_async_runtime::AbortableList;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_protocol_app::prelude::*;
use hopr_protocol_hopr::prelude::*;
use hopr_transport_protocol::{TicketEvent, run_packet_pipeline};

use crate::{HoprTransportProcess, config::HoprPacketPipelineConfig};

/// Contains all components required to run the HOPR packet pipeline.
#[derive(Clone)]
pub struct HoprPipelineComponents<TEvt, S, Chain, Db> {
    /// Sink for [`TicketEvents`](TicketEvent).
    pub ticket_events: TEvt,
    /// Store for SURBs and Reply Openers.
    pub surb_store: S,
    /// Chain API for interacting with the blockchain.
    pub chain_api: Chain,
    /// Database for storing tickets and other data.
    pub db: Db,
}

pub fn run_hopr_packet_pipeline<WIn, WOut, Chain, S, Db, TEvt, AppOut, AppIn>(
    (packet_key, chain_key): (OffchainKeypair, ChainKeypair),
    wire_msg: (WOut, WIn),
    api: (AppOut, AppIn),
    components: HoprPipelineComponents<TEvt, S, Chain, Db>,
    channels_dst: Hash,
    cfg: HoprPacketPipelineConfig,
) -> AbortableList<HoprTransportProcess>
where
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    WIn: futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + 'static,
    Db: HoprDbTicketOperations + Clone + Send + Sync + 'static,
    Chain: ChainKeyOperations + ChainReadChannelOperations + ChainValues + Clone + Send + Sync + 'static,
    S: SurbStore + Clone + Send + Sync + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::error::Error,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting, ApplicationDataOut)> + Send + 'static,
{
    let HoprPipelineComponents {
        ticket_events,
        surb_store,
        chain_api,
        db,
    } = components;

    let ticket_proc = HoprTicketProcessor::new(
        chain_api.clone(),
        db.clone(),
        chain_key.clone(),
        channels_dst,
        cfg.ticket_processing,
    );
    let encoder = HoprEncoder::new(
        chain_key.clone(),
        chain_api.clone(),
        surb_store.clone(),
        ticket_proc.clone(),
        channels_dst,
        cfg.codec,
    );
    let decoder = HoprDecoder::new(
        (packet_key.clone(), chain_key.clone()),
        chain_api.clone(),
        surb_store,
        ticket_proc.clone(),
        channels_dst,
        cfg.codec,
    );

    let mut processes = AbortableList::default();

    #[cfg(feature = "capture")]
    let (encoder, decoder) = {
        use crate::capture;

        let writer: Box<dyn capture::PacketWriter + Send + 'static> =
            if let Ok(desc) = std::env::var("HOPR_CAPTURE_PACKETS") {
                if let Ok(pcap_writer) = std::fs::File::create(&desc).and_then(capture::PcapPacketWriter::new) {
                    tracing::warn!("pcap file packet capture initialized to {desc}");
                    Box::new(pcap_writer)
                } else {
                    tracing::error!(desc, "failed to create packet capture: invalid socket address or file");
                    Box::new(capture::NullWriter)
                }
            } else {
                tracing::warn!("no packet capture specified");
                Box::new(capture::NullWriter)
            };

        let (sender, ah) = capture::packet_capture_channel(writer);
        processes.insert(HoprTransportProcess::Capture, ah);
        (
            capture::CapturePacketCodec::new(encoder, *packet_key.public(), sender.clone()),
            capture::CapturePacketCodec::new(decoder, *packet_key.public(), sender.clone()),
        )
    };

    let (index_sync_handle, index_sync_reg) = futures::future::AbortHandle::new_pair();
    hopr_async_runtime::prelude::spawn(ticket_proc.outgoing_index_sync_task(index_sync_reg));
    processes.insert(HoprTransportProcess::OutgoingIndexSync, index_sync_handle);

    processes.flat_map_extend_from(
        run_packet_pipeline(
            packet_key.clone(),
            wire_msg,
            (encoder, decoder),
            ticket_proc,
            ticket_events,
            cfg.pipeline,
            api,
        ),
        HoprTransportProcess::Pipeline,
    );

    processes
}
