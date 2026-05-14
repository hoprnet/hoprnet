//! Builder for constructing the HOPR packet pipeline.

use bytes::Bytes;
use hopr_api::{
    PeerId,
    node::TicketEvent,
    types::{crypto::prelude::*, internal::prelude::*},
};
use hopr_crypto_packet::HoprSurb;
use hopr_protocol_app::prelude::*;
use hopr_protocol_hopr::prelude::*;
use hopr_utils::runtime::AbortableList;

use super::{NodeType, NoopTicketProcessor, PacketPipelineProcesses, config::PacketPipelineConfig, run_packet_pipeline_inner, NopExitAcknowledgementShareProcessor};
use crate::PeerProtocolCounterRegistry;

/// Placeholder type used by [`PacketPipelineBuilder`] for generic parameters that have
/// not yet been provided via the corresponding builder method.
pub struct Unset;

/// Builder for constructing the HOPR packet pipeline for a specific node type.
///
/// The builder is constructed from a packet key via [`PacketPipelineBuilder::new`]; the
/// transport, codec and application API must then be provided via the
/// [`PacketPipelineBuilder::transport`], [`PacketPipelineBuilder::codec`] and
/// [`PacketPipelineBuilder::api`] builder methods.
///
/// Terminal methods for each node type are then exposed:
/// - [`PacketPipelineBuilder::build_for_relay`] — full pipeline, requires ticket processing via
///   [`PacketPipelineBuilder::with_ticket_processing`].
/// - [`PacketPipelineBuilder::build_for_entry`] — Entry nodes; the incoming acknowledgement pipeline is not started at
///   all.
/// - [`PacketPipelineBuilder::build_for_exit`] — Exit nodes; the incoming acknowledgement pipeline is started but only
///   drains the stream (no ticket processing).
///
/// The pipeline does not handle mixing itself; it needs to be injected as a separate process
/// overlay on top of the `wire_msg` Stream or Sink.
pub struct PacketPipelineBuilder<WIn, WOut, C, D, T, TEvt, AppOut, AppIn> {
    packet_key: OffchainKeypair,
    wire_msg: (WOut, WIn),
    codec: (C, D),
    cfg: PacketPipelineConfig,
    api: (AppOut, AppIn),
    counters: PeerProtocolCounterRegistry,
    ticket_proc: Option<T>,
    ticket_events: Option<TEvt>,
}

impl
    PacketPipelineBuilder<
        Unset,
        Unset,
        Unset,
        Unset,
        NoopTicketProcessor,
        futures::sink::Drain<TicketEvent>,
        Unset,
        Unset,
    >
{
    /// Creates a new builder with the common parameters shared by all node types.
    ///
    /// The transport, codec and application API must be supplied via
    /// [`PacketPipelineBuilder::transport`], [`PacketPipelineBuilder::codec`] and
    /// [`PacketPipelineBuilder::api`] before any of the terminal `build_for_*` methods can
    /// be called.
    ///
    /// The pipeline configuration defaults to [`PacketPipelineConfig::default`]; use
    /// [`PacketPipelineBuilder::with_config`] to override it. The per-peer counter registry
    /// defaults to an empty one; use [`PacketPipelineBuilder::with_counters`] to override it.
    ///
    /// Use [`PacketPipelineBuilder::with_ticket_processing`] to attach ticket processing
    /// before calling [`PacketPipelineBuilder::build_for_relay`].
    pub fn new(packet_key: OffchainKeypair) -> Self {
        Self {
            packet_key,
            wire_msg: (Unset, Unset),
            codec: (Unset, Unset),
            cfg: PacketPipelineConfig::default(),
            api: (Unset, Unset),
            counters: PeerProtocolCounterRegistry::default(),
            ticket_proc: None,
            ticket_events: None,
        }
    }
}

impl<WIn, WOut, C, D, T, TEvt, AppOut, AppIn> PacketPipelineBuilder<WIn, WOut, C, D, T, TEvt, AppOut, AppIn> {
    /// Overrides the default [`PacketPipelineConfig`].
    #[must_use]
    pub fn with_config(mut self, cfg: PacketPipelineConfig) -> Self {
        self.cfg = cfg;
        self
    }

    /// Overrides the default (empty) per-peer protocol counter registry.
    #[must_use]
    pub fn with_counters(mut self, counters: PeerProtocolCounterRegistry) -> Self {
        self.counters = counters;
        self
    }

    /// Sets the underlying wire-message transport (outgoing sink, incoming stream).
    #[must_use]
    pub fn transport<WIn2, WOut2>(
        self,
        wire_msg: (WOut2, WIn2),
    ) -> PacketPipelineBuilder<WIn2, WOut2, C, D, T, TEvt, AppOut, AppIn> {
        PacketPipelineBuilder {
            packet_key: self.packet_key,
            wire_msg,
            codec: self.codec,
            cfg: self.cfg,
            api: self.api,
            counters: self.counters,
            ticket_proc: self.ticket_proc,
            ticket_events: self.ticket_events,
        }
    }

    /// Sets the packet codec (encoder, decoder).
    #[must_use]
    pub fn codec<C2, D2>(self, codec: (C2, D2)) -> PacketPipelineBuilder<WIn, WOut, C2, D2, T, TEvt, AppOut, AppIn> {
        PacketPipelineBuilder {
            packet_key: self.packet_key,
            wire_msg: self.wire_msg,
            codec,
            cfg: self.cfg,
            api: self.api,
            counters: self.counters,
            ticket_proc: self.ticket_proc,
            ticket_events: self.ticket_events,
        }
    }

