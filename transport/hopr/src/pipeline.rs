use bytes::Bytes;
use hopr_api::{
    chain::{ChainKeyOperations, ChainReadChannelOperations, ChainReadTicketOperations, ChainValues},
    tickets::TicketFactory,
    types::{
        crypto::prelude::*,
        internal::{prelude::*, routing::ResolvedTransportRouting},
    },
};
use hopr_crypto_packet::{HoprPixSpec, HoprShareResolution, HoprSurb};
use hopr_protocol_app::prelude::*;
use hopr_protocol_hopr::prelude::*;
use hopr_protocol_pix::{EntryShareGenerator, ExitAcknowledgementShareProcessor};
use hopr_utils::runtime::AbortableList;

use crate::{
    HoprTransportProcess, PeerProtocolCounterRegistry,
    config::HoprPacketPipelineConfig,
    protocol::{NopExitAcknowledgementShareProcessor, PacketPipelineBuilder, Unset},
};

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
/// The builder is constructed via [`HoprPacketPipelineBuilder::new`] which takes no arguments.
/// The required components must then be supplied via the corresponding builder methods:
/// [`identity`](HoprPacketPipelineBuilder::identity), [`transport`](HoprPacketPipelineBuilder::transport),
/// [`api`](HoprPacketPipelineBuilder::api), [`surb_store`](HoprPacketPipelineBuilder::surb_store),
/// [`chain_api`](HoprPacketPipelineBuilder::chain_api),
/// [`ticket_factory`](HoprPacketPipelineBuilder::ticket_factory) and
/// [`channels_dst`](HoprPacketPipelineBuilder::channels_dst).
///
/// The per-peer counter registry defaults to an empty one; override it via
/// [`HoprPacketPipelineBuilder::with_counters`].
///
/// The configuration ([`HoprPacketPipelineConfig`]) is optional and defaults to
/// `HoprPacketPipelineConfig::default()`; override it via [`HoprPacketPipelineBuilder::with_config`].
pub struct HoprPacketPipelineBuilder<
    WIn,
    WOut,
    Chain,
    S,
    TFact,
    G,
    AppOut,
    AppIn,
    TEvt = futures::sink::Drain<hopr_api::node::TicketEvent>,
    A = NopExitAcknowledgementShareProcessor,
    SEvt = futures::sink::Drain<HoprShareResolution>,
> {
    packet_key: Option<OffchainKeypair>,
    chain_key: Option<ChainKeypair>,
    wire_msg: (WOut, WIn),
    api: (AppOut, AppIn),
    surb_store: S,
    chain_api: Chain,
    ticket_factory: TFact,
    ssa_generator: G,
    counters: PeerProtocolCounterRegistry,
    channels_dst: Option<Hash>,
    cfg: HoprPacketPipelineConfig,
    ticket_events: Option<TEvt>,
    exit_ack_proc: A,
    ssa_events: SEvt,
}

impl Default
    for HoprPacketPipelineBuilder<
        Unset,
        Unset,
        Unset,
        Unset,
        Unset,
        Unset,
        Unset,
        Unset,
        futures::sink::Drain<hopr_api::node::TicketEvent>,
        NopExitAcknowledgementShareProcessor,
        futures::sink::Drain<HoprShareResolution>,
    >
{
    fn default() -> Self {
        Self::new()
    }
}

impl
    HoprPacketPipelineBuilder<
        Unset,
        Unset,
        Unset,
        Unset,
        Unset,
        Unset,
        Unset,
        Unset,
        futures::sink::Drain<hopr_api::node::TicketEvent>,
        NopExitAcknowledgementShareProcessor,
        futures::sink::Drain<HoprShareResolution>,
    >
{
    /// Creates a new empty builder. All required components must then be supplied via the
    /// corresponding builder methods before calling any of the terminal `build_for_*` methods.
    pub fn new() -> Self {
        Self {
            packet_key: None,
            chain_key: None,
            wire_msg: (Unset, Unset),
            api: (Unset, Unset),
            surb_store: Unset,
            chain_api: Unset,
            ticket_factory: Unset,
            ssa_generator: Unset,
            counters: PeerProtocolCounterRegistry::default(),
            channels_dst: None,
            cfg: HoprPacketPipelineConfig::default(),
            ticket_events: None,
            exit_ack_proc: NopExitAcknowledgementShareProcessor,
            ssa_events: futures::sink::drain(),
        }
    }
}

