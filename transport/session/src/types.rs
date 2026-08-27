use std::{
    convert::Into,
    fmt::Debug,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures::{SinkExt, StreamExt, TryStreamExt};
use hopr_api::{
    node::PixDepositData,
    types::{
        internal::{prelude::HoprPseudonym, routing::DestinationRouting},
        primitive::errors::GeneralError,
    },
};
use hopr_crypto_packet::{
    HoprPixSpec,
    prelude::{HoprPacket, HoprPixCommitmentProof, HoprPixGroupElement},
};
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataIn, ApplicationDataOut, ReservedTag, Tag};
use hopr_protocol_pix::{PixParams, PixSpec, SsaId};
#[cfg(feature = "telemetry")]
use hopr_protocol_session::NoopTracker;
use hopr_protocol_session::{
    AcknowledgementMode, AcknowledgementState, AcknowledgementStateConfig, ReliableSocket, SessionSocketConfig,
    UnreliableSocket,
    flow_control::{DeliveryClock, DeliveryMeter, DeliveryTap, FlowControlConfig},
};
use hopr_protocol_start::StartProtocol;
use hopr_utils::network_types::utils::{AsyncWriteSink, DuplexIO};
use tracing::{debug, instrument, warn};

use crate::{
    Capabilities, Capability,
    balancer::BalancerStateValues,
    errors::{SessionManagerError, TransportSessionError},
    flow_control::{PacedWriter, SurbSupply},
};

/// Wrapper for [`Capabilities`] that makes conversion to/from `u8` possible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoprSessionCapabilities(pub Capabilities);

impl HoprSessionCapabilities {
    pub fn empty() -> Self {
        Self(Capabilities::empty())
    }
}

impl TryFrom<u8> for HoprSessionCapabilities {
    type Error = GeneralError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Capabilities::new(value)
            .map(Self)
            .map_err(|_| GeneralError::ParseError("capabilities".into()))
    }
}

impl From<HoprSessionCapabilities> for u8 {
    fn from(value: HoprSessionCapabilities) -> Self {
        *value.0.as_ref()
    }
}

impl From<HoprSessionCapabilities> for Capabilities {
    fn from(value: HoprSessionCapabilities) -> Self {
        value.0
    }
}

impl From<Capabilities> for HoprSessionCapabilities {
    fn from(value: Capabilities) -> Self {
        Self(value)
    }
}

impl AsRef<Capabilities> for HoprSessionCapabilities {
    fn as_ref(&self) -> &Capabilities {
        &self.0
    }
}

/// Start protocol instantiation for HOPR.
pub type HoprStartProtocol = StartProtocol<
    SessionId,
    SessionTarget,
    HoprSessionCapabilities,
    HoprPixGroupElement,
    HoprPixCommitmentProof,
    HoprPixDepositPayload,
>;

/// The deposit data a [deposit pool](hopr_api::chain::DepositPool) produced for one batch of SSAs.
///
/// This is what the Exit *collects*, not what it sends: the pool answers a
/// [`PixDepositDataRequest`](hopr_api::node::PixDepositDataRequest) with one
/// [`PixDepositData`] per requested [`PixAddressId`](hopr_api::node::PixAddressId), delivered one at
/// a time over the `deposit_data_created` channel, so a batch arrives as a flat list rather than as
/// anything keyed. [`deposit_data_for_batch`] turns it into the per-SSA map an `SsaRequest` carries.
pub type HoprPixDepositData = Vec<PixDepositData>;

/// The deposit data for a *single* SSA, as it travels inside an `SsaRequest`.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct HoprPixDepositPayload(#[serde(with = "serde_bytes")] pub Box<[u8]>);

/// Turns the deposit data a pool produced for a batch into the per-SSA map an `SsaRequest` carries.
///
/// Every SSA in the batch must come out of this with an entry, and the absence of one is fatal — see
/// [`MissingDepositData`](crate::errors::SessionManagerError::MissingDepositData) for why the batch
/// cannot simply travel short. An *empty* entry is not an absent one: a pool with nothing to attach
/// says so by answering with empty data, which is a value
/// ([`PixDepositData::is_empty`]) and travels fine.
///
/// Entries the batch has no place for — another Session's, an SSA outside this batch, a second answer
/// for an index already answered — are dropped and reported. They are not themselves fatal: what they
/// are evidence of is, and the gap they leave behind is what fails.
pub(crate) fn deposit_data_for_batch(
    session_id: &SessionId,
    batch: &[hopr_protocol_pix::SsaIndex],
    deposit_data: HoprPixDepositData,
) -> Result<std::collections::HashMap<hopr_protocol_pix::SsaIndex, HoprPixDepositPayload>, SessionManagerError> {
    let mut out = std::collections::HashMap::with_capacity(batch.len());
    let (mut foreign, mut duplicate) = (0usize, 0usize);

    for entry in deposit_data {
        let ssa_index = entry.id.ssa_index();
        if &entry.id.session_id() != session_id || !batch.contains(&ssa_index) {
            foreign += 1;
            continue;
        }
        if out.insert(ssa_index, HoprPixDepositPayload(entry.data)).is_some() {
            duplicate += 1;
        }
    }

    if foreign > 0 || duplicate > 0 {
        warn!(
            %session_id,
            requested = batch.len(),
            usable = out.len(),
            foreign,
            duplicate,
            "deposit pool answered with entries this batch has no place for"
        );
    }

    let missing = batch.iter().filter(|i| !out.contains_key(i)).collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(SessionManagerError::MissingDepositData(format!(
            "session {session_id} is missing deposit data for {} of {} SSAs in the batch: {missing:?} ({foreign} \
             entries were for other SSAs, {duplicate} were repeats)",
            missing.len(),
            batch.len(),
        )));
    }

    Ok(out)
}

/// Quota per single SSA in bytes.
///
/// The quota in bytes has only informative value for the user - the volume of Exit -> Entry data
/// one SSA cycle carries, which is what a single deposit pays for.
///
/// Not the volume at which the Exit *recovers* the SSA private key: recovery needs
/// `polys × threshold` useful shares, and so happens before the cycle drains. The quota covers the
/// whole cycle, surplus included — see [`pix_params_to_quota`].
///
/// The SessionManager always counts in packets, not in bytes, when it comes to quota management.
pub type SsaQuota = u64;