    /// Sets the application API (outgoing sink for received data, incoming stream for data to send).
    #[must_use]
    pub fn api<AppOut2, AppIn2>(
        self,
        api: (AppOut2, AppIn2),
    ) -> PacketPipelineBuilder<WIn, WOut, C, D, T, TEvt, AppOut2, AppIn2> {
        PacketPipelineBuilder {
            packet_key: self.packet_key,
            wire_msg: self.wire_msg,
            codec: self.codec,
            cfg: self.cfg,
            api,
            counters: self.counters,
            ticket_proc: self.ticket_proc,
            ticket_events: self.ticket_events,
        }
    }

    /// Attaches a ticket processor and a ticket-events sink to the builder.
    ///
    /// Required before calling [`PacketPipelineBuilder::build_for_relay`]. Has no effect on
    /// Entry/Exit builds, which never process tickets.
    #[must_use]
    pub fn with_ticket_processing<T2, TEvt2>(
        self,
        ticket_proc: T2,
        ticket_events: TEvt2,
    ) -> PacketPipelineBuilder<WIn, WOut, C, D, T2, TEvt2, AppOut, AppIn>
    where
        T2: UnacknowledgedTicketProcessor + Sync + Send + 'static,
        TEvt2: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
        TEvt2::Error: std::error::Error,
    {
        PacketPipelineBuilder {
            packet_key: self.packet_key,
            wire_msg: self.wire_msg,
            codec: self.codec,
            cfg: self.cfg,
            api: self.api,
            counters: self.counters,
            ticket_proc: Some(ticket_proc),
            ticket_events: Some(ticket_events),
        }
    }
}

impl<WIn, WOut, C, D, T, TEvt, AppOut, AppIn> PacketPipelineBuilder<WIn, WOut, C, D, T, TEvt, AppOut, AppIn>
where
    WOut: futures::Sink<(PeerId, Bytes)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    WIn: futures::Stream<Item = (PeerId, Bytes)> + Send + 'static,
    C: PacketEncoder + Sync + Send + 'static,
    D: PacketDecoder + Sync + Send + 'static,
    T: UnacknowledgedTicketProcessor + Sync + Send + 'static,
    TEvt: futures::Sink<TicketEvent> + Clone + Unpin + Send + 'static,
    TEvt::Error: std::error::Error,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
{
    /// Builds and starts the full packet pipeline for a HOPR **Relay** node.
    ///
    /// Relay nodes run the full pipeline: outgoing/incoming messages, outgoing acknowledgements,
    /// and incoming acknowledgements (with ticket processing).
    ///
    /// # Panics
    ///
    /// Panics if [`PacketPipelineBuilder::with_ticket_processing`] was not called before this method.
    #[must_use]
    pub fn build_for_relay(self) -> AbortableList<PacketPipelineProcesses> {
        let ticket_proc = self
            .ticket_proc
            .expect("Relay node requires ticket processing; call with_ticket_processing() first");
        let ticket_events = self
            .ticket_events
            .expect("Relay node requires ticket processing; call with_ticket_processing() first");
        run_packet_pipeline_inner(
            NodeType::Relay,
            self.packet_key,
            self.wire_msg,
            self.codec,
            ticket_proc,
            NopExitAcknowledgementShareProcessor, // TODO: must be given
            ticket_events,
            futures::sink::drain(), // TODO: must be given
            self.cfg,
            self.api,
            self.counters,
        )
    }
}

impl<WIn, WOut, C, D, T, TEvt, AppOut, AppIn> PacketPipelineBuilder<WIn, WOut, C, D, T, TEvt, AppOut, AppIn>
where
    WOut: futures::Sink<(PeerId, Bytes)> + Clone + Unpin + Send + 'static,
    WOut::Error: std::error::Error,
    WIn: futures::Stream<Item = (PeerId, Bytes)> + Send + 'static,
    C: PacketEncoder + Sync + Send + 'static,
    D: PacketDecoder + Sync + Send + 'static,
    AppOut: futures::Sink<(HoprPseudonym, ApplicationDataIn)> + Send + 'static,
    AppOut::Error: std::error::Error,
    AppIn: futures::Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> + Send + 'static,
{
    /// Builds and starts the packet pipeline for a HOPR **Entry** node.
    ///
    /// Entry nodes never relay packets and therefore do not process tickets. As a consequence,
    /// the incoming acknowledgement pipeline is **not** started.
    /// Any ticket processor or ticket events sink previously set via
    /// [`PacketPipelineBuilder::with_ticket_processing`] is ignored.
    #[must_use]
    pub fn build_for_entry(self) -> AbortableList<PacketPipelineProcesses> {
        run_packet_pipeline_inner(
            NodeType::Entry,
            self.packet_key,
            self.wire_msg,
            self.codec,
            NoopTicketProcessor,
            NopExitAcknowledgementShareProcessor,
            futures::sink::drain(),
            futures::sink::drain(),
            self.cfg,
            self.api,
            self.counters,
        )
    }

    /// Builds and starts the packet pipeline for a HOPR **Exit** node.
    ///
    /// Exit nodes do not process tickets either. However, in contrast to
    /// [`PacketPipelineBuilder::build_for_entry`], the incoming acknowledgement pipeline is kept
    /// running (it only drains the stream) to support future development.
    /// Any ticket processor or ticket events sink previously set via
    /// [`PacketPipelineBuilder::with_ticket_processing`] is ignored.
    #[must_use]
    pub fn build_for_exit(self) -> AbortableList<PacketPipelineProcesses> {
        run_packet_pipeline_inner(
            NodeType::Exit,
            self.packet_key,
            self.wire_msg,
            self.codec,
            NoopTicketProcessor,
            NopExitAcknowledgementShareProcessor, // TODO: must be given
            futures::sink::drain(),
            futures::sink::drain(), // TODO: must be given
            self.cfg,
            self.api,
            self.counters,
        )
    }
}
