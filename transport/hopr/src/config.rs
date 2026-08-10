use std::{
    fmt::{Display, Formatter},
    net::ToSocketAddrs,
    num::ParseIntError,
    str::FromStr,
    time::Duration,
};

use hopr_api::Multiaddr;
pub use hopr_protocol_hopr::{HoprCodecConfig, HoprUnacknowledgedTicketProcessorConfig, SurbStoreConfig};
use hopr_protocol_pix::SsaReconstructorConfig;
pub use hopr_transport_mixer::config::MixerConfig;
pub use hopr_transport_probe::config::ProbeConfig;
use hopr_transport_session::{
    DEFAULT_MAX_SSAS_PER_SSA_REQUEST, DEFAULT_PIX_POLYS_PER_SSA, DEFAULT_PIX_SHARES_PER_POLY,
    DEFAULT_PIX_SURPLUS_SHARES, IncomingSessionPixConfig, MAX_SSA_BATCH_SIZE, MIN_BALANCER_SAMPLING_INTERVAL,
    MIN_SURB_BUFFER_DURATION,
};
use proc_macro_regex::regex;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::{errors::HoprTransportError, protocol::PacketPipelineConfig};

const DEFAULT_COUNTER_FLUSH_INTERVAL: Duration = Duration::from_secs(15);

const DEFAULT_PER_PEER_CHANNEL_CAPACITY: usize = 5_000;
const DEFAULT_STREAM_OPEN_TIMEOUT: Duration = Duration::from_secs(2);
const DEFAULT_FRAME_WRITER_BACKPRESSURE_BYTES: usize = 131_072;
const DEFAULT_EGRESS_BACKPRESSURE_TIMEOUT: Duration = Duration::from_secs(2);

/// Minimum accepted value for [`StreamProtocolConfig::stream_open_timeout`].
pub const MIN_STREAM_OPEN_TIMEOUT: Duration = Duration::from_millis(1);

/// Minimum accepted value for [`StreamProtocolConfig::egress_backpressure_timeout`].
pub const MIN_EGRESS_BACKPRESSURE_TIMEOUT: Duration = Duration::from_millis(1);

fn default_per_peer_channel_capacity() -> usize {
    DEFAULT_PER_PEER_CHANNEL_CAPACITY
}

fn default_stream_open_timeout() -> Duration {
    DEFAULT_STREAM_OPEN_TIMEOUT
}

fn default_frame_writer_backpressure_bytes() -> usize {
    DEFAULT_FRAME_WRITER_BACKPRESSURE_BYTES
}

#[inline]
fn default_egress_backpressure_timeout() -> Duration {
    DEFAULT_EGRESS_BACKPRESSURE_TIMEOUT
}

fn validate_stream_open_timeout(value: &Duration) -> Result<(), ValidationError> {
    if MIN_STREAM_OPEN_TIMEOUT <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("stream open timeout must be at least 1 ms"))
    }
}

fn validate_egress_backpressure_timeout(value: &Duration) -> Result<(), ValidationError> {
    if MIN_EGRESS_BACKPRESSURE_TIMEOUT <= *value {
        Ok(())
    } else {
        // A zero (or sub-millisecond) timeout would make every full channel fall straight into
        // drop-newest, silently defeating the backpressure feature — reject it at config time.
        Err(ValidationError::new(
            "egress backpressure timeout must be at least 1 ms",
        ))
    }
}

/// Configuration of the per-peer egress stream layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Validate, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct StreamProtocolConfig {
    /// Capacity of the per-peer egress channel (in packets).
    ///
    /// The egress drain enqueues each outgoing packet via `try_send`. When the
    /// channel is full the behaviour depends on the stream state: while the stream
    /// is still opening it drops the newest packet (a slow open for one peer must
    /// not head-of-line-block others); once the stream is open and its write pump
    /// is draining, it instead applies bounded backpressure — waiting up to
    /// `EGRESS_BACKPRESSURE_TIMEOUT` for space so wire-rate backpressure propagates
    /// upstream — and only drops the newest packet if the peer stays full past that
    /// timeout. The channel absorbs bursts while a stream is being opened; once open
    /// the write pump continuously drains it, so it stays near-empty under normal load.
    ///
    /// Sized to absorb a typical SURB pre-fill burst (default SurbBalancer:
    /// target 7 000 / max 5 000/s).
    ///
    /// Defaults to 5 000.
    #[validate(range(min = 1))]
    #[default(default_per_peer_channel_capacity())]
    #[cfg_attr(feature = "serde", serde(default = "default_per_peer_channel_capacity"))]
    pub per_peer_channel_capacity: usize,

    /// Timeout for the `NetworkStreamControl::open` call when opening a new
    /// outgoing stream to a peer.
    ///
    /// A timeout is mandatory: without it a permanently-unreachable peer would park
    /// the opener task indefinitely. When the open attempt fails or times out the
    /// buffered packets for that peer are dropped and a debug-level log entry is
    /// emitted. The cache entry is then invalidated so the next send triggers a
    /// fresh open attempt.
    ///
    /// Must be at least 1 ms. Defaults to 2 seconds.
    #[validate(custom(function = "validate_stream_open_timeout"))]
    #[default(default_stream_open_timeout())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_stream_open_timeout", with = "humantime_serde")
    )]
    pub stream_open_timeout: Duration,

    /// Pending-write-buffer byte threshold on the framed writer before a flush is forced.
    ///
    /// A value of `1` flushes on every encoded frame (one syscall per message).
    /// Larger values coalesce adjacent small frames into a single quinn write call,
    /// reducing connection-mutex acquisitions and driver wake-ups on the hot path.
    /// A HOPR packet is ~1 440 bytes; at the default 128 KiB threshold roughly 91
    /// packets are coalesced per write, cutting driver wake frequency ~30×.
    ///
    /// Defaults to 131 072 bytes (128 KiB).
    #[validate(range(min = 1))]
    #[default(default_frame_writer_backpressure_bytes())]
    #[cfg_attr(feature = "serde", serde(default = "default_frame_writer_backpressure_bytes"))]
    pub frame_writer_backpressure_bytes: usize,

    /// Maximum time the egress drain waits on a full — but open and draining — per-peer channel
    /// before falling back to drop-newest.
    ///
    /// While the stream is open, a full channel means the wire is slower than the producer, so
    /// waiting here propagates wire-rate backpressure up through the mixer and session socket to the
    /// application writer (no packet loss). The bound ensures a single permanently-stalled peer cannot
    /// head-of-line-block delivery to other peers indefinitely: after this timeout the packet is
    /// dropped and the drain moves on. Healthy peers drain far faster than this, so the timeout is not
    /// hit in normal operation.
    ///
    /// Defaults to 2 seconds. Must be at least 1 ms — a zero value would defeat the feature.
    #[validate(custom(function = "validate_egress_backpressure_timeout"))]
    #[default(default_egress_backpressure_timeout())]
    #[cfg_attr(feature = "serde", serde(default = "default_egress_backpressure_timeout"))]
    pub egress_backpressure_timeout: Duration,
}

fn default_counter_flush_interval() -> Duration {
    DEFAULT_COUNTER_FLUSH_INTERVAL
}

