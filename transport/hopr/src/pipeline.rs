use bytes::Bytes;
use hopr_api::{
    chain::{ChainKeyOperations, ChainReadChannelOperations, ChainReadTicketOperations, ChainValues},
    tickets::TicketFactory,
    types::{
        crypto::prelude::*,
        internal::{prelude::*, routing::ResolvedTransportRouting},
    },
};
use hopr_crypto_packet::HoprSurb;
use hopr_protocol_app::prelude::*;
use hopr_protocol_hopr::prelude::*;
use hopr_utils::runtime::AbortableList;

use crate::{HoprTransportProcess, config::HoprPacketPipelineConfig, protocol::PacketPipelineBuilder};

/// Builder for the HOPR packet pipeline.
///
/// Creates the encoder/decoder, the unacknowledged-ticket processor, optionally hooks up the
/// packet capture (when the `capture` feature is enabled) and finally delegates to the lower-level
/// [`PacketPipelineBuilder`] to spawn the per-stage tasks. The shape of the spawned pipeline is
/// selected by which terminal `build_for_*` method is called:
///
/// - [`HoprPacketPipelineBuilder::build_for_relay`] — full pipeline. Requires
///   [`HoprPacketPipelineBuilder::with_ticket_events`] to be called beforehand.
/// - [`HoprPacketPipelineBuilder::build_for_entry`] — Entry nodes. Ticket events are not needed (and any value
///   previously set is ignored).
/// - [`HoprPacketPipelineBuilder::build_for_exit`] — Exit nodes. Ticket events are not needed (and any value previously
///   set is ignored).
///
/// The configuration ([`HoprPacketPipelineConfig`]) is optional and defaults to
/// `HoprPacketPipelineConfig::default()`; override it via [`HoprPacketPipelineBuilder::with_config`].
pub struct HoprPacketPipelineBuilder<
    WIn,
    WOut,
    Chain,
    S,
    TFact,
    AppOut,
    AppIn,
    TEvt = futures::sink::Drain<hopr_api::node::TicketEvent>,
> {
    packet_key: OffchainKeypair,
    chain_key: ChainKeypair,
    wire_msg: (WOut, WIn),
    api: (AppOut, AppIn),
    surb_store: S,
    chain_api: Chain,
    ticket_factory: TFact,
    counters: crate::protocol::PeerProtocolCounterRegistry,
    channels_dst: Hash,
    cfg: HoprPacketPipelineConfig,
    ticket_events: Option<TEvt>,
}

impl<WIn, WOut, Chain, S, TFact, AppOut, AppIn>
    HoprPacketPipelineBuilder<
        WIn,
        WOut,
        Chain,
        S,
        TFact,
        AppOut,
        AppIn,
        futures::sink::Drain<hopr_api::node::TicketEvent>,
    >
{
    /// Creates a new builder with the mandatory components.
    ///
    /// The configuration defaults to [`HoprPacketPipelineConfig::default`]; use
    /// [`HoprPacketPipelineBuilder::with_config`] to override it.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        keypairs: (OffchainKeypair, ChainKeypair),
        wire_msg: (WOut, WIn),
        api: (AppOut, AppIn),
        surb_store: S,
        chain_api: Chain,
        ticket_factory: TFact,
        counters: crate::protocol::PeerProtocolCounterRegistry,
        channels_dst: Hash,
    ) -> Self {
        let (packet_key, chain_key) = keypairs;
        Self {
            packet_key,
            chain_key,
            wire_msg,
            api,
            surb_store,
            chain_api,
            ticket_factory,
            counters,
            channels_dst,
            cfg: HoprPacketPipelineConfig::default(),
            ticket_events: None,
        }
    }
}