/// What a single SSA deposit buys: every Exit → Entry payload byte the cycle carries.
///
/// Counts [`PixParams::emitted_shares_per_poly`], i.e. `threshold + surplus`, and not the threshold
/// alone. A polynomial leaves the generator's queue only once it has emitted `threshold + surplus`
/// shares (`SsaShareGenerator::next_share`), whether or not any were lost, and each share rides back
/// on one Exit → Entry data packet — so this is exactly the traffic the Exit serves per cycle.
/// Charging the threshold alone left the surplus unpaid, which at the deployed 1.25× factor is a
/// fifth of all Exit → Entry traffic.
///
/// The surplus is insurance the Entry buys against share loss, and it is billed like insurance: on
/// purchase, not on claim. One deposit per cycle at this rate is what makes paid-for equal served.
///
/// The other half of that equality lives in the generator's own tests
/// (`drain_shares_by_polynomial` in `hopr-protocol-pix`), which pin a cycle's emission at
/// `polys × (threshold + surplus)`. Change either expression and the other has to move with it.
pub(crate) const fn pix_params_to_quota(params: &PixParams) -> SsaQuota {
    params.polys_per_ssa() as SsaQuota
        * params.emitted_shares_per_poly() as SsaQuota
        * HoprPacket::PAYLOAD_SIZE as SsaQuota
}

/// Default number of polynomials ("SSA parts") a single SSA is split into.
///
/// This is the single source of truth for the Entry-side generator dimension
/// (`PixGlobalConfig::num_ssa_parts`) and for the Exit-side acceptance policy
/// ([`IncomingSessionPixConfig::quota_range`](crate::IncomingSessionPixConfig::quota_range)).
/// Both must be derived from it so the two cannot drift apart: the Exit computes the
/// offered quota as `polys × (shares + surplus) × PAYLOAD_SIZE` and rejects the Session if it falls
/// outside its configured range, so a hard-coded range that no longer matches the
/// dimension defaults makes every PIX Session fail to establish.
///
/// ## Choosing the split
///
/// For a fixed useful-share count `U = polys × threshold` the *product* is pinned, so the split
/// between the two is free — but the costs scale differently, and dropping the non-constant
/// coefficient commitments (see [`hopr_protocol_pix::SsaPartCommitment`]) changed which way they
/// pull:
///
/// * Commitment wire volume and Exit ingest are one commitment per polynomial — **linear in `polys`**, and formerly
///   `polys × threshold`. Ingest is dominated by point decompression plus the cofactor-8 subgroup check.
/// * Reconstructor commitment memory is likewise `polys`, no longer `U`.
/// * Share verification is one scalar multiplication per *polynomial*, not `O(threshold)` per share. It used to be `U ×
///   threshold` and is now simply `polys`.
/// * Interpolating a polynomial is `O(threshold²)` field operations, and there are `polys` of them — `U × threshold`,
///   **linear in `threshold`**, and it is field arithmetic rather than curve arithmetic.
/// * Detection of a dishonest Entry takes `threshold` return packets, since a share set is only checked once it
///   interpolates.
/// * On the **Entry**, `SsaShareGenerator::next_share` evaluates a `threshold`-wide polynomial by Horner for every
///   share it emits — `U × threshold` again. This is much the smaller of the two per-share terms, but it is not zero,
///   and describing the Entry as threshold-free (as this list once did) is wrong.
///
/// So raising `threshold` (and lowering `polys`) buys a proportionally smaller commitment phase at
/// the cost of more interpolation, later fault detection and more Entry evaluation.
///
/// Both sides have since been measured, and `8192 × 64` stands — see
/// [`hopr_protocol_pix::DEFAULT_POLY_THRESHOLD`] for the tables. The objective is **Exit
/// reconstruction capacity**, because the Exit serves 10–30 clients while an Entry serves only
/// itself: on that measure the deployed threshold is within 0.4 % of the optimum, and the fixed
/// per-polynomial cost the Exit amortises over `threshold` shares means a *lower* threshold is
/// worse, not better. Summing Entry and Exit per-share cost instead would favour 48 by about 3 %;
/// that reading is recorded there and deliberately not acted on.
///
/// The quota is fixed by `polys × (threshold + surplus)`, so the split can be re-tuned without
/// touching session negotiation as long as that product holds.
///
/// ## Why this is an alias
///
/// The generator that produces the shares lives in `hopr-protocol-pix` and carries its own
/// defaults, so the split existed as two independent literals — and they drifted: this side was
/// re-tuned to `8192 × 64` while the pix crate stayed at a threshold of 128, which made
/// [`hopr_protocol_pix::SsaGeneratorConfig::default`] imply a 1.01 GiB quota, outside the very
/// range derived below. Four benchmarks had grown comments explaining which of the two to
/// believe. Aliasing removes the choice.
pub const DEFAULT_PIX_POLYS_PER_SSA: u16 = hopr_protocol_pix::DEFAULT_POLYS_PER_SSA;

/// Default number of shares required to reconstruct a single SSA part.
///
/// See [`DEFAULT_PIX_POLYS_PER_SSA`] for why this is shared between both sides, why it is kept
/// small relative to the polynomial count, and why it is an alias rather than a literal.
pub const DEFAULT_PIX_SHARES_PER_POLY: u8 = hopr_protocol_pix::DEFAULT_POLY_THRESHOLD;

/// Default number of shares emitted per SSA part beyond [`DEFAULT_PIX_SHARES_PER_POLY`].
///
/// An alias for the same reason as its two siblings, and with more riding on it than they have: the
/// surplus is part of [`DEFAULT_PIX_SSA_QUOTA`], so a second, disagreeing default would not merely
/// mis-describe a cycle — it would price one.
pub const DEFAULT_PIX_SURPLUS_SHARES: u8 = hopr_protocol_pix::DEFAULT_SURPLUS_SHARES;

/// The elliptic curve suite this build instantiates PIX over, as announced to peers.
///
/// A property of how the node was compiled — `pix-bjj` or `pix-secp256k1` on `hopr-crypto-packet`
/// — not of its configuration, so it is read off the spec rather than named again here. Peers that
/// disagree about it cannot exchange PIX commitments at all, because every curve-sized field on the
/// wire changes width with it; announcing it is what turns that into a refused Session instead of
/// undecodable traffic.
pub const LOCAL_PIX_SUITE: hopr_protocol_pix::PixSuite = <HoprPixSpec as PixSpec>::PIX_SUITE;

