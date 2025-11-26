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

use crate::HoprTransportProcess;

#[derive(Clone, Copy, Debug, Default)]
pub struct HoprPacketPipelineConfig {
    pub codec_cfg: HoprCodecConfig,
    pub ticket_proc_cfg: HoprTicketProcessorConfig,
}

#[allow(clippy::too_many_arguments)]
pub fn run_hopr_packet_pipeline<WIn, WOut, R, S, Db, TEvt, AppOut, AppIn>(
    (packet_key, chain_key): (OffchainKeypair, ChainKeypair),
    wire_msg: (WOut, WIn),
    api: (AppOut, AppIn),
    ticket_events: TEvt,
    surb_store: S,
    provider: R,
    db: Db,
    cfg: HoprPacketPipelineConfig,
) -> AbortableList<HoprTransportProcess>
where
    WOut: futures::Sink<(PeerId, Box<[u8]>)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::fmt::Display,
    WIn: futures::Stream<Item = (PeerId, Box<[u8]>)> + Send + 'static,
    Db: HoprDbTicketOperations + Clone + Send + Sync + 'static,
    R: ChainKeyOperations + ChainReadChannelOperations + ChainValues + Clone + Send + Sync + 'static,
    S: SurbStore + Clone + Send + Sync + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::fmt::Display,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::fmt::Display,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting, ApplicationDataOut)> + Send + 'static,
{
    let ticket_proc = std::sync::Arc::new(HoprTicketProcessor::new(
        provider.clone(),
        db.clone(),
        chain_key.clone(),
        cfg.ticket_proc_cfg,
    ));
    let encoder = HoprEncoder::new(
        provider.clone(),
        surb_store.clone(),
        ticket_proc.clone(),
        chain_key.clone(),
        cfg.codec_cfg,
    );
    let decoder = HoprDecoder::new(
        provider.clone(),
        surb_store,
        ticket_proc.clone(),
        (packet_key.clone(), chain_key.clone()),
        cfg.codec_cfg,
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

    processes.flat_map_extend_from(
        run_packet_pipeline(
            packet_key.clone(),
            wire_msg,
            (encoder, decoder),
            ticket_proc,
            ticket_events,
            api,
        ),
        HoprTransportProcess::Pipeline,
    );

    processes
}