impl<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt>
    HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt>
{
    /// Overrides the default [`HoprPacketPipelineConfig`].
    pub fn with_config(mut self, cfg: HoprPacketPipelineConfig) -> Self {
        self.cfg = cfg;
        self
    }

    /// Attaches the ticket events sink. Required for Relay nodes (see
    /// [`HoprPacketPipelineBuilder::build_for_relay`]); ignored by Entry and Exit nodes.
    pub fn with_ticket_events<TEvt2>(
        self,
        ticket_events: TEvt2,
    ) -> HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt2> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg: self.wire_msg,
            api: self.api,
            surb_store: self.surb_store,
            chain_api: self.chain_api,
            ticket_factory: self.ticket_factory,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: Some(ticket_events),
        }
    }
}

// Implementation detail: codec, decoder and optional capture wiring shared by the three terminals.
impl<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt>
    HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt>
where
    WOut: futures::Sink<(PeerId, Bytes)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    WIn: futures::Stream<Item = (PeerId, Bytes)> + Send + 'static,
    Chain: ChainKeyOperations
        + ChainReadChannelOperations
        + ChainReadTicketOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
    S: SurbStore + Clone + Send + Sync + 'static,
    TFact: TicketFactory + Clone + Send + Sync + 'static,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
{
    /// Builds the codec pair (and capture wiring when enabled) and the unacknowledged ticket
    /// processor, returning them together with an [`AbortableList`] already containing the
    /// capture task if any was started.
    #[allow(clippy::type_complexity)]
    fn prepare(
        self,
    ) -> (
        OffchainKeypair,
        (WOut, WIn),
        (AppOut, AppIn),
        crate::protocol::PeerProtocolCounterRegistry,
        HoprUnacknowledgedTicketProcessor<Chain>,
        Option<TEvt>,
        HoprPacketPipelineConfig,
        // Codec parts in their final shape (possibly wrapped by capture)
        AbortableList<HoprTransportProcess>,
        BuiltCodec<Chain, S, TFact>,
    ) {
        let HoprPacketPipelineBuilder {
            packet_key,
            chain_key,
            wire_msg,
            api,
            surb_store,
            chain_api,
            ticket_factory,
            counters,
            channels_dst,
            cfg,
            ticket_events,
        } = self;

        let unack_ticket_proc = HoprUnacknowledgedTicketProcessor::new(
            chain_api.clone(),
            chain_key.clone(),
            channels_dst,
            cfg.ack_processor,
        );

        let encoder = HoprEncoder::new(
            chain_key.clone(),
            chain_api.clone(),
            surb_store.clone(),
            ticket_factory.clone(),
            channels_dst,
            cfg.codec,
        );

        let decoder = HoprDecoder::new(
            (packet_key.clone(), chain_key.clone()),
            chain_api.clone(),
            surb_store,
            ticket_factory.clone(),
            channels_dst,
            cfg.codec,
        );

        #[allow(unused_mut)]
        let mut processes = AbortableList::default();

        #[cfg(feature = "capture")]
        let codec = {
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
            BuiltCodec::Captured(
                capture::CapturePacketCodec::new(encoder, *packet_key.public(), sender.clone()),
                capture::CapturePacketCodec::new(decoder, *packet_key.public(), sender),
            )
        };

        #[cfg(not(feature = "capture"))]
        let codec = BuiltCodec::Plain(encoder, decoder);

        (
            packet_key,
            wire_msg,
            api,
            counters,
            unack_ticket_proc,
            ticket_events,
            cfg,
            processes,
            codec,
        )
    }
}

/// Internal helper to keep the codec types abstracted between the capture/no-capture builds.
enum BuiltCodec<Chain, S, TFact>
where
    Chain: ChainKeyOperations
        + ChainReadChannelOperations
        + ChainReadTicketOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
    S: SurbStore + Clone + Send + Sync + 'static,
    TFact: TicketFactory + Clone + Send + Sync + 'static,
{
    #[cfg(not(feature = "capture"))]
    Plain(HoprEncoder<Chain, S, TFact>, HoprDecoder<Chain, S, TFact>),
    #[cfg(feature = "capture")]
    Captured(
        crate::capture::CapturePacketCodec<HoprEncoder<Chain, S, TFact>>,
        crate::capture::CapturePacketCodec<HoprDecoder<Chain, S, TFact>>,
    ),
}