/// The three defaults and the local curve suite, as the single set both nodes must agree on.
///
/// The `match` is a compile-time range check: defaults that fall outside what
/// [`PixParams::try_new`] accepts fail the build rather than every Session.
pub(crate) const DEFAULT_PIX_PARAMS: PixParams = match PixParams::try_new(
    DEFAULT_PIX_POLYS_PER_SSA,
    DEFAULT_PIX_SHARES_PER_POLY,
    DEFAULT_PIX_SURPLUS_SHARES,
    LOCAL_PIX_SUITE,
) {
    Ok(params) => params,
    Err(_) => panic!("default PIX dimensions must be within the protocol ranges"),
};

/// Nominal per-SSA data quota implied by the default PIX dimensions.
///
/// This is the amount of Exit → Entry data covered by a single SSA deposit when both
/// nodes run the default PIX configuration.
pub const DEFAULT_PIX_SSA_QUOTA: SsaQuota = pix_params_to_quota(&DEFAULT_PIX_PARAMS);

/// Divisor applied to [`DEFAULT_PIX_SSA_QUOTA`] to obtain the lower bound of the default
/// [`quota_range`](crate::IncomingSessionPixConfig::quota_range).
///
/// Preserves the 4× span the range had when its bounds were hard-coded.
pub(crate) const DEFAULT_PIX_QUOTA_RANGE_SPAN: SsaQuota = 4;

/// Representation of a data quota per SSA agreed upon during the Session establishment.
///
/// No longer `Copy`: [`deposit_data`](Self::deposit_data) owns its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgreedSsaQuota {
    /// ID of the SSA.
    pub ssa_id: SsaId<HoprPseudonym>,
    /// Deposit address of the SSA.
    pub deposit_address: <HoprPixSpec as PixSpec>::DepositAddress,
    /// Quota of the SSA in bytes.
    pub quota_per_ssa: SsaQuota,
    /// Deposit data the pool produced for this SSA.
    ///
    /// Both nodes end up holding the same value by different routes: the Entry rebuilds it from the
    /// `SsaRequest` it just decoded, the Exit recalls what it sent for this index. Empty when the pool
    /// produced nothing for this SSA — the field is not optional, because a pool that has no deposit
    /// data to attach and one that failed to attach it are the same thing to a reader, and
    /// [`PixDepositData::is_empty`] already says which it is.
    pub deposit_data: PixDepositData,
}

/// Events raised by the [`crate::manager::SessionManager`] in response to received PIX messages.
#[derive(Debug)]
pub enum HoprSessionOutPixEvent {
    /// Event raised by the [`crate::manager::SessionManager`] of an Entry node can deposit funds to an SSA for the
    /// agreed data quota.
    ReadyToDeposit(AgreedSsaQuota),
    /// Event raised by the [`crate::manager::SessionManager`] of an Exit node, whenever it knows a new SSA and expects
    /// funds to be deposited.
    ///
    /// The attached sender is used to deliver updates once the deposit is completed.
    DepositNeeded(AgreedSsaQuota, hopr_api::node::DepositUpdated),
    /// Event raised by the [`crate::manager::SessionManager`] of an Exit node before it requests
    /// commitments for a batch of SSAs, asking the deposit pool for the data to attach to them.
    ///
    /// The Exit waits [`DEPOSIT_DATA_REQUEST_TIMEOUT`](crate::DEPOSIT_DATA_REQUEST_TIMEOUT) for one
    /// answer per requested id, and a shortfall is *fatal*: the SSA request fails with
    /// [`MissingDepositData`](crate::errors::SessionManagerError::MissingDepositData) and the Session
    /// is closed with [`ClosureReason::MissingDepositData`]. A pool with nothing to attach must
    /// therefore answer with *empty* data rather than stay silent — empty is a value, silence is not.
    ///
    /// A listener that cannot answer at all should drop the attached sender: that ends the wait
    /// immediately and fails the request now, whereas holding it costs every SSA request the full
    /// timeout before failing anyway.
    DepositDataRequest(hopr_api::node::PixDepositDataRequest),
}

/// Events received by the [`crate::manager::SessionManager`] in reaction to received shares from the packet pipeline.
#[derive(Debug, Clone)]
pub enum HoprSessionInPixEvent {
    /// Informs the [`crate::manager::SessionManager`] that an SSA was fully recovered.
    SsaRecovered(SsaId<HoprPseudonym>),
    /// Informs the [`crate::manager::SessionManager`] that the early recovery threshold was reached
    /// for an SSA — the next SSA request can be made.
    SsaAlmostRecovered(SsaId<HoprPseudonym>),
    /// Informs the [`crate::manager::SessionManager`] that unverifiable shares were encountered.
    UnverifiableShare(SsaId<HoprPseudonym>),
}

impl HoprSessionInPixEvent {
    /// Extracts the pseudonym of the SSA that might map to an existing Session.
    pub fn pseudonym(&self) -> &HoprPseudonym {
        match self {
            HoprSessionInPixEvent::SsaRecovered(ssa_id) => ssa_id.pseudonym(),
            HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id) => ssa_id.pseudonym(),
            HoprSessionInPixEvent::UnverifiableShare(ssa_id) => ssa_id.pseudonym(),
        }
    }
}

/// Constant application tag used for all sessions.
/// Previously tags were dynamically allocated per session.
pub const SESSION_APPLICATION_TAG: Tag = Tag::Reserved(ReservedTag::Session as u64);

/// [`SessionId`], [`ServiceId`], and [`SessionTarget`] are provided by `hopr-types` and
/// re-exported here via `hopr-utils`, so they match the published `hopr-api` session types.
///
/// - `SessionId` is a type alias for `HoprPseudonym` (a constant application tag is used for all sessions instead
///   of dynamically allocating tags).
/// - `ServiceId` identifies a service local to the Exit node (e.g. Cover Traffic).
/// - `SessionTarget` describes where data received over the session is forwarded.
pub use hopr_utils::network_types::types::{ServiceId, SessionId, SessionTarget};