/// Simulated per-packet transit latency inserted between the mixer and the wire.
///
/// When set on a node's config, every packet emitted by the mixer is held for a
/// Gaussian-jittered delay before being forwarded to the transport layer.  The delay
/// is **FIFO** (packets are never reordered): the release deadline is `max(prev_deadline,
/// now) + sample`, so back-to-back bursts accumulate a monotonically non-decreasing
/// offset rather than reordering.
///
/// **Intended for testing only** — simulates WAN-link transit latency (e.g. ~50 ms) in
/// a local cluster.  Defaults to `None` (disabled; zero production overhead).
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct TransitLatencyConfig {
    /// Mean transit latency per packet.
    #[default(Duration::from_millis(50))]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub mean: Duration,
    /// Standard deviation of the transit latency.
    ///
    /// Set to zero for a deterministic (fixed) delay equal to `mean`.
    #[default(Duration::from_millis(5))]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub std_dev: Duration,
}

/// Complete configuration of the HOPR protocol stack.
#[derive(Debug, smart_default::SmartDefault, Validate, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HoprProtocolConfig {
    /// Libp2p-related transport configuration
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub transport: TransportConfig,
    /// HOPR packet pipeline configuration
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub packet: HoprPacketPipelineConfig,
    /// Probing protocol configuration
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub probe: ProbeConfig,
    /// Session protocol global configuration
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub session: SessionGlobalConfig,
    /// Global configuration for the PIX.
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub pix: PixGlobalConfig,
    /// Per-node PIX session configuration for incoming sessions.
    #[validate(custom(function = "validate_incoming_session_pix_config"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub incoming_session_pix_config: IncomingSessionPixConfig,
    /// Mixer configuration.
    #[cfg_attr(feature = "serde", serde(default))]
    pub mixer: MixerConfig,
    /// Simulated transit latency shim between the mixer output and the wire.
    ///
    /// When `Some`, a Gaussian-jittered FIFO delay is inserted before every forwarded
    /// packet — simulating WAN-link transit time in a local cluster test run.
    /// Set `None` (the default) in production: zero overhead.
    #[cfg_attr(feature = "serde", serde(default))]
    pub transit_latency: Option<TransitLatencyConfig>,
    /// Per-peer egress stream configuration
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub stream: StreamProtocolConfig,
    /// Path planner configuration
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub path_planner: crate::path::PathPlannerConfig,
    /// Interval at which per-peer protocol conformance counters are flushed
    /// into the network graph.
    ///
    /// Default is 15 seconds.
    #[default(default_counter_flush_interval())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_counter_flush_interval", with = "humantime_serde")
    )]
    pub counter_flush_interval: Duration,
}

/// Rejects an [`IncomingSessionPixConfig`] whose acceptance range can never match anything, or whose
/// SSA batch size is outside what the protocol supports.
///
/// `quota_range` is operator-settable, and an empty (inverted) range silently makes
/// `check_pix_params` reject every offered PIX parameter set, which surfaces only as
/// `UnacceptablePixParams` errors at Session establishment time.
///
/// `ssas_per_request` is checked here rather than with a `range` attribute because
/// `IncomingSessionPixConfig` lives in `hopr-transport-session` and carries no `Validate` derive of
/// its own. Zero would mean no `SsaRequest` is ever sent, and above [`MAX_SSA_BATCH_SIZE`] the
/// per-cycle reconstructor state and the Start protocol channel pre-allocation both grow past what
/// that ceiling exists to bound. `SessionManager::new` clamps rather than trusting this check, since
/// nothing forces a programmatically built config through it.
fn validate_incoming_session_pix_config(cfg: &IncomingSessionPixConfig) -> Result<(), ValidationError> {
    if cfg.quota_range.is_empty() {
        return Err(ValidationError::new(
            "pix quota_range must be non-empty (start must not exceed end)",
        ));
    }
    if !(1..=MAX_SSA_BATCH_SIZE).contains(&cfg.ssas_per_request) {
        return Err(ValidationError::new(
            "pix ssas_per_request must be between 1 and MAX_SSA_BATCH_SIZE",
        ));
    }
    Ok(())
}

/// Headroom over the profiled dimension product that [`PixGlobalConfig`] will accept.
///
/// See [`validate_pix_dimension_product`] for why the ceiling is on the product and why this
/// multiple does not have to move when the polynomial/threshold split is re-tuned.
const MAX_PIX_DIMENSION_PRODUCT_FACTOR: usize = 4;

/// Rejects dimensions whose *product* is far outside anything that has been measured.
///
/// `num_ssa_parts` and `ssa_part_size` are range-validated independently, and their ranges permit
/// 16192 × 255 = 4 128 960 commitments, about 8× the profiled operating point of 8192 × 64 =
/// 524 288 (≈49 MiB of peak reconstructor state and ≈1.25 s of commitment ingest per cycle). Nothing
/// downstream catches that: the product *is* the per-cycle quota, and the only guard on it is the
/// peer Exit's `quota_range` rejection — which protects the Exit, and arrives after this node has
/// already generated the cycle.
///
/// The ceiling is deliberately on the product rather than on either field, and that is what makes it
/// stable: re-tuning the split holds the product constant — 4096 × 128 and 8192 × 64 are both
/// exactly 524 288, which is why the derived `quota_range` survived that change untouched. Only a
/// deliberate decision to raise the per-cycle quota needs to revisit this, and such a decision has to
/// widen the Exit's `quota_range` in concert regardless.
///
/// It binds less hard than it once did: `ssa_part_size` was capped at 4096 before the threshold was
/// narrowed to a byte so it could share the negotiated `PixParams` word with the surplus, which took
/// the field-range maximum product down from 126× the profiled point to under 8×. It still binds
/// over most of the two ranges, which is the intended effect.
fn validate_pix_dimension_product(cfg: &PixGlobalConfig) -> Result<(), ValidationError> {
    const PROFILED: usize = DEFAULT_PIX_POLYS_PER_SSA as usize * DEFAULT_PIX_SHARES_PER_POLY as usize;

    if cfg.num_ssa_parts.saturating_mul(cfg.ssa_part_size) > MAX_PIX_DIMENSION_PRODUCT_FACTOR * PROFILED {
        return Err(ValidationError::new(
            "num_ssa_parts * ssa_part_size exceeds the supported per-cycle dimension product",
        ));
    }
    Ok(())
}

/// Global configuration for the Protocol for Incentivization of eXits (PIX).
#[derive(Clone, Copy, Debug, PartialEq, Validate, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
#[validate(schema(function = "validate_pix_dimension_product", skip_on_field_errors = false))]
pub struct PixGlobalConfig {
    /// Number of parts an SSA is split into.
    ///
    /// This scales will with the CPU parallelism.
    ///
    /// Defaults to [`DEFAULT_PIX_POLYS_PER_SSA`], which is also what
    /// [`IncomingSessionPixConfig::quota_range`] is derived from — changing this without
    /// widening the peer Exit's `quota_range` accordingly will get the Session rejected.
    ///
    /// The range below bounds this field alone. What actually costs is the *product* with
    /// [`ssa_part_size`](Self::ssa_part_size), which validation bounds separately at 4× the profiled
    /// operating point — see `validate_pix_dimension_product`.
    #[validate(range(min = 8, max = 16192))]
    #[default(DEFAULT_PIX_POLYS_PER_SSA as usize)]
    pub num_ssa_parts: usize,