impl<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt>
    HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt>
{
    /// Overrides the default [`HoprPacketPipelineConfig`].
    #[must_use]
    pub fn with_config(mut self, cfg: HoprPacketPipelineConfig) -> Self {
        self.cfg = cfg;
        self
    }

    /// Overrides the default (empty) per-peer protocol counter registry.
    #[must_use]
    pub fn with_counters(mut self, counters: PeerProtocolCounterRegistry) -> Self {
        self.counters = counters;
        self
    }

    /// Sets the node identity (chain and offchain keypairs).
    #[must_use]
    pub fn identity<'a, I>(mut self, identity: I) -> Self
    where
        I: Into<(&'a ChainKeypair, &'a OffchainKeypair)>,
    {
        let (chain_key, packet_key) = identity.into();
        self.chain_key = Some(chain_key.clone());
        self.packet_key = Some(packet_key.clone());
        self
    }

    /// Sets the channel-set domain separator used by the codec and ticket processor.
    #[must_use]
    pub fn channels_dst(mut self, channels_dst: Hash) -> Self {
        self.channels_dst = Some(channels_dst);
        self
    }

    /// Sets the underlying wire-message transport (outgoing sink, incoming stream).
    #[must_use]
    pub fn transport<WIn2, WOut2>(
        self,
        wire_msg: (WOut2, WIn2),
    ) -> HoprPacketPipelineBuilder<WIn2, WOut2, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg,
            api: self.api,
            surb_store: self.surb_store,
            chain_api: self.chain_api,
            ticket_factory: self.ticket_factory,
            ssa_generator: self.ssa_generator,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: self.ticket_events,
            exit_ack_proc: self.exit_ack_proc,
            ssa_events: self.ssa_events,
        }
    }

    /// Sets the application API (incoming sink, outgoing stream).
    #[must_use]
    pub fn api<AppOut2, AppIn2>(
        self,
        api: (AppOut2, AppIn2),
    ) -> HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, G, AppOut2, AppIn2, TEvt, A, SEvt> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg: self.wire_msg,
            api,
            surb_store: self.surb_store,
            chain_api: self.chain_api,
            ticket_factory: self.ticket_factory,
            ssa_generator: self.ssa_generator,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: self.ticket_events,
            exit_ack_proc: self.exit_ack_proc,
            ssa_events: self.ssa_events,
        }
    }

    /// Sets the SURB store used by the encoder/decoder.
    #[must_use]
    pub fn surb_store<S2>(
        self,
        surb_store: S2,
    ) -> HoprPacketPipelineBuilder<WIn, WOut, Chain, S2, TFact, G, AppOut, AppIn, TEvt, A, SEvt> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg: self.wire_msg,
            api: self.api,
            surb_store,
            chain_api: self.chain_api,
            ticket_factory: self.ticket_factory,
            ssa_generator: self.ssa_generator,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: self.ticket_events,
            exit_ack_proc: self.exit_ack_proc,
            ssa_events: self.ssa_events,
        }
    }

    /// Sets the chain API used by the encoder/decoder and the unacknowledged ticket processor.
    #[must_use]
    pub fn chain_api<Chain2>(
        self,
        chain_api: Chain2,
    ) -> HoprPacketPipelineBuilder<WIn, WOut, Chain2, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg: self.wire_msg,
            api: self.api,
            surb_store: self.surb_store,
            chain_api,
            ticket_factory: self.ticket_factory,
            ssa_generator: self.ssa_generator,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: self.ticket_events,
            exit_ack_proc: self.exit_ack_proc,
            ssa_events: self.ssa_events,
        }
    }

    /// Sets the ticket factory used by the encoder/decoder.
    #[must_use]
    pub fn ticket_factory<TFact2>(
        self,
        ticket_factory: TFact2,
    ) -> HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact2, G, AppOut, AppIn, TEvt, A, SEvt> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg: self.wire_msg,
            api: self.api,
            surb_store: self.surb_store,
            chain_api: self.chain_api,
            ticket_factory,
            ssa_generator: self.ssa_generator,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: self.ticket_events,
            exit_ack_proc: self.exit_ack_proc,
            ssa_events: self.ssa_events,
        }
    }

    /// Sets the SSA share generator used by the encoder.
    #[must_use]
    pub fn ssa_generator<G2>(
        self,
        ssa_generator: G2,
    ) -> HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, G2, AppOut, AppIn, TEvt, A, SEvt> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg: self.wire_msg,
            api: self.api,
            surb_store: self.surb_store,
            chain_api: self.chain_api,
            ticket_factory: self.ticket_factory,
            ssa_generator,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: self.ticket_events,
            exit_ack_proc: self.exit_ack_proc,
            ssa_events: self.ssa_events,
        }
    }

    /// Attaches the ticket events sink. Required for Relay nodes (see
    /// [`HoprPacketPipelineBuilder::build_for_relay`]); ignored by Entry and Exit nodes.
    #[must_use]
    pub fn with_ticket_events<TEvt2>(
        self,
        ticket_events: TEvt2,
    ) -> HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt2, A, SEvt> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg: self.wire_msg,
            api: self.api,
            surb_store: self.surb_store,
            chain_api: self.chain_api,
            ticket_factory: self.ticket_factory,
            ssa_generator: self.ssa_generator,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: Some(ticket_events),
            exit_ack_proc: self.exit_ack_proc,
            ssa_events: self.ssa_events,
        }
    }

    /// Attaches an exit-acknowledgement share processor and a recovered-SSA events sink to the
    /// builder. Used by Relay and Exit nodes (see
    /// [`HoprPacketPipelineBuilder::build_for_relay`] and
    /// [`HoprPacketPipelineBuilder::build_for_exit`]); ignored by Entry nodes.
    ///
    /// Optional: when not called, the builder defaults to [`NopExitAcknowledgementShareProcessor`]
    /// and a draining sink ([`futures::sink::drain`]).
    #[must_use]
    pub fn with_exit_ack_share_processing<A2, SEvt2>(
        self,
        exit_ack_proc: A2,
        ssa_events: SEvt2,
    ) -> HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A2, SEvt2> {
        HoprPacketPipelineBuilder {
            packet_key: self.packet_key,
            chain_key: self.chain_key,
            wire_msg: self.wire_msg,
            api: self.api,
            surb_store: self.surb_store,
            chain_api: self.chain_api,
            ticket_factory: self.ticket_factory,
            ssa_generator: self.ssa_generator,
            counters: self.counters,
            channels_dst: self.channels_dst,
            cfg: self.cfg,
            ticket_events: self.ticket_events,
            exit_ack_proc,
            ssa_events,
        }
    }
}