pub(crate) fn caps_to_ack_mode(caps: Capabilities) -> AcknowledgementMode {
    if caps.contains(Capability::RetransmissionAck | Capability::RetransmissionNack) {
        AcknowledgementMode::Both
    } else if caps.contains(Capability::RetransmissionAck) {
        AcknowledgementMode::Full
    } else {
        AcknowledgementMode::Partial
    }
}

/// Indicates the closure reason of a [`HoprSession`].
///
/// Delivered to the `on_close` callback a Session is constructed with — see
/// [`HoprSession::new`] — and to the `SessionManager`'s closure notifier, so a caller has to be able
/// to name this type to write either.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
pub enum ClosureReason {
    /// Write-half of the Session has been closed.
    WriteClosed,
    /// Read-part of the Session has been closed (encountered empty read).
    EmptyRead,
    /// Session has been evicted from the cache due to inactivity or capacity reasons.
    Eviction,
    /// Deposit to an SSA has not been made on-time on a PIX-enabled Session.
    UnrealizedDeposit,
    /// The local deposit pool did not supply the deposit data an SSA batch needs.
    ///
    /// Exit-side and locally caused, unlike [`UnrealizedDeposit`](Self::UnrealizedDeposit) which is
    /// the Entry failing to pay. Kept apart from it for exactly that reason: the two look alike from
    /// the outside — a PIX Session that stopped — and point at opposite nodes.
    MissingDepositData,
}

/// Helper trait to allow Box aliasing
trait AsyncReadWrite: futures::AsyncWrite + futures::AsyncRead + Send + Unpin {}
impl<T: futures::AsyncWrite + futures::AsyncRead + Send + Unpin> AsyncReadWrite for T {}

/// Wrapper for an incoming [`HoprSession`] carrying the [`SessionId`] and [`SessionTarget`]
/// extracted from the Start protocol during session establishment.
///
/// This is the published generic [`hopr_api::node::IncomingSession`] specialized to the
/// concrete [`HoprSession`] byte-stream.
pub type IncomingSession = hopr_api::node::IncomingSession<HoprSession>;

/// Configures the Session protocol socket over HOPR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, smart_default::SmartDefault, serde::Serialize)]
pub struct HoprSessionConfig {
    /// Capabilities of the Session protocol socket.
    ///
    /// Default is no capabilities.
    #[default(Capabilities::empty())]
    pub capabilities: Capabilities,
    /// Expected frame size of the Session protocol socket.
    ///
    /// Default is 1500.
    #[default(1500)]
    pub frame_mtu: usize,
    /// Maximum amount of time an incomplete frame can be kept in the buffer.
    ///
    /// Default is 800 ms
    #[default(Duration::from_millis(800))]
    #[serde(with = "humantime_serde")]
    pub frame_timeout: Duration,
    /// Maximum number of segments to buffer in the downstream transport.
    /// If 0 is given, the transport is unbuffered.
    ///
    /// Default is 0.
    #[default(0)]
    pub max_buffered_segments: usize,
    /// Abandon the frame due next once this many later frames are waiting behind it.
    ///
    /// Head-of-line bound, distinct from [`Self::frame_timeout`]: that one waits for a frame that
    /// may still arrive, this bounds how much already-received data is held while it waits. `None`
    /// keeps the timeout as the only rule.
    pub max_frames_behind_gap: Option<usize>,
}

/// Represents the Session protocol socket over HOPR.
///
/// This is essentially a HOPR-specific wrapper for [`ReliableSocket`] and [`UnreliableSocket`]
/// Session protocol sockets.
#[pin_project::pin_project]
pub struct HoprSession {
    id: SessionId,
    #[pin]
    inner: Box<dyn AsyncReadWrite>,
    routing: DestinationRouting,
    cfg: HoprSessionConfig,
    on_close: Option<Box<dyn FnOnce(SessionId, ClosureReason) + Send + Sync>>,
}

pub(crate) const SESSION_SOCKET_CAPACITY: usize = 16384;

impl HoprSession {
    /// Creates a new HOPR Session.
    ///
    /// It builds an [`futures::io::AsyncRead`] + [`futures::io::AsyncWrite`] transport
    /// from the given `hopr` interface and passing it to the appropriate [`UnreliableSocket`] or [`ReliableSocket`]
    /// based on the given `capabilities`.
    ///
    /// The `on_close` closure can be optionally called when the Session has been closed via `poll_close`.
    #[tracing::instrument(skip_all, fields(id, routing, cfg, session_id = %id))]
    pub fn new<Tx, Rx>(
        id: SessionId,
        routing: DestinationRouting,
        cfg: HoprSessionConfig,
        hopr: (Tx, Rx),
        on_close: Option<Box<dyn FnOnce(SessionId, ClosureReason) + Send + Sync>>,
    ) -> Result<Self, TransportSessionError>
    where
        Tx: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Send + Unpin + 'static,
        Rx: futures::Stream<Item = ApplicationDataIn> + Send + Unpin + 'static,
        Tx::Error: std::error::Error + Send + Sync,
    {
        Self::new_with_surb_state(id, routing, cfg, hopr, on_close, None, None)
    }