    /// Number of shares required to reconstruct an SSA part.
    ///
    /// This does not scale well with CPU parallelism.
    ///
    /// Defaults to [`DEFAULT_PIX_SHARES_PER_POLY`]. See [`num_ssa_parts`](Self::num_ssa_parts)
    /// for the interaction with the Exit's accepted quota range, and
    /// `validate_pix_dimension_product` for the bound on the two together.
    /// Capped at 255 because the threshold is one byte of the negotiated
    /// [`PixParams`](hopr_protocol_pix::PixParams) word — see
    /// [`MAX_POLY_THRESHOLD`](hopr_protocol_pix::MAX_POLY_THRESHOLD).
    #[validate(range(min = 2, max = 255))]
    #[default(DEFAULT_PIX_SHARES_PER_POLY as usize)]
    pub ssa_part_size: usize,

    /// Number of shares sent in addition to `ssa_part_size` to reconstruct an SSA part.
    ///
    /// This is used to account for potential packet loss but makes it take longer for the
    /// other side to reconstruct the entire SSA from all its parts. This is because if
    /// no packet loss is present, the other side can reconstruct the SSA from fewer shares.
    ///
    /// **This is an absolute share count, not a ratio.** The default is half of
    /// [`DEFAULT_PIX_SHARES_PER_POLY`] — a surplus factor of 1.5× — but it is a constant computed
    /// once, so raising [`ssa_part_size`](Self::ssa_part_size) and leaving this alone lowers the
    /// surplus factor, and lowering it raises it. Re-tune the two together.
    ///
    /// The factor is what matters, because it is what this costs. A polynomial leaves the
    /// generator's queue at `ssa_part_size + additional_shares` shares whether or not any were
    /// lost, so this is service the Exit performs in every case — and since the surplus travels to
    /// the peer as part of the negotiated [`PixParams`](hopr_protocol_pix::PixParams), the per-SSA
    /// quota counts it and the deposit pays for it. It buys loss tolerance, and it is charged for
    /// like any other insurance: on purchase, not on claim.
    ///
    /// Raising it therefore costs money rather than earning free service, which is the way round it
    /// should be. It used to be the other way: the surplus was excluded from the quota, so the
    /// rational Entry raised this dial to take traffic it was not billed for.
    ///
    /// Capped at 255 because it is the other byte of that word.
    #[validate(range(min = 0, max = 255))]
    #[default(DEFAULT_PIX_SURPLUS_SHARES as usize)]
    pub additional_shares: usize,

    /// Maximum number of SSA commitments this node, acting as an Entry, accepts in a single
    /// `SsaRequest` from an Exit.
    ///
    /// This is a protection against a misbehaving Exit rather than a preference: each accepted entry
    /// costs a full client commitment, its own burst of `SsaCommit` packets and its own on-chain
    /// deposit, so an uncapped request would let one inbound packet amplify into minutes of CPU and
    /// as many simultaneous deposits as the wire format admits (27). An over-cap request is rejected
    /// in full before any of that work starts.
    ///
    /// **Must be at least the `ssas_per_request` of every Exit this node uses.** The batch size is not
    /// negotiated — the Exit cannot learn this value — so an Exit batching above it has every request
    /// rejected, and every such Session is lost. The refusal is reported to the Exit as an
    /// `UnacceptablePixParams` `SessionError` so it fails in about a round trip rather than as a
    /// deposit timeout minutes later, but raising the Exit side still requires raising this in step.
    ///
    /// Unlike its neighbours this is not a dimension, so `validate_pix_dimension_product` ignores it.
    ///
    /// Defaults to 2, minimum 1, maximum 20 (`MAX_SSA_BATCH_SIZE`).
    #[validate(range(min = 1, max = 20))]
    #[default(DEFAULT_MAX_SSAS_PER_SSA_REQUEST)]
    pub max_ssas_per_request: usize,

    /// Exit-side SSA reconstructor configuration.
    ///
    /// Nested rather than flattened so the whole PIX surface stays under one key, and so that
    /// Exit-side capacity does not intermix with the Entry-side dimensions above.
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub reconstructor: PixReconstructorConfig,
}

/// Rejects a reconstructor configuration that [`SsaReconstructorConfig`] itself would reject.
///
/// The mirror below deliberately carries no `range` attributes of its own. Every bound on those
/// seven fields is a property of the reconstructor, not of this crate, so the protocol type stays
/// the single source of truth for them and this delegates rather than restating. A restated range
/// is simply a second place to forget when the first one moves.
///
/// `validator` schema functions return one [`ValidationError`], so the inner [`ValidationErrors`]
/// is folded into its message — [`validate_incoming_session_pix_config`] above is the existing
/// precedent in this file for a hand-written check that reaches into another crate.
fn validate_pix_reconstructor_config(cfg: &PixReconstructorConfig) -> Result<(), ValidationError> {
    SsaReconstructorConfig::from(*cfg).validate().map_err(|errors| {
        let mut error = ValidationError::new("pix reconstructor configuration is out of range");
        error.message = Some(errors.to_string().into());
        error
    })
}

/// Operator-facing mirror of [`SsaReconstructorConfig`], the Exit-side share reconstructor.
///
/// Stands in the same relationship to the protocol type that the fields of [`PixGlobalConfig`]
/// stand in to `SsaGeneratorConfig`: `hopr-protocol-pix` owns the type the reconstructor is built
/// from, and this crate owns the shape an operator writes. It exists because none of these seven
/// values was reachable from a config file at all — both production constructors took
/// `SsaReconstructorConfig::default()`, so the Exit side of PIX was unconfigurable while the Entry
/// side was not.
///
/// Duplicating seven fields is the price of the mirror, and two guards pay it. The [`From`] impl
/// below is written exhaustively, so a field added to [`SsaReconstructorConfig`] fails to compile
/// until it is mirrored here; and `pix_reconstructor_mirror_matches_the_protocol_defaults` asserts
/// the two default sets still agree. Validation is not duplicated at all — see
/// [`validate_pix_reconstructor_config`].
///
/// Each field's rationale lives on the protocol type and is linked rather than copied, since a
/// third copy of the same prose is a third thing to keep true.
#[derive(Clone, Copy, Debug, PartialEq, Validate, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
#[validate(schema(function = "validate_pix_reconstructor_config", skip_on_field_errors = false))]
pub struct PixReconstructorConfig {
    /// Time until the complete commitment to an SSA must be received.
    ///
    /// Defaults to 2 minutes. See
    /// [`SsaReconstructorConfig::incomplete_commitment_lifetime`].
    #[default(SsaReconstructorConfig::DEFAULT_INCOMPLETE_COMMITMENT_LIFETIME)]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub incomplete_commitment_lifetime: Duration,

    /// Maximum time an SSA cycle can go without progress before it is discarded.
    ///
    /// Defaults to 30 minutes. See [`SsaReconstructorConfig::unused_verifier_lifetime`].
    #[default(SsaReconstructorConfig::DEFAULT_UNUSED_VERIFIER_LIFETIME)]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub unused_verifier_lifetime: Duration,

    /// Maximum number of peers tracked simultaneously with unacknowledged shares.
    ///
    /// Defaults to 2000, minimum 10. See [`SsaReconstructorConfig::max_tracked_peers`].
    #[default(SsaReconstructorConfig::DEFAULT_MAX_TRACKED_PEERS)]
    pub max_tracked_peers: usize,