// Implementation detail: codec, decoder and optional capture wiring shared by the three terminals.
impl<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt>
    HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt>
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
    G: EntryShareGenerator<HoprPixSpec> + Clone + Send + Sync + 'static,
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
        PeerProtocolCounterRegistry,
        HoprUnacknowledgedTicketProcessor<Chain>,
        Option<TEvt>,
        A,
        SEvt,
        HoprPacketPipelineConfig,
        // Codec parts in their final shape (possibly wrapped by capture)
        AbortableList<HoprTransportProcess>,
        BuiltCodec<Chain, G, S, TFact>,
    ) {
        let HoprPacketPipelineBuilder {
            packet_key,
            chain_key,
            wire_msg,
            api,
            surb_store,
            chain_api,
            ticket_factory,
            ssa_generator,
            counters,
            channels_dst,
            cfg,
            ticket_events,
            exit_ack_proc,
            ssa_events,
        } = self;

        let packet_key = packet_key.expect("identity() must be called before building the pipeline");
        let chain_key = chain_key.expect("identity() must be called before building the pipeline");
        let channels_dst = channels_dst.expect("channels_dst() must be called before building the pipeline");

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
            ssa_generator,
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
            exit_ack_proc,
            ssa_events,
            cfg,
            processes,
            codec,
        )
    }
}