    /// Like [`new`](Self::new) but threads the SURB balancer state and the opt-in client-side
    /// flow-control config. `flow_control` = the client's [`FlowControlConfig`] for this session
    /// (`None` leaves it unpaced); `surb_mgmt` gives the window its anti-grief down-only SURB ceiling.
    /// The entry (sending) side passes `Some(..)`; sites without them pass `None`.
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip_all, fields(id, routing, cfg, session_id = %id))]
    pub fn new_with_surb_state<Tx, Rx>(
        id: SessionId,
        routing: DestinationRouting,
        cfg: HoprSessionConfig,
        hopr: (Tx, Rx),
        on_close: Option<Box<dyn FnOnce(SessionId, ClosureReason) + Send + Sync>>,
        surb_mgmt: Option<Arc<BalancerStateValues>>,
        flow_control: Option<FlowControlConfig>,
    ) -> Result<Self, TransportSessionError>
    where
        Tx: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Send + Unpin + 'static,
        Rx: futures::Stream<Item = ApplicationDataIn> + Send + Unpin + 'static,
        Tx::Error: std::error::Error + Send + Sync,
    {
        let routing_clone = routing.clone();

        #[cfg(feature = "telemetry")]
        let (session_id_write, session_id_read) = (id, id);

        // Wrap the HOPR transport so that it appears as regular transport to the SessionSocket
        let transport = DuplexIO(
            AsyncWriteSink::<{ ApplicationData::PAYLOAD_SIZE }, _>(hopr.0.sink_map_err(std::io::Error::other).with(
                move |buf: Box<[u8]>| {
                    #[cfg(feature = "telemetry")]
                    crate::telemetry::record_session_write(&session_id_write, buf.len());
                    // The Session protocol does not set any packet info on outgoing packets.
                    // However, the SessionManager on top usually overrides this.
                    futures::future::ready(
                        ApplicationData::new(SESSION_APPLICATION_TAG, buf.into_vec())
                            .map(|data| (routing_clone.clone(), ApplicationDataOut::with_no_packet_info(data)))
                            .map_err(std::io::Error::other),
                    )
                },
            )),
            // The Session protocol ignores the packet info on incoming packets.
            // It is typically SessionManager's job to interpret those.
            hopr.1
                .map(move |data| {
                    #[cfg(feature = "telemetry")]
                    crate::telemetry::record_session_read(&session_id_read, data.data.plain_text.len());
                    Ok::<_, std::io::Error>(data.data.plain_text)
                })
                .into_async_read(),
        );

        // Based on the requested capabilities, see if we should use the Session protocol
        let inner: Box<dyn AsyncReadWrite> = if cfg.capabilities.contains(Capability::Segmentation) {
            let socket_cfg = SessionSocketConfig {
                frame_size: cfg.frame_mtu,
                frame_timeout: cfg.frame_timeout,
                capacity: SESSION_SOCKET_CAPACITY,
                flush_immediately: cfg.capabilities.contains(Capability::NoDelay),
                max_buffered_segments: cfg.max_buffered_segments,
                // Anti-bufferbloat bound; only meaningful when flow control is enabled, which is
                // also where the honest clock that observes the resulting loss lives.
                max_frame_age: flow_control.and_then(|c| c.max_frame_age),
                // Head-of-line bound, and deliberately *not* gated on flow control the way
                // `max_frame_age` is. The reasoning there runs backwards for this one: a session
                // that can retransmit may still recover a missing frame, so waiting is
                // productive, while a session without retransmission is waiting for something
                // that is never coming and holds everything already received behind it. The
                // sessions that need this bound most are exactly the ones flow control excludes.
                max_frames_behind_gap: cfg.max_frames_behind_gap,
                ..Default::default()
            };

            // Need to test the capabilities separately, because any Retransmission capability
            // implies Segmentation, and therefore `is_disjoint` would fail
            if cfg.capabilities.contains(Capability::RetransmissionAck)
                || cfg.capabilities.contains(Capability::RetransmissionNack)
            {
                let fc = flow_control;

                // TODO: update config values
                let ack_cfg = AcknowledgementStateConfig {
                    // This is a very coarse assumption, that a single 3-hop packet
                    // takes on average 200 ms to deliver.
                    // We can no longer base this timeout on the number of hops because
                    // it is not known for SURB-based routing.
                    expected_packet_latency: Duration::from_millis(200),
                    mode: caps_to_ack_mode(cfg.capabilities),
                    backoff_base: 0.2,
                    max_incoming_frame_retries: 1,
                    // Under flow control the sender is paced to the SURB drain rate, so an un-acked
                    // frame is usually just a *delayed* ack on a temporarily-starved return path, not
                    // a genuine loss. The retry budget is a flow-control config knob (`frame_retries`,
                    // default 2 = original); a robust profile raises it so delayed frames recover
                    // instead of being abandoned (an abandoned frame leaves a gap → stream corruption).
                    // `.max(1)`: never drop the retry budget to 0 — an abandoned frame under
                    // reliable-mode flow control leaves a gap and corrupts the stream.
                    max_outgoing_frame_retries: fc.map(|c| c.frame_retries.max(1) as usize).unwrap_or(2),
                    // Retire an outgoing frame that is already too stale to be worth delivering,
                    // rather than spending the remaining retry budget on it.
                    max_frame_age: fc.and_then(|c| c.max_frame_age),
                    ..Default::default()
                };

                debug!(
                    ?socket_cfg,
                    ?ack_cfg,
                    flow_control = fc.is_some(),
                    "opening new stateful session socket"
                );

                // Opt-in client-side flow control: when enabled, install the honest-clock tap on the
                // ack state and keep the paired clock to drive the paced writer.
                let (ack_state, flow_control) = match fc {
                    Some(fc_cfg) => {
                        let meter = DeliveryMeter::default();
                        let ack_state = AcknowledgementState::<{ ApplicationData::PAYLOAD_SIZE }>::new(id, ack_cfg)
                            .with_delivery_tap(DeliveryTap::new(meter.clone(), cfg.frame_mtu));
                        let clock = DeliveryClock::new(meter, Some(ack_cfg.expected_packet_latency));
                        (ack_state, Some((fc_cfg, clock)))
                    }
                    None => (
                        AcknowledgementState::<{ ApplicationData::PAYLOAD_SIZE }>::new(id, ack_cfg),
                        None,
                    ),
                };

                let socket = ReliableSocket::new(
                    transport,
                    ack_state,
                    socket_cfg,
                    #[cfg(feature = "telemetry")]
                    NoopTracker,
                )?;

                match flow_control {
                    Some((fc_cfg, clock)) => {
                        let surb_state = surb_mgmt
                            .clone()
                            .unwrap_or_else(|| Arc::new(BalancerStateValues::default()));
                        let supply = SurbSupply::new(surb_state, cfg.frame_mtu);
                        debug!(?fc_cfg, "wrapping session socket with paced flow-control writer");
                        Box::new(PacedWriter::new(socket, fc_cfg, clock, supply))
                    }
                    None => Box::new(socket),
                }
            } else {
                debug!(?socket_cfg, "opening new stateless session socket");

                Box::new(UnreliableSocket::<{ ApplicationData::PAYLOAD_SIZE }>::new_stateless(
                    id,
                    transport,
                    socket_cfg,
                    #[cfg(feature = "telemetry")]
                    NoopTracker,
                )?)
            }
        } else {
            debug!("opening raw session socket");
            Box::new(transport)
        };

        Ok(Self {
            id,
            inner,
            routing,
            cfg,
            on_close,
        })
    }

    /// ID of this Session.
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Routing options used to deliver data.
    pub fn routing(&self) -> &DestinationRouting {
        &self.routing
    }

    /// Configuration of this Session.
    pub fn config(&self) -> &HoprSessionConfig {
        &self.cfg
    }
}