    /// Maximum number of awaited acknowledgements held **per peer**.
    ///
    /// Defaults to 1 000 000, minimum 10 000. See [`SsaReconstructorConfig::max_awaiting_acks`].
    #[default(SsaReconstructorConfig::DEFAULT_MAX_AWAITING_ACKS)]
    pub max_awaiting_acks: usize,

    /// Maximum time an acknowledgement is awaited before its share is discarded.
    ///
    /// Defaults to 30 seconds. See [`SsaReconstructorConfig::max_ack_await_time`].
    #[default(SsaReconstructorConfig::DEFAULT_MAX_ACK_AWAIT_TIME)]
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub max_ack_await_time: Duration,

    /// Whether to use the batch verification algorithm for acknowledgements.
    ///
    /// Defaults to `false`. See [`SsaReconstructorConfig::use_batch_verification`], which records
    /// the measurements behind that default and why the knob was kept rather than removed.
    #[default(SsaReconstructorConfig::DEFAULT_USE_BATCH_VERIFICATION)]
    pub use_batch_verification: bool,

    /// Fraction of reconstructed polynomials at which an early recovery notification is emitted.
    ///
    /// Defaults to 0.85, range 0.0..=1.0. See
    /// [`SsaReconstructorConfig::early_recovery_threshold`].
    #[default(SsaReconstructorConfig::DEFAULT_EARLY_RECOVERY_THRESHOLD)]
    pub early_recovery_threshold: f64,

    /// Ceiling on the total awaiting-acknowledgement state held across every peer, in bytes.
    ///
    /// Defaults to 1 GiB, minimum 16 MiB. See
    /// [`SsaReconstructorConfig::max_ack_buffer_bytes`].
    ///
    /// This — not the product of [`max_tracked_peers`](Self::max_tracked_peers) and
    /// [`max_awaiting_acks`](Self::max_awaiting_acks) — is what bounds the reconstructor's
    /// acknowledgement buffer, and the reconstructor enforces it as shares arrive rather than
    /// checking a workload model here. A model would have to assume a Session count and a packet
    /// rate; `maximum_managed_sessions` validates to 100 000 and `SessionCapability::NoRateControl`
    /// removes the rate limiter, so neither assumption survives contact with a legal configuration.
    #[default(SsaReconstructorConfig::DEFAULT_MAX_ACK_BUFFER_BYTES)]
    pub max_ack_buffer_bytes: usize,
}

impl From<PixReconstructorConfig> for SsaReconstructorConfig {
    /// Both sides are written out exhaustively, with no `..Default::default()` on either.
    ///
    /// That is the guard the mirror rests on: a field added to [`SsaReconstructorConfig`] leaves
    /// this initialiser incomplete and a field added to [`PixReconstructorConfig`] leaves the
    /// pattern incomplete, so either one fails to compile until it is mirrored. Struct update
    /// syntax would compile in both directions and silently pin the new knob to its default.
    fn from(cfg: PixReconstructorConfig) -> Self {
        let PixReconstructorConfig {
            incomplete_commitment_lifetime,
            unused_verifier_lifetime,
            max_tracked_peers,
            max_awaiting_acks,
            max_ack_await_time,
            use_batch_verification,
            early_recovery_threshold,
            max_ack_buffer_bytes,
        } = cfg;

        Self {
            incomplete_commitment_lifetime,
            unused_verifier_lifetime,
            max_tracked_peers,
            max_awaiting_acks,
            max_ack_await_time,
            use_batch_verification,
            early_recovery_threshold,
            max_ack_buffer_bytes,
        }
    }
}

/// Configuration of the HOPR packet pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Validate, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HoprPacketPipelineConfig {
    /// HOPR packet codec configuration
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub codec: HoprCodecConfig,
    /// Configuration of unacknowledged tickets processing.
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub ack_processor: HoprUnacknowledgedTicketProcessorConfig,
    /// Single Use Reply Block (SURB) handling configuration
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub surb_store: SurbStoreConfig,
    /// Packet pipeline configuration controlling output/input concurrency and acknowledgement processing
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub pipeline: PacketPipelineConfig,
}

regex!(is_dns_address_regex "^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\\.)*[a-z0-9][a-z0-9-]{0,61}[a-z0-9]$");

/// Check whether the string looks like a valid domain.
#[inline]
pub fn looks_like_domain(s: &str) -> bool {
    is_dns_address_regex(s)
}

/// Check whether the string is an actual reachable domain.
pub fn is_reachable_domain(host: &str) -> bool {
    host.to_socket_addrs().is_ok_and(|i| i.into_iter().next().is_some())
}

/// Enumeration of possible host types.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HostType {
    /// IPv4 based host
    IPv4(String),
    /// DNS based host
    Domain(String),
}

impl validator::Validate for HostType {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match &self {
            HostType::IPv4(ip4) => validate_ipv4_address(ip4).map_err(|e| {
                let mut errs = ValidationErrors::new();
                errs.add("ipv4", e);
                errs
            }),
            HostType::Domain(domain) => validate_dns_address(domain).map_err(|e| {
                let mut errs = ValidationErrors::new();
                errs.add("domain", e);
                errs
            }),
        }
    }
}

impl Default for HostType {
    fn default() -> Self {
        HostType::IPv4("127.0.0.1".to_owned())
    }
}

/// Configuration of the listening host.
///
/// This is used for the P2P and REST API listeners.
///
/// Intentionally has no default because it depends on the use case.
#[derive(Debug, Validate, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HostConfig {
    /// Host on which to listen
    #[cfg_attr(feature = "serde", serde(default))]
    pub address: HostType,
    /// Listening TCP or UDP port (mandatory).
    #[validate(range(min = 1u16))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub port: u16,
}

impl FromStr for HostConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip_or_dns, str_port) = match s.split_once(':') {
            None => return Err("Invalid host, is not in the '<host>:<port>' format".into()),
            Some(split) => split,
        };

        let port = str_port.parse().map_err(|e: ParseIntError| e.to_string())?;

        if validator::ValidateIp::validate_ipv4(&ip_or_dns) {
            Ok(Self {
                address: HostType::IPv4(ip_or_dns.to_owned()),
                port,
            })
        } else if looks_like_domain(ip_or_dns) {
            Ok(Self {
                address: HostType::Domain(ip_or_dns.to_owned()),
                port,
            })
        } else {
            Err("Not a valid IPv4 or domain host".into())
        }
    }
}

impl Display for HostConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{}", self.address, self.port)
    }
}

fn default_multiaddr_transport(port: u16) -> String {
    cfg_if::cfg_if! {
        if #[cfg(feature = "p2p-announce-quic")] {
            // In case we run on a Dappnode-like device, presumably behind NAT, we fall back to TCP
            // to circumvent issues with QUIC in such environments. To make this work reliably,
            // we would need proper NAT traversal support.
            let on_dappnode = std::env::var("DAPPNODE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false);

            // Using HOPRD_NAT a user can overwrite the default behaviour even on a Dappnode-like device
            let uses_nat = std::env::var("HOPRD_NAT")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(on_dappnode);

            if uses_nat {
                format!("tcp/{port}")
            } else {
                format!("udp/{port}/quic-v1")
            }
        } else {
            format!("tcp/{port}")
        }
    }
}