/// Internal helper to keep the codec types abstracted between the capture/no-capture builds.
enum BuiltCodec<Chain, G, S, TFact>
where
    Chain: ChainKeyOperations
        + ChainReadChannelOperations
        + ChainReadTicketOperations
        + ChainValues
        + Clone
        + Send
        + Sync
        + 'static,
    G: EntryShareGenerator<HoprPixSpec> + Clone + Send + Sync + 'static,
    S: SurbStore + Clone + Send + Sync + 'static,
    TFact: TicketFactory + Clone + Send + Sync + 'static,
{
    #[cfg(not(feature = "capture"))]
    Plain(HoprEncoder<Chain, G, S, TFact>, HoprDecoder<Chain, S, TFact>),
    #[cfg(feature = "capture")]
    Captured(
        crate::capture::CapturePacketCodec<HoprEncoder<Chain, G, S, TFact>>,
        crate::capture::CapturePacketCodec<HoprDecoder<Chain, S, TFact>>,
    ),
}

// Terminal: Relay
impl<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt>
    HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt>
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
    G: EntryShareGenerator<HoprPixSpec> + Clone + Send + Sync + 'static,
    A: ExitAcknowledgementShareProcessor<HoprPixSpec> + Send + Sync + 'static,
    SEvt: futures::Sink<HoprShareResolution> + Clone + Unpin + Send + 'static,
    SEvt::Error: std::error::Error,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
{
    /// Builds the pipeline configured for a Relay node.
    ///
    /// # Panics
    /// Panics if [`HoprPacketPipelineBuilder::with_ticket_events`] was not called.
    pub fn build_for_relay(self) -> AbortableList<HoprTransportProcess> {
        let (
            packet_key,
            wire_msg,
            api,
            counters,
            unack_ticket_proc,
            ticket_events,
            exit_ack_proc,
            ssa_events,
            _cfg,
            mut processes,
            codec,
        ) = self.prepare();

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
                .with_exit_ack_share_processing(exit_ack_proc, ssa_events)
                .build_for_relay(),
            #[cfg(feature = "capture")]
            BuiltCodec::Captured(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(_cfg.pipeline)
                .with_ticket_processing(unack_ticket_proc, ticket_events)
                .with_exit_ack_share_processing(exit_ack_proc, ssa_events)
                .build_for_relay(),
        };

        processes.flat_map_extend_from(inner, HoprTransportProcess::Pipeline);
        processes
    }
}

// Terminal: Entry / Exit (no ticket events required)
impl<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt>
    HoprPacketPipelineBuilder<WIn, WOut, Chain, S, TFact, G, AppOut, AppIn, TEvt, A, SEvt>
where
    A: ExitAcknowledgementShareProcessor<HoprPixSpec> + Send + Sync + 'static,
    SEvt: futures::Sink<HoprShareResolution> + Clone + Unpin + Send + 'static,
    SEvt::Error: std::error::Error,
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
    G: EntryShareGenerator<HoprPixSpec> + Clone + Send + Sync + 'static,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
{
    /// Builds the pipeline configured for an Entry node.
    ///
    /// The incoming acknowledgement pipeline is not started; ticket events (if any) are ignored.
    pub fn build_for_entry(self) -> AbortableList<HoprTransportProcess> {
        let (
            packet_key,
            wire_msg,
            api,
            counters,
            _unack,
            _ticket_events,
            _exit_ack_proc,
            _ssa_events,
            cfg,
            mut processes,
            codec,
        ) = self.prepare();

        let inner = match codec {
            #[cfg(not(feature = "capture"))]
            BuiltCodec::Plain(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(cfg.pipeline)
                .build_for_entry(),
            #[cfg(feature = "capture")]
            BuiltCodec::Captured(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(cfg.pipeline)
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
        let (
            packet_key,
            wire_msg,
            api,
            counters,
            _unack,
            _ticket_events,
            exit_ack_proc,
            ssa_events,
            cfg,
            mut processes,
            codec,
        ) = self.prepare();

        let inner = match codec {
            #[cfg(not(feature = "capture"))]
            BuiltCodec::Plain(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(cfg.pipeline)
                .with_exit_ack_share_processing(exit_ack_proc, ssa_events)
                .build_for_exit(),
            #[cfg(feature = "capture")]
            BuiltCodec::Captured(encoder, decoder) => PacketPipelineBuilder::new(packet_key.clone())
                .transport(wire_msg)
                .codec((encoder, decoder))
                .api(api)
                .with_counters(counters)
                .with_config(cfg.pipeline)
                .with_exit_ack_share_processing(exit_ack_proc, ssa_events)
                .build_for_exit(),
        };

        processes.flat_map_extend_from(inner, HoprTransportProcess::Pipeline);
        processes
    }
}