impl std::fmt::Debug for HoprSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("routing", &self.routing)
            .finish_non_exhaustive()
    }
}

impl futures::AsyncRead for HoprSession {
    #[instrument(name = "Session::poll_read", level = "trace", skip_all, fields(session_id = %self.id), ret)]
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        let read = futures::ready!(this.inner.poll_read(cx, buf))?;
        if read == 0 {
            tracing::trace!("hopr session empty read");
            // Empty read signals end of the socket, notify if needed
            if let Some(notifier) = this.on_close.take() {
                tracing::trace!("notifying read half closure of session");
                notifier(*this.id, ClosureReason::EmptyRead);
            }
        }
        Poll::Ready(Ok(read))
    }
}

impl futures::AsyncWrite for HoprSession {
    #[instrument(name = "Session::poll_write", level = "trace", skip_all, fields(session_id = %self.id), ret)]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }

    #[instrument(name = "Session::poll_flush", level = "trace", skip_all, fields(session_id = %self.id), ret)]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    #[instrument(name = "Session::poll_close", level = "trace", skip_all, fields(session_id = %self.id), ret)]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();
        futures::ready!(this.inner.poll_close(cx))?;
        tracing::trace!("hopr session closed");

        #[cfg(feature = "telemetry")]
        crate::telemetry::set_session_state(this.id, crate::telemetry::SessionLifecycleState::Closing);

        if let Some(notifier) = this.on_close.take() {
            tracing::trace!("notifying write half closure of session");
            notifier(*this.id, ClosureReason::WriteClosed);
        }

        Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "runtime-tokio")]
impl tokio::io::AsyncRead for HoprSession {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let slice = buf.initialize_unfilled();
        let n = std::task::ready!(futures::AsyncRead::poll_read(self.as_mut(), cx, slice))?;
        buf.advance(n);
        Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "runtime-tokio")]