impl TryFrom<&HostConfig> for Multiaddr {
    type Error = HoprTransportError;

    fn try_from(value: &HostConfig) -> Result<Self, Self::Error> {
        match &value.address {
            HostType::IPv4(ip) => Multiaddr::from_str(
                format!("/ip4/{}/{}", ip.as_str(), default_multiaddr_transport(value.port)).as_str(),
            )
            .map_err(|e| HoprTransportError::Api(e.to_string())),
            HostType::Domain(domain) => Multiaddr::from_str(
                format!("/dns4/{}/{}", domain.as_str(), default_multiaddr_transport(value.port)).as_str(),
            )
            .map_err(|e| HoprTransportError::Api(e.to_string())),
        }
    }
}

fn validate_ipv4_address(s: &str) -> Result<(), ValidationError> {
    if validator::ValidateIp::validate_ipv4(&s) {
        let ipv4 = std::net::Ipv4Addr::from_str(s)
            .map_err(|_| ValidationError::new("Failed to deserialize the string into an ipv4 address"))?;

        if ipv4.is_private() || ipv4.is_multicast() || ipv4.is_unspecified() {
            return Err(ValidationError::new(
                "IPv4 cannot be private, multicast or unspecified (0.0.0.0)",
            ))?;
        }
        Ok(())
    } else {
        Err(ValidationError::new("Invalid IPv4 address provided"))
    }
}

fn validate_dns_address(s: &str) -> Result<(), ValidationError> {
    if looks_like_domain(s) || is_reachable_domain(s) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid DNS address provided"))
    }
}

/// Configuration of the physical transport mechanism.
#[derive(Debug, Default, Validate, Clone, Copy, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct TransportConfig {
    /// When true, assume that the node is running in an isolated network and does
    /// not need any connection to nodes outside the subnet
    #[cfg_attr(feature = "serde", serde(default))]
    pub announce_local_addresses: bool,
    /// When true, assume a testnet with multiple nodes running on the same machine
    /// or in the same private IPv4 network
    #[cfg_attr(feature = "serde", serde(default))]
    pub prefer_local_addresses: bool,
}

const DEFAULT_SESSION_IDLE_TIMEOUT: Duration = Duration::from_mins(3);

const SESSION_IDLE_MIN_TIMEOUT: Duration = Duration::from_secs(2);

const DEFAULT_SESSION_ESTABLISH_RETRY_DELAY: Duration = Duration::from_secs(2);

const DEFAULT_SESSION_ESTABLISH_MAX_RETRIES: usize = 3;

const DEFAULT_SESSION_BALANCER_SAMPLING: Duration = Duration::from_millis(100);

const DEFAULT_SESSION_BALANCER_BUFFER_DURATION: Duration = Duration::from_secs(5);

const DEFAULT_MAXIMUM_MANAGED_SESSIONS: usize = 100;

fn default_session_balancer_buffer_duration() -> Duration {
    DEFAULT_SESSION_BALANCER_BUFFER_DURATION
}

fn default_session_establish_max_retries() -> usize {
    DEFAULT_SESSION_ESTABLISH_MAX_RETRIES
}

fn default_session_idle_timeout() -> Duration {
    DEFAULT_SESSION_IDLE_TIMEOUT
}

fn default_session_establish_retry_delay() -> Duration {
    DEFAULT_SESSION_ESTABLISH_RETRY_DELAY
}

fn default_session_balancer_sampling() -> Duration {
    DEFAULT_SESSION_BALANCER_SAMPLING
}

fn default_max_managed_sessions() -> usize {
    DEFAULT_MAXIMUM_MANAGED_SESSIONS
}

/// Transport-layer default for the SURB balance notification period; this is the effective default
/// for [`SessionGlobalConfig::surb_balance_notify_period`] (15s). It deliberately overrides the
/// lower-level fallback in `SessionManagerConfig` (whose own field default is 60s) with a tighter
/// 15s cadence, so the Entry's dead-reckoned estimate of the Exit's SURB buffer is corrected often
/// enough to keep the SURB balancer from under-producing (and starving the Exit) under drift,
/// without the per-session keep-alive overhead of the previous 2s cadence. The 1s floor is enforced
/// downstream by `SessionManager::new` (`MIN_SURB_BUFFER_NOTIFICATION_PERIOD`).
fn default_session_surb_balance_notify_period() -> Option<Duration> {
    Some(Duration::from_secs(15))
}

fn validate_session_idle_timeout(value: &Duration) -> Result<(), ValidationError> {
    if SESSION_IDLE_MIN_TIMEOUT <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("session idle timeout is too low"))
    }
}

fn validate_balancer_sampling(value: &Duration) -> Result<(), ValidationError> {
    if MIN_BALANCER_SAMPLING_INTERVAL <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("balancer sampling interval is too low"))
    }
}

fn validate_balancer_buffer_duration(value: &Duration) -> Result<(), ValidationError> {
    if MIN_SURB_BUFFER_DURATION <= *value {
        Ok(())
    } else {
        Err(ValidationError::new("minmum SURB buffer duration is too low"))
    }
}

fn validate_surb_balance_notify_period(value: &Duration) -> Result<(), ValidationError> {
    // `custom` on an `Option` field skips `None` and passes the inner value on `Some`.
    if *value >= Duration::from_secs(1) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "SURB balance notify period must be at least 1 second",
        ))
    }
}