// Terminal: Relay
impl<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt>
    HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt>
where
    WOut: futures::Sink<(PeerId, Bytes)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    WIn: futures::Stream<Item = (PeerId, Bytes)> + Send + 'static,
    Chain: ChainKeyOperations
        + ChainReadChannelOperations
        + ChainReadTicketOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
    S: SurbStore + Clone + Send + Sync + 'static,
    TEvt: futures::Sink<hopr_api::node::TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::error::Error,
    TFact: TicketFactory + Clone + Send + Sync + 'static,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
{
    /// Builds the pipeline configured for a Relay node.
    ///
    /// # Panics
    /// Panics if [`HoprPacketPipelineBuilder::with_ticket_events`] was not called.
    pub fn build_for_relay(self) -> AbortableList<HoprTransportProcess> {
        let (packet_key, wire_msg, api, counters, unack_ticket_proc, ticket_events, _cfg, mut processes, codec) =
            self.prepare();

        let ticket_events = ticket_events.expect("Relay node requires ticket events; call with_ticket_events() first");

        let inner = match codec {
            #[cfg(not(feature = "capture"))]
            BuiltCodec::Plain(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(_cfg.pipeline)
                .with_ticket_processing(unack_ticket_proc, ticket_events)
                .build_for_relay(),
            #[cfg(feature = "capture")]
            BuiltCodec::Captured(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(_cfg.pipeline)
                .with_ticket_processing(unack_ticket_proc, ticket_events)
                .build_for_relay(),
        };

        processes.flat_map_extend_from(inner, HoprTransportProcess::Pipeline);
        processes
    }
}

// Terminal: Entry / Exit (no ticket events required)
impl<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt>
    HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, AppOut, AppIn, TEvt>
where
    WOut: futures::Sink<(PeerId, Bytes)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    WIn: futures::Stream<Item = (PeerId, Bytes)> + Send + 'static,
    Chain: ChainKeyOperations
        + ChainReadChannelOperations
        + ChainReadTicketOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
    S: SurbStore + Clone + Send + Sync + 'static,
    TFact: TicketFactory + Clone + Send + Sync + 'static,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
{
    /// Builds the pipeline configured for an Entry node.
    ///
    /// The incoming acknowledgement pipeline is not started; ticket events (if any) are ignored.
    pub fn build_for_entry(self) -> AbortableList<HoprTransportProcess> {
        let (packet_key, wire_msg, api, counters, _unack, _ticket_events, _cfg, mut processes, codec) = self.prepare();

        let inner = match codec {
            #[cfg(not(feature = "capture"))]
            BuiltCodec::Plain(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(_cfg.pipeline)
                .build_for_entry(),
            #[cfg(feature = "capture")]
            BuiltCodec::Captured(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(_cfg.pipeline)
                .build_for_entry(),
        };

        processes.flat_map_extend_from(inner, HoprTransportProcess::Pipeline);
        processes
    }

    /// Builds the pipeline configured for an Exit node.
    ///
    /// The incoming acknowledgement pipeline is started but its acknowledgements are drained
    /// (never forwarded to a ticket processor); ticket events (if any) are ignored.
    pub fn build_for_exit(self) -> AbortableList<HoprTransportProcess> {
        let (packet_key, wire_msg, api, counters, _unack, _ticket_events, _cfg, mut processes, codec) = self.prepare();

        let inner = match codec {
            #[cfg(not(feature = "capture"))]
            BuiltCodec::Plain(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(_cfg.pipeline)
                .build_for_exit(),
            #[cfg(feature = "capture")]
            BuiltCodec::Captured(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(_cfg.pipeline)
                .build_for_exit(),
        };

        processes.flat_map_extend_from(inner, HoprTransportProcess::Pipeline);
        processes
    }
}