impl tokio::io::AsyncWrite for HoprSession {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        futures::AsyncWrite::poll_write(self.as_mut(), cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        futures::AsyncWrite::poll_flush(self.as_mut(), cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        futures::AsyncWrite::poll_close(self.as_mut(), cx)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use futures::{AsyncReadExt, AsyncWriteExt};
    use hopr_api::types::{
        crypto::prelude::*,
        crypto_random::Randomizable,
        internal::{prelude::HoprPseudonym, routing::RoutingOptions},
        primitive::prelude::*,
    };
    use hopr_utils::network_types::prelude::SealedHost;

    use super::*;

    fn deposit_entry(session_id: &SessionId, ssa_index: u32, data: &[u8]) -> PixDepositData {
        PixDepositData {
            id: hopr_api::node::PixAddressId::new(
                session_id,
                hopr_protocol_pix::SsaIndex::new(ssa_index).expect("non-zero"),
            ),
            data: data.into(),
        }
    }

    /// The pool answers a batch as a flat list, in whatever order it produced the entries, so the
    /// transform has to key them itself — and pair each payload with the index its own `id` names,
    /// not with the position it arrived in.
    #[test]
    fn deposit_data_for_batch_should_key_payloads_by_their_own_ssa_index() -> anyhow::Result<()> {
        let session_id: SessionId = HoprPseudonym::random();
        let batch = [
            hopr_protocol_pix::SsaIndex::new(7).expect("non-zero"),
            hopr_protocol_pix::SsaIndex::new(8).expect("non-zero"),
        ];

        // Deliberately not in batch order: nothing promises the channel delivers them sorted.
        let out = deposit_data_for_batch(
            &session_id,
            &batch,
            vec![
                deposit_entry(&session_id, 8, b"eight"),
                deposit_entry(&session_id, 7, b"seven"),
            ],
        )?;

        assert_eq!(2, out.len());
        assert_eq!(
            Some(&HoprPixDepositPayload(b"seven".as_slice().into())),
            out.get(&batch[0])
        );
        assert_eq!(
            Some(&HoprPixDepositPayload(b"eight".as_slice().into())),
            out.get(&batch[1])
        );

        Ok(())
    }

    /// An answer must not be credited to an SSA it does not name, even when the index alone matches:
    /// a `PixAddressId` is the pseudonym *and* the index, and only both together identify the SSA.
    #[test]
    fn deposit_data_for_batch_should_drop_entries_outside_the_batch() -> anyhow::Result<()> {
        let session_id: SessionId = HoprPseudonym::random();
        let other_session: SessionId = HoprPseudonym::random();
        let batch = [hopr_protocol_pix::SsaIndex::new(7).expect("non-zero")];

        let out = deposit_data_for_batch(
            &session_id,
            &batch,
            vec![
                // Right Session, index the batch does not contain.
                deposit_entry(&session_id, 9, b"not in batch"),
                // Right index, but belonging to another Session.
                deposit_entry(&other_session, 7, b"wrong session"),
                deposit_entry(&session_id, 7, b"good"),
            ],
        )?;

        assert_eq!(1, out.len());
        assert_eq!(
            Some(&HoprPixDepositPayload(b"good".as_slice().into())),
            out.get(&batch[0])
        );

        Ok(())
    }

    /// An SSA with no answer fails the batch: the data only ever travels in the `SsaRequest` that
    /// carries the commitments, so an Entry that needed it would have no way to obtain it later.
    #[test]
    fn deposit_data_for_batch_should_reject_a_short_answer() -> anyhow::Result<()> {
        let session_id: SessionId = HoprPseudonym::random();
        let batch = [
            hopr_protocol_pix::SsaIndex::new(1).expect("non-zero"),
            hopr_protocol_pix::SsaIndex::new(2).expect("non-zero"),
        ];

        // One of two answered.
        assert!(matches!(
            deposit_data_for_batch(&session_id, &batch, vec![deposit_entry(&session_id, 2, b"only one")]),
            Err(SessionManagerError::MissingDepositData(_))
        ));

        // Nothing answered at all — the no-pool case.
        assert!(matches!(
            deposit_data_for_batch(&session_id, &batch, Vec::new()),
            Err(SessionManagerError::MissingDepositData(_))
        ));

        // An entry that is present but *empty* is an answer, not an absence: a pool with nothing to
        // attach says so this way, and the batch travels.
        let out = deposit_data_for_batch(
            &session_id,
            &batch,
            vec![deposit_entry(&session_id, 1, b""), deposit_entry(&session_id, 2, b"")],
        )?;
        assert_eq!(2, out.len());
        assert!(out.values().all(|payload| payload.0.is_empty()));

        Ok(())
    }

    // --- PIX quota tests ---

    /// The quota must count what the generator emits, not what the Exit needs to recover.
    ///
    /// This is one half of "paid-for equals served". The other half is
    /// `drain_shares_by_polynomial` in `hopr-protocol-pix`, which pins a cycle's emission at
    /// `polys × (threshold + surplus)` — the same expression. If one moves without the other, the
    /// Exit is either serving unpaid traffic or charging for traffic it never sends.
    #[test]
    fn quota_must_price_every_share_a_cycle_emits() -> anyhow::Result<()> {
        for (polys, threshold, surplus) in [(1u16, 2u8, 0u8), (8, 2, 2), (8192, 64, 32), (16192, 255, 255)] {
            let params = PixParams::try_new(polys, threshold, surplus, LOCAL_PIX_SUITE)?;
            assert_eq!(
                polys as u64 * (threshold as u64 + surplus as u64) * HoprPacket::PAYLOAD_SIZE as u64,
                pix_params_to_quota(&params),
                "the quota must cover every share the cycle emits"
            );
        }
        Ok(())
    }

    /// A non-zero surplus must cost something. Before it was priced, these two were equal — which
    /// is exactly the state this guards against returning to.
    #[test]
    fn a_surplus_must_raise_the_quota_above_the_threshold_alone() -> anyhow::Result<()> {
        let threshold_only = pix_params_to_quota(&PixParams::try_new(8192, 64, 0, LOCAL_PIX_SUITE)?);
        let with_surplus = pix_params_to_quota(&PixParams::try_new(8192, 64, 32, LOCAL_PIX_SUITE)?);

        assert!(
            with_surplus > threshold_only,
            "a surplus of 32 must be charged for, got {with_surplus} against {threshold_only}"
        );
        assert_eq!(
            threshold_only * 3 / 2,
            with_surplus,
            "a surplus of half the threshold must cost half the threshold's quota again"
        );

        // And the same, at the surplus this node actually defaults to: `threshold / 4`, i.e. a
        // deployed factor of 1.25x rather than the 1.5x this test used to describe as "deployed".
        let deployed = pix_params_to_quota(&DEFAULT_PIX_PARAMS);
        assert_eq!(threshold_only * 5 / 4, deployed, "the deployed surplus factor is 1.25x");
        Ok(())
    }

    /// The defaults the Exit's `quota_range` is anchored on are the ones an Entry actually runs.
    #[test]
    fn default_quota_must_follow_the_default_dimensions() {
        assert_eq!(
            DEFAULT_PIX_POLYS_PER_SSA as u64
                * (DEFAULT_PIX_SHARES_PER_POLY as u64 + DEFAULT_PIX_SURPLUS_SHARES as u64)
                * HoprPacket::PAYLOAD_SIZE as u64,
            DEFAULT_PIX_SSA_QUOTA
        );
        // Anchored on the ratio function rather than on a restatement of it. This line used to read
        // `DEFAULT_PIX_SHARES_PER_POLY / 2`, which was the ratio before it became `threshold / 4`;
        // restating it is precisely how it came to disagree with the constant it checks.
        assert_eq!(
            DEFAULT_PIX_SURPLUS_SHARES,
            hopr_protocol_pix::default_surplus_for(DEFAULT_PIX_SHARES_PER_POLY)
        );
    }

    // --- ByteCapabilities tests ---

    #[test]
    fn byte_capabilities_roundtrip_via_u8() -> anyhow::Result<()> {
        let flags: Capabilities = Capability::Segmentation.into();
        let caps = HoprSessionCapabilities::from(flags);
        let byte_val: u8 = caps.into();
        let restored = HoprSessionCapabilities::try_from(byte_val)?;
        assert_eq!(caps, restored);
        Ok(())
    }

    #[test]
    fn byte_capabilities_invalid_bits_are_rejected() {
        // 0xFF has bits set that don't correspond to any Capability
        assert!(HoprSessionCapabilities::try_from(0xFF_u8).is_err());
    }

    #[test]
    fn byte_capabilities_empty_is_zero() {
        let caps = HoprSessionCapabilities::from(Capabilities::empty());
        let byte_val: u8 = caps.into();
        assert_eq!(byte_val, 0);
    }

    #[test]
    fn byte_capabilities_combined_flags() -> anyhow::Result<()> {
        let caps: Capabilities = Capability::Segmentation | Capability::NoRateControl;
        let byte_caps = HoprSessionCapabilities::from(caps);
        let byte_val: u8 = byte_caps.into();
        let restored = HoprSessionCapabilities::try_from(byte_val)?;
        assert_eq!(*restored.as_ref(), caps);
        Ok(())
    }

    // --- caps_to_ack_mode tests ---

    #[test]
    fn caps_to_ack_mode_both_when_ack_and_nack() {
        let caps: Capabilities = Capability::RetransmissionAck | Capability::RetransmissionNack;
        assert_eq!(caps_to_ack_mode(caps), AcknowledgementMode::Both);
    }

    #[test]
    fn caps_to_ack_mode_full_when_only_ack() {
        let caps: Capabilities = Capability::RetransmissionAck.into();
        assert_eq!(caps_to_ack_mode(caps), AcknowledgementMode::Full);
    }

    #[test]
    fn caps_to_ack_mode_partial_when_no_retransmission() {
        let caps: Capabilities = Capability::Segmentation.into();
        assert_eq!(caps_to_ack_mode(caps), AcknowledgementMode::Partial);
    }

    #[test]
    fn caps_to_ack_mode_partial_when_empty() {
        assert_eq!(caps_to_ack_mode(Capabilities::empty()), AcknowledgementMode::Partial);
    }

    #[test]
    fn caps_to_ack_mode_should_be_partial_when_only_nack() {
        let caps: Capabilities = Capability::RetransmissionNack.into();
        assert_eq!(caps_to_ack_mode(caps), AcknowledgementMode::Partial);
    }

    // --- ClosureReason tests ---

    #[test]
    fn closure_reason_display_values_are_stable() {
        let reasons = [
            ClosureReason::WriteClosed,
            ClosureReason::EmptyRead,
            ClosureReason::Eviction,
            ClosureReason::UnrealizedDeposit,
            ClosureReason::MissingDepositData,
        ];
        insta::assert_debug_snapshot!(reasons);
    }

    // --- HoprSessionConfig tests ---

    #[test]
    fn hopr_session_config_default_snapshot() {
        let cfg = HoprSessionConfig::default();
        insta::assert_yaml_snapshot!(cfg);
    }

    // --- SessionTarget tests ---

    #[test]
    fn session_target_variants_debug_snapshot() -> anyhow::Result<()> {
        let targets: Vec<SessionTarget> = vec![
            SessionTarget::UdpStream(SealedHost::Plain(
                "127.0.0.1:8080".parse().context("parsing UDP target")?,
            )),
            SessionTarget::TcpStream(SealedHost::Plain("10.0.0.1:443".parse().context("parsing TCP target")?)),
            SessionTarget::ExitNode(42),
        ];
        insta::assert_debug_snapshot!(targets);
        Ok(())
    }

    // --- SessionId edge cases ---

    #[test]
    fn session_id_display_and_debug_should_be_identical() {
        let id = HoprPseudonym::random();
        assert_eq!(format!("{id}"), format!("{id:?}"));
    }

    #[test]
    fn session_id_hash_eq_consistency() {
        use std::collections::HashSet;
        let pseudonym = HoprPseudonym::random();
        let id1: SessionId = pseudonym;
        let id2: SessionId = pseudonym;
        let id3: SessionId = HoprPseudonym::random();

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2));
        assert!(!set.contains(&id3), "different pseudonym should not be in the set");
    }