/// Global configuration of Sessions and the Session manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Validate, smart_default::SmartDefault)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct SessionGlobalConfig {
    /// Maximum time before an idle Session is closed.
    ///
    /// Defaults to 3 minutes.
    #[validate(custom(function = "validate_session_idle_timeout"))]
    #[default(default_session_idle_timeout())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_session_idle_timeout", with = "humantime_serde")
    )]
    pub idle_timeout: Duration,

    /// Maximum number of Sessions that can be managed by the Session manager.
    ///
    /// Default is 100, minimum is 2, maximum is 100 000.
    #[validate(range(min = 2, max = 100_000))]
    #[default(default_max_managed_sessions())]
    #[cfg_attr(feature = "serde", serde(default = "default_max_managed_sessions"))]
    pub maximum_managed_sessions: usize,

    /// Maximum retries to attempt to establish the Session
    /// Set 0 for no retries.
    ///
    /// Defaults to 3, maximum is 20.
    #[validate(range(min = 0, max = 20))]
    #[default(default_session_establish_max_retries())]
    #[cfg_attr(feature = "serde", serde(default = "default_session_establish_max_retries"))]
    pub establish_max_retries: usize,

    /// Delay between Session establishment retries.
    ///
    /// Default is 2 seconds.
    #[default(default_session_establish_retry_delay())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_session_establish_retry_delay", with = "humantime_serde")
    )]
    pub establish_retry_timeout: Duration,

    /// Sampling interval for SURB balancer in milliseconds.
    ///
    /// Default is 100 milliseconds.
    #[validate(custom(function = "validate_balancer_sampling"))]
    #[default(default_session_balancer_sampling())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_session_balancer_sampling", with = "humantime_serde")
    )]
    pub balancer_sampling_interval: Duration,

    /// Minimum runway of received SURBs in seconds.
    ///
    /// This applies to incoming Sessions on Exit nodes only and is the main indicator of how
    /// the egress traffic will be shaped, unless the `NoRateControl` Session
    /// capability is specified during initiation.
    ///
    /// Default is 5 seconds, minimum is 1 second.
    #[validate(custom(function = "validate_balancer_buffer_duration"))]
    #[default(default_session_balancer_buffer_duration())]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_session_balancer_buffer_duration", with = "humantime_serde")
    )]
    pub balancer_minimum_surb_buffer_duration: Duration,

    /// How often the Exit reports its true SURB buffer level to the Entry, as an absolute
    /// correction of the Entry's dead-reckoned estimate. Without it, cumulative packet loss
    /// silently inflates the estimate until the Exit runs out of SURBs and can no longer
    /// send reply data.
    ///
    /// Default is 15 seconds. Set to `null` to disable; minimum effective period is 1 second.
    #[validate(custom(function = "validate_surb_balance_notify_period"))]
    #[default(default_session_surb_balance_notify_period())]
    #[cfg_attr(
        feature = "serde",
        serde(
            default = "default_session_surb_balance_notify_period",
            with = "humantime_serde::option"
        )
    )]
    pub surb_balance_notify_period: Option<Duration>,

    /// Tag allocator partition configuration.
    #[validate(nested)]
    #[cfg_attr(feature = "serde", serde(default))]
    pub tag_allocator: hopr_transport_tag_allocator::TagAllocatorConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Exit computes the offered quota as `polys × (shares + surplus) × HoprPacket::PAYLOAD_SIZE`
    /// and rejects the Session when it falls outside `quota_range`. An Entry running the default
    /// `PixGlobalConfig` must therefore always be acceptable to an Exit running the default
    /// `IncomingSessionPixConfig`, otherwise PIX cannot be used at all out of the box — and
    /// before both structs became `serde(default)` there was no way for an operator to fix it.
    ///
    /// The surplus is in that product, so this also guards the alias that gives the two crates one
    /// default surplus: if `additional_shares` and `hopr-protocol-pix`'s own default drift apart
    /// again, the quota computed here stops matching the one the range is anchored on.
    #[test]
    fn default_pix_dimensions_must_be_inside_default_incoming_quota_range() {
        let pix = PixGlobalConfig::default();
        let incoming = IncomingSessionPixConfig::default();

        let quota = pix.num_ssa_parts as u64
            * (pix.ssa_part_size + pix.additional_shares) as u64
            * hopr_crypto_packet::prelude::HoprPacket::PAYLOAD_SIZE as u64;

        assert!(
            incoming.quota_range.contains(&quota),
            "default PIX quota {quota} is outside the default accepted range {:?} — every PIX session would be \
             rejected with UnacceptablePixParams",
            incoming.quota_range
        );

        // Both sides are derived from the same constants, so the range must be anchored exactly
        // at the nominal quota. Asserting the relationship rather than a literal keeps this test
        // correct if `HoprPacket::PAYLOAD_SIZE` ever changes, while still failing loudly if the
        // range or the dimensions stop being derived from a shared source.
        assert_eq!(
            quota,
            *incoming.quota_range.end(),
            "the accepted range must be anchored at the nominal default quota"
        );
    }

    #[test]
    fn default_pix_configs_must_validate() {
        PixGlobalConfig::default()
            .validate()
            .expect("default PixGlobalConfig must be valid");
        validate_incoming_session_pix_config(&IncomingSessionPixConfig::default())
            .expect("default IncomingSessionPixConfig must be valid");
        HoprProtocolConfig::default()
            .validate()
            .expect("default HoprProtocolConfig must be valid");
    }

    /// The operator-facing mirror must round-trip to the protocol type it stands for.
    ///
    /// Both sides now read the same `SsaReconstructorConfig::DEFAULT_*` constants, so this is
    /// structurally true rather than merely observed — which is the point of asserting it. What the
    /// test actually guards is a future edit that replaces one of those references with a literal:
    /// the mirror would still compile, still validate, and quietly install a different Exit.
    ///
    /// The field *set* is guarded by the compiler instead — `From` is written exhaustively in both
    /// directions, so neither struct can grow a field the other lacks.
    #[test]
    fn pix_reconstructor_mirror_matches_the_protocol_defaults() {
        assert_eq!(
            SsaReconstructorConfig::default(),
            SsaReconstructorConfig::from(PixReconstructorConfig::default()),
            "the operator-facing mirror and the reconstructor it configures have drifted apart"
        );
    }

    /// Each dimension range is satisfiable on its own well past anything measured, so the product
    /// needs its own bound.
    ///
    /// Both fields are operator-settable, and the product *is* the per-cycle quota: it decides how
    /// many polynomials the Entry builds and how many commitments and part builders the Exit holds.
    /// The peer's `quota_range` refusal is no defence — it fires after this node has generated.
    #[test]
    fn pix_dimensions_are_bounded_by_their_product_not_only_field_by_field() {
        const MAX_NUM_SSA_PARTS: usize = 16192;
        const MAX_SSA_PART_SIZE: usize = hopr_protocol_pix::MAX_POLY_THRESHOLD as usize;

        // The extremes of the two field ranges, each individually valid.
        let extreme = PixGlobalConfig {
            num_ssa_parts: MAX_NUM_SSA_PARTS,
            ssa_part_size: MAX_SSA_PART_SIZE,
            ..Default::default()
        };
        assert!(
            extreme.validate().is_err(),
            "16192 x 255 is ~8x the profiled product and must be rejected"
        );

        // Re-splitting at a constant product is exactly what a re-tune does, and must stay valid —
        // this is why the ceiling is on the product rather than on either field. The splits are
        // fewer than they were: `ssa_part_size` is now capped at 255, so the 2048 x 256 and
        // 1024 x 512 re-splits this used to cover are no longer expressible at all.
        let profiled = DEFAULT_PIX_POLYS_PER_SSA as usize * DEFAULT_PIX_SHARES_PER_POLY as usize;
        for (polys, shares) in [(4096usize, 128usize), (8192, 64)] {
            assert_eq!(polys * shares, profiled, "test case must hold the product constant");
            let cfg = PixGlobalConfig {
                num_ssa_parts: polys,
                ssa_part_size: shares,
                ..Default::default()
            };
            assert!(
                cfg.validate().is_ok(),
                "{polys} x {shares} is the profiled product re-split and must stay valid"
            );
        }

        // The headroom is real: the ceiling is straddled rather than hit, because hitting it exactly
        // is no longer possible. `4 x profiled` is 2^21, and every factorisation of it with
        // `ssa_part_size <= 255` needs `num_ssa_parts >= 16384`, past that field's own maximum. So
        // the pair below is the largest accepted product and the next one up, one share apart.
        let ceiling = MAX_PIX_DIMENSION_PRODUCT_FACTOR * profiled;
        let just_under = PixGlobalConfig {
            num_ssa_parts: MAX_NUM_SSA_PARTS,
            ssa_part_size: ceiling / MAX_NUM_SSA_PARTS,
            ..Default::default()
        };
        assert!(
            just_under.num_ssa_parts * just_under.ssa_part_size <= ceiling,
            "test case must sit under the ceiling"
        );
        assert!(
            just_under.validate().is_ok(),
            "the largest reachable product under the ceiling must be accepted"
        );

        let past_ceiling = PixGlobalConfig {
            ssa_part_size: just_under.ssa_part_size + 1,
            ..just_under
        };
        assert!(
            past_ceiling.num_ssa_parts * past_ceiling.ssa_part_size > ceiling,
            "test case must sit over the ceiling"
        );
        assert!(past_ceiling.validate().is_err(), "past the ceiling must be rejected");
    }

    // The reversed range is the point of the test: `quota_range` is operator-settable, so an
    // inverted range is reachable from a config file and must be rejected by validation rather
    // than silently matching nothing.
    #[allow(clippy::reversed_empty_ranges)]
    #[test]
    fn empty_pix_quota_range_is_rejected() {
        let cfg = IncomingSessionPixConfig {
            quota_range: 100..=10,
            ..Default::default()
        };
        assert!(validate_incoming_session_pix_config(&cfg).is_err());

        let cfg = HoprProtocolConfig {
            incoming_session_pix_config: IncomingSessionPixConfig {
                quota_range: 100..=10,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn pix_configs_are_reachable_from_serialized_config() {
        // Regression guard: these two fields used to be `serde(skip)`, which pinned them to
        // their defaults and made PIX unconfigurable.
        let json = r#"{
            "pix": { "num_ssa_parts": 2048 },
            "incoming_session_pix_config": { "enforce_pix": true, "max_deposit_wait": "90s" }
        }"#;
        let cfg: HoprProtocolConfig = serde_json::from_str(json).expect("PIX config must deserialize");

        assert_eq!(2048, cfg.pix.num_ssa_parts);
        // Container-level `serde(default)` keeps unspecified fields at their defaults.
        assert_eq!(
            PixGlobalConfig::default().ssa_part_size,
            cfg.pix.ssa_part_size,
            "unspecified PIX fields must fall back to their defaults"
        );
        assert!(cfg.incoming_session_pix_config.enforce_pix);
        assert_eq!(
            Duration::from_secs(90),
            cfg.incoming_session_pix_config.max_deposit_wait
        );
        assert_eq!(
            IncomingSessionPixConfig::default().quota_range,
            cfg.incoming_session_pix_config.quota_range
        );

        // Both SSA batch knobs must be settable from a config file too, since raising one without the
        // other is a silently fatal misconfiguration and an operator needs to be able to do both.
        let json = r#"{
            "pix": { "max_ssas_per_request": 5 },
            "incoming_session_pix_config": { "ssas_per_request": 5 }
        }"#;
        let cfg: HoprProtocolConfig = serde_json::from_str(json).expect("SSA batch config must deserialize");
        assert_eq!(5, cfg.pix.max_ssas_per_request);
        assert_eq!(5, cfg.incoming_session_pix_config.ssas_per_request);
        cfg.validate().expect("a matched pair of batch knobs must validate");

        // Same defect one struct down: every Exit-side reconstructor dial used to be unreachable
        // because both production constructors took `SsaReconstructorConfig::default()`. The
        // durations must come through `humantime_serde` rather than as a struct of secs/nanos.
        let json = r#"{
            "pix": { "reconstructor": { "max_ack_await_time": "45s", "max_tracked_peers": 500 } }
        }"#;
        let cfg: HoprProtocolConfig = serde_json::from_str(json).expect("reconstructor config must deserialize");
        assert_eq!(Duration::from_secs(45), cfg.pix.reconstructor.max_ack_await_time);
        assert_eq!(500, cfg.pix.reconstructor.max_tracked_peers);
        assert_eq!(
            PixReconstructorConfig::default().unused_verifier_lifetime,
            cfg.pix.reconstructor.unused_verifier_lifetime,
            "unspecified reconstructor fields must fall back to their defaults"
        );
        cfg.validate().expect("a narrowed reconstructor must validate");

        // And it must reach the reconstructor, not merely parse: the conversion is what the two
        // production constructors consume.
        assert_eq!(
            Duration::from_secs(45),
            SsaReconstructorConfig::from(cfg.pix.reconstructor).max_ack_await_time
        );
    }

    /// Both SSA batch knobs are bounded by [`MAX_SSA_BATCH_SIZE`], and the `range` attribute on
    /// `max_ssas_per_request` has to spell that ceiling out as a literal — so assert the two agree.
    ///
    /// Zero is rejected on both sides for different reasons: an Exit asking for zero SSAs would never
    /// send an `SsaRequest` at all, and an Entry accepting zero would reject every request it ever
    /// receives. Either way PIX silently stops working.
    #[test]
    fn ssa_batch_knobs_are_bounded_by_the_shared_ceiling() {
        // The literal in the `range` attribute must track the constant it stands for.
        let at_ceiling = PixGlobalConfig {
            max_ssas_per_request: MAX_SSA_BATCH_SIZE,
            ..Default::default()
        };
        assert!(
            at_ceiling.validate().is_ok(),
            "MAX_SSA_BATCH_SIZE itself must be accepted — the range literal has drifted below it"
        );
        let past_ceiling = PixGlobalConfig {
            max_ssas_per_request: MAX_SSA_BATCH_SIZE + 1,
            ..Default::default()
        };
        assert!(
            past_ceiling.validate().is_err(),
            "above MAX_SSA_BATCH_SIZE must be rejected — the range literal has drifted above it"
        );

        assert!(
            PixGlobalConfig {
                max_ssas_per_request: 0,
                ..Default::default()
            }
            .validate()
            .is_err(),
            "an Entry accepting zero SSAs per request would reject every request"
        );

        for ssas_per_request in [0, MAX_SSA_BATCH_SIZE + 1] {
            let cfg = IncomingSessionPixConfig {
                ssas_per_request,
                ..Default::default()
            };
            assert!(
                validate_incoming_session_pix_config(&cfg).is_err(),
                "ssas_per_request of {ssas_per_request} is outside 1..={MAX_SSA_BATCH_SIZE} and must be rejected"
            );

            let cfg = HoprProtocolConfig {
                incoming_session_pix_config: IncomingSessionPixConfig {
                    ssas_per_request,
                    ..Default::default()
                },
                ..Default::default()
            };
            assert!(
                cfg.validate().is_err(),
                "an out-of-range ssas_per_request must fail the whole protocol config"
            );
        }

        assert!(
            validate_incoming_session_pix_config(&IncomingSessionPixConfig {
                ssas_per_request: MAX_SSA_BATCH_SIZE,
                ..Default::default()
            })
            .is_ok(),
            "the ceiling itself must be accepted"
        );
    }

    #[test]
    fn egress_backpressure_timeout_rejects_sub_minimum_values() {
        assert!(validate_egress_backpressure_timeout(&Duration::ZERO).is_err());
        assert!(validate_egress_backpressure_timeout(&Duration::from_micros(500)).is_err());
        assert!(validate_egress_backpressure_timeout(&MIN_EGRESS_BACKPRESSURE_TIMEOUT).is_ok());
        assert!(validate_egress_backpressure_timeout(&DEFAULT_EGRESS_BACKPRESSURE_TIMEOUT).is_ok());
    }

    #[test]
    fn stream_protocol_config_default_is_valid() {
        assert!(StreamProtocolConfig::default().validate().is_ok());
    }

    #[test]
    fn test_valid_domains_for_looks_like_a_domain() {
        assert!(looks_like_domain("localhost"));
        assert!(looks_like_domain("hoprnet.org"));
        assert!(looks_like_domain("hub.hoprnet.org"));
    }

    #[test]
    fn test_valid_domains_for_does_not_look_like_a_domain() {
        assert!(!looks_like_domain(".org"));
        assert!(!looks_like_domain("-hoprnet-.org"));
    }

    #[test]
    fn test_valid_domains_should_be_reachable() {
        assert!(!is_reachable_domain("google.com"));
    }

    #[test]
    fn test_verify_valid_ip4_addresses() {
        assert!(validate_ipv4_address("1.1.1.1").is_ok());
        assert!(validate_ipv4_address("1.255.1.1").is_ok());
        assert!(validate_ipv4_address("187.1.1.255").is_ok());
        assert!(validate_ipv4_address("127.0.0.1").is_ok());
    }

    #[test]
    fn test_verify_invalid_ip4_addresses() {
        assert!(validate_ipv4_address("1.256.1.1").is_err());
        assert!(validate_ipv4_address("-1.1.1.255").is_err());
        assert!(validate_ipv4_address("127.0.0.256").is_err());
        assert!(validate_ipv4_address("1").is_err());
        assert!(validate_ipv4_address("1.1").is_err());
        assert!(validate_ipv4_address("1.1.1").is_err());
        assert!(validate_ipv4_address("1.1.1.1.1").is_err());
    }

    #[test]
    fn test_verify_valid_dns_addresses() {
        assert!(validate_dns_address("localhost").is_ok());
        assert!(validate_dns_address("google.com").is_ok());
        assert!(validate_dns_address("hub.hoprnet.org").is_ok());
    }

    #[test]
    fn test_verify_invalid_dns_addresses() {
        assert!(validate_dns_address("-hoprnet-.org").is_err());
    }

    #[test]
    fn test_multiaddress_on_dappnode_default() {
        temp_env::with_var("DAPPNODE", Some("true"), || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }

    #[cfg(feature = "p2p-announce-quic")]
    #[test]
    fn test_multiaddress_on_non_dappnode_default() {
        temp_env::with_vars([("DAPPNODE", Some("false")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "p2p-announce-quic"))]
    #[test]
    fn test_multiaddress_on_non_dappnode_default() {
        assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
    }

    #[test]
    fn test_multiaddress_on_non_dappnode_uses_nat() {
        temp_env::with_var("HOPRD_NAT", Some("true"), || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }

    #[cfg(feature = "p2p-announce-quic")]
    #[test]
    fn test_multiaddress_on_non_dappnode_not_uses_nat() {
        temp_env::with_var("HOPRD_NAT", Some("false"), || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "p2p-announce-quic"))]
    #[test]
    fn test_multiaddress_on_non_dappnode_not_uses_nat() {
        temp_env::with_var("HOPRD_NAT", Some("false"), || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }

    #[cfg(feature = "p2p-announce-quic")]
    #[test]
    fn test_multiaddress_on_dappnode_not_uses_nat() {
        temp_env::with_vars([("DAPPNODE", Some("true")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "udp/1234/quic-v1");
        });
    }

    #[cfg(not(feature = "p2p-announce-quic"))]
    #[test]
    fn test_multiaddress_on_dappnode_not_uses_nat() {
        temp_env::with_vars([("DAPPNODE", Some("true")), ("HOPRD_NAT", Some("false"))], || {
            assert_eq!(default_multiaddr_transport(1234), "tcp/1234");
        });
    }

    // --- HostConfig::FromStr tests ---

    #[test]
    fn host_config_parses_ipv4_address() {
        let cfg = HostConfig::from_str("1.2.3.4:9091").unwrap();
        insta::assert_debug_snapshot!(cfg);
    }

    #[test]
    fn host_config_parses_domain() {
        let cfg = HostConfig::from_str("example.com:443").unwrap();
        insta::assert_debug_snapshot!(cfg);
    }

    #[test]
    fn host_config_rejects_missing_port() {
        assert!(HostConfig::from_str("1.2.3.4").is_err());
    }

    #[test]
    fn host_config_rejects_invalid_port() {
        assert!(HostConfig::from_str("1.2.3.4:abc").is_err());
    }

    #[test]
    fn host_config_rejects_invalid_host() {
        assert!(HostConfig::from_str("-invalid-.com:80").is_err());
    }

    #[test]
    fn host_config_display_roundtrip() {
        let cfg = HostConfig {
            address: HostType::IPv4("10.0.0.1".into()),
            port: 8080,
        };
        insta::assert_yaml_snapshot!(cfg.to_string());
    }

    // --- TryFrom<&HostConfig> for Multiaddr tests ---

    #[test]
    fn multiaddr_from_ipv4_host_config() {
        let cfg = HostConfig {
            address: HostType::IPv4("1.2.3.4".into()),
            port: 9091,
        };
        let addr = Multiaddr::try_from(&cfg).unwrap();
        insta::assert_yaml_snapshot!(addr.to_string());
    }

    #[test]
    fn multiaddr_from_domain_host_config() {
        let cfg = HostConfig {
            address: HostType::Domain("example.com".into()),
            port: 443,
        };
        let addr = Multiaddr::try_from(&cfg).unwrap();
        insta::assert_yaml_snapshot!(addr.to_string());
    }

    // --- SessionGlobalConfig validation tests ---

    #[test]
    fn session_global_config_default_is_valid() {
        let cfg = SessionGlobalConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn session_global_config_too_low_idle_timeout_is_rejected() {
        let cfg = SessionGlobalConfig {
            idle_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn session_global_config_too_many_retries_is_rejected() {
        let cfg = SessionGlobalConfig {
            establish_max_retries: 21,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn stream_protocol_config_default_has_expected_values() {
        let cfg = StreamProtocolConfig::default();
        assert_eq!(cfg.per_peer_channel_capacity, DEFAULT_PER_PEER_CHANNEL_CAPACITY);
        assert_eq!(cfg.stream_open_timeout, DEFAULT_STREAM_OPEN_TIMEOUT);
        assert_eq!(
            cfg.frame_writer_backpressure_bytes,
            DEFAULT_FRAME_WRITER_BACKPRESSURE_BYTES
        );
        cfg.validate().expect("default StreamProtocolConfig must be valid");
    }

    #[test]
    fn stream_protocol_config_zero_capacity_is_rejected() {
        let cfg = StreamProtocolConfig {
            per_peer_channel_capacity: 0,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn stream_protocol_config_zero_backpressure_bytes_is_rejected() {
        let cfg = StreamProtocolConfig {
            frame_writer_backpressure_bytes: 0,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn stream_protocol_config_zero_stream_open_timeout_is_rejected() {
        let cfg = StreamProtocolConfig {
            stream_open_timeout: Duration::ZERO,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn stream_protocol_config_zero_backpressure_timeout_is_rejected() {
        // Proves the field `#[validate(custom)]` attribute is actually wired into the derived
        // `StreamProtocolConfig::validate()` — which the parent `HoprProtocolConfig`/`HoprLibConfig`
        // invoke via `#[validate(nested)]` at node build time (builder.rs `cfg.validate()?`).
        let cfg = StreamProtocolConfig {
            egress_backpressure_timeout: Duration::ZERO,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }
}