    // --- Existing tests ---

    #[test_log::test(tokio::test)]
    async fn test_session_bidirectional_flow_without_segmentation() -> anyhow::Result<()> {
        let dst: Address = (&ChainKeypair::random()).into();
        let id: SessionId = HoprPseudonym::random();
        const DATA_LEN: usize = 5000;

        let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

        let mut alice_session = HoprSession::new(
            id,
            DestinationRouting::forward_only(dst, RoutingOptions::Hops(0.try_into()?)),
            Default::default(),
            (
                alice_tx,
                alice_rx
                    .map(|(_, data)| ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                    .inspect(|d| debug!("alice rcvd: {}", d.data.total_len())),
            ),
            None,
        )?;

        let mut bob_session = HoprSession::new(
            id,
            DestinationRouting::Return(id.into()),
            Default::default(),
            (
                bob_tx,
                bob_rx
                    .map(|(_, data)| ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                    .inspect(|d| debug!("bob rcvd: {}", d.data.total_len())),
            ),
            None,
        )?;

        let alice_sent = hopr_api::types::crypto_random::random_bytes::<DATA_LEN>();
        let bob_sent = hopr_api::types::crypto_random::random_bytes::<DATA_LEN>();

        let mut bob_recv = [0u8; DATA_LEN];
        let mut alice_recv = [0u8; DATA_LEN];

        tokio::time::timeout(Duration::from_secs(1), alice_session.write_all(&alice_sent))
            .await
            .context("alice write failed")?
            .context("alice write timed out")?;
        alice_session.flush().await?;

        tokio::time::timeout(Duration::from_secs(1), bob_session.write_all(&bob_sent))
            .await
            .context("bob write failed")?
            .context("bob write timed out")?;
        bob_session.flush().await?;

        tokio::time::timeout(Duration::from_secs(1), bob_session.read_exact(&mut bob_recv))
            .await
            .context("bob read failed")?
            .context("bob read timed out")?;

        tokio::time::timeout(Duration::from_secs(1), alice_session.read_exact(&mut alice_recv))
            .await
            .context("alice read failed")?
            .context("alice read timed out")?;

        assert_eq!(&alice_sent, bob_recv.as_slice());
        assert_eq!(bob_sent, alice_recv);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_session_bidirectional_flow_with_segmentation() -> anyhow::Result<()> {
        let dst: Address = (&ChainKeypair::random()).into();
        let id: SessionId = HoprPseudonym::random();
        const DATA_LEN: usize = 5000;

        let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

        let mut alice_session = HoprSession::new(
            id,
            DestinationRouting::forward_only(dst, RoutingOptions::Hops(0.try_into()?)),
            HoprSessionConfig {
                capabilities: Capability::Segmentation.into(),
                ..Default::default()
            },
            (
                alice_tx,
                alice_rx
                    .map(|(_, data)| ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                    .inspect(|d| debug!("alice rcvd: {}", d.data.total_len())),
            ),
            None,
        )?;

        let mut bob_session = HoprSession::new(
            id,
            DestinationRouting::Return(id.into()),
            HoprSessionConfig {
                capabilities: Capability::Segmentation.into(),
                ..Default::default()
            },
            (
                bob_tx,
                bob_rx
                    .map(|(_, data)| ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                    .inspect(|d| debug!("bob rcvd: {}", d.data.total_len())),
            ),
            None,
        )?;

        let alice_sent = hopr_api::types::crypto_random::random_bytes::<DATA_LEN>();
        let bob_sent = hopr_api::types::crypto_random::random_bytes::<DATA_LEN>();

        let mut bob_recv = [0u8; DATA_LEN];
        let mut alice_recv = [0u8; DATA_LEN];

        tokio::time::timeout(Duration::from_secs(1), alice_session.write_all(&alice_sent))
            .await
            .context("alice write failed")?
            .context("alice write timed out")?;
        alice_session.flush().await?;

        tokio::time::timeout(Duration::from_secs(1), bob_session.write_all(&bob_sent))
            .await
            .context("bob write failed")?
            .context("bob write timed out")?;
        bob_session.flush().await?;

        tokio::time::timeout(Duration::from_secs(1), bob_session.read_exact(&mut bob_recv))
            .await
            .context("bob read failed")?
            .context("bob read timed out")?;

        tokio::time::timeout(Duration::from_secs(1), alice_session.read_exact(&mut alice_recv))
            .await
            .context("alice read failed")?
            .context("alice read timed out")?;

        assert_eq!(alice_sent, bob_recv);
        assert_eq!(bob_sent, alice_recv);

        Ok(())
    }
}
