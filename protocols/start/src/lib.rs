//! This crate defines the Start sub-protocol used for HOPR Session initiation and management.
//!
//! The Start protocol is used to establish Session as described in HOPR
//! [`RFC-0012`](https://github.com/hoprnet/rfc/tree/main/rfcs/RFC-0012-session-start-protocol).
//! and is implemented via the [`StartProtocol`] enum.
//!
//! The protocol is defined via generic arguments `I` (for Session ID), `T` (for Session Target),
//! `C` (for Session capabilities) and `G` (for Session Stealth Address commitment representation).
//!
//! Per `RFC-0012`, the types `I` and `T` are serialized/deserialized to the CBOR binary format
//! (see [`RFC7049`](https://datatracker.ietf.org/doc/html/rfc7049)) and therefore must implement
//! `serde::Serialize + serde::Deserialize`.
//! The capability type `C` must be expressible as a single unsigned byte.
//!
//! The `G` type is used to represent the Session Stealth Address commitment representation.
//! It is typically a [`PixGroupRepr`](hopr_protocol_pix::PixGroupRepr)
//!
//! See [`StartProtocol`] docs for the protocol diagram.

/// Contains errors raised by the Start protocol.
pub mod errors;

use hopr_crypto_packet::prelude::HoprPacket;
use hopr_protocol_app::prelude::{ApplicationData, ReservedTag, Tag};
use hopr_protocol_pix::{MAX_POLYS_PER_SSA, SsaCommitment};

use crate::errors::StartProtocolError;

/// Challenge that identifies a Start initiation protocol message.
pub type StartChallenge = u64;

/// Lists all Start protocol error reasons.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display, strum::FromRepr)]
pub enum StartErrorReason {
    /// Unknown error.
    Unknown = 0,
    /// No more slots are available at the recipient.
    NoSlotsAvailable = 1,
    /// Recipient is busy.
    Busy = 2,
    /// The recipient requires incentivization or the incentivization parameters are not acceptable.
    UnacceptablePixParams = 3,
}

/// Identifies which entity a [`StartErrorType`] refers to.
///
/// During session establishment (before `SessionEstablished` is sent), errors
/// refer back to the initiation [`StartChallenge`]. After the session is
/// established, errors refer to the established session by its `SessionId`.
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(derive(strum::FromRepr, strum::EnumCount), repr(u8))]
pub enum ErrorIdentifier<I> {
    /// The error relates to a pending session initiation identified by this challenge.
    Challenge(StartChallenge),
    /// The error relates to an established session identified by this session ID.
    SessionId(I),
}

/// Error message in the Start protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartErrorType<I> {
    /// Identifies the session initiation or established session this error refers to.
    pub identifier: ErrorIdentifier<I>,
    /// The [reason](StartErrorReason) of this error.
    pub reason: StartErrorReason,
}

/// The session initiation message of the Start protocol.
///
/// ## Generic parameters
/// - `T` is the session target
/// - `C` are session capabilities
///
/// The `additional_data` are set dependent on the `capabilities`
/// or set to `0x0000000000000000` to be ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartInitiation<T, C> {
    /// Random challenge for this initiation.
    pub challenge: StartChallenge,
    /// Target of the session, i.e., what should the other party do with the traffic.
    pub target: T,
    /// Requested capabilities of the session.
    ///
    /// This might also contain information required for the PIX protocol.
    pub capabilities: C,
    /// Additional options (might be `capabilities` dependent), ignored if `0x0000000000000000`.
    ///
    /// When PIX is offered, the upper 32 bits are the
    /// [`PixParams`](hopr_protocol_pix::PixParams) word — see
    /// [`PixParams::into_additional_data`](hopr_protocol_pix::PixParams::into_additional_data) —
    /// and the lower 32 bits are the SURB balancer target. The field is then fully allocated;
    /// there is no room left in it to negotiate anything further.
    pub additional_data: u64,
}

/// Message of the Start protocol that confirms the establishment of a session.
///
/// ## Generic parameters
/// `I` is for session identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartEstablished<I> {
    /// Challenge that was used in the [initiation message](StartInitiation) to establish correspondence.
    pub orig_challenge: StartChallenge,
    /// Session ID that was selected by the recipient.
    pub session_id: I,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Lists all messages of the Start protocol for a session establishment.
///
/// ## Generic parameters
/// - `I` is the session identifier.
/// - `T` is the session target.
/// - `C` are session capabilities.
/// - `G` is the type of the commitment to the Session Stealth Address (SSA).
/// - `K` is the wire form of the proof of knowledge accompanying a client SSA commitment.
///
/// # Diagram of the protocol
/// ```mermaid
/// sequenceDiagram
///      Entry->>Exit: SessionInitiation (Challenge)
///      alt If Exit can accept a new session
///      Note right of Exit: SessionID [Pseudonym, Tag]
///      Exit->>Entry: SessionEstablished (Challenge, SessionID)
///      Note left of Entry: SessionID [Pseudonym, Tag]
///      Exit->>Entry: SsaRequest (SessionID, SsaIndex, ServerCommitments)
///      Entry->>Exit: SsaCommit (SessionID, SsaIndex, CoeffIndex, PolyCoeffs)
///      Entry->>Exit: KeepAlive (SessionID)
///      Exit->>Entry: KeepAlive (SessionID)
///      Note over Entry,Exit: Data
///      else If Exit cannot accept a new session
///      Exit->>Entry: SessionError (Challenge, Reason)
///      end
///      opt If initiation attempt times out
///      Note left of Entry: Failure
///      end
/// ```
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(derive(strum::FromRepr, strum::EnumCount), repr(u8))]
pub enum StartProtocol<I, T, C, G, K, D> {
    /// Request to initiate a new session.
    StartSession(StartInitiation<T, C>),
    /// Confirmation that a new session has been established by the counterparty.
    SessionEstablished(StartEstablished<I>),
    /// Client's message to fill Client commitments to establish a Session Stealth Address (SSA).
    SsaCommit(SsaClientCommitmentMessage<I, G, K>),
    /// Server-side commitment to Session Stealth Address (SSA).
    SsaRequest(SsaServerCommitmentMessage<I, G, D>),
    /// Counterparty could not establish a new session due to an error.
    SessionError(StartErrorType<I>),
    /// A ping message to keep the session alive.
    KeepAlive(KeepAliveMessage<I>),
}

/// Filling up the Client's commitment to the Session Stealth Address (SSA).
///
/// The generic argument `G` typically represents a [`PixGroupRepr`](hopr_protocol_pix::PixGroupRepr).
///
/// The overall Client's commitment to a single new SSA usually requires multiple messages, all
/// sharing the same [`SsaIndex`](hopr_protocol_pix::SsaIndex).
///
/// Each of these messages contains commitments to polynomial coefficients that all belong
/// to the same coefficient in each polynomial.
///
/// In practice every message carries `coefficient_index == 0`: PIX commits to each polynomial's
/// constant term and nothing else, so the constant-term pass *is* the whole commitment. The field
/// (and phase 2 of [`new_multiple`](Self::new_multiple), which now has nothing to emit) is retained
/// because the wire format still admits higher coefficient indices; a peer that sends them merely
/// wastes bandwidth, since the receiver ignores them.
///
/// See [`hopr_protocol_pix::SsaPartCommitment`] for why the non-constant coefficient commitments
/// were dropped and what that costs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaClientCommitmentMessage<I, G, K> {
    /// Session ID.
    pub session_id: I,
    /// Index of the Session Stealth Address (SSA) that is being committed.
    pub ssa_index: hopr_protocol_pix::SsaIndex,
    /// Index of the polynomial coefficient that is being committed.
    ///
    /// Zero value indicates the polynomial constant term commitment, which when summed over
    /// all polynomials for a given [`SsaIndex`](hopr_protocol_pix::SsaIndex)
    /// results in the Client's SSA commitment.
    ///
    /// This is always zero in practice — see the type-level documentation.
    pub coefficient_index: u16,
    /// Proof that the Client knows the discrete logarithm of its SSA commitment.
    ///
    /// Present exactly when `coefficient_index == 0`: the commitment it opens is the *sum* of all
    /// constant terms, so it belongs with the messages that deliver them, and there is no
    /// separate presence flag on the wire — the coefficient index decides it. Every constant-term
    /// message carries it rather than only one, so no single lost packet can strand a cycle that
    /// would otherwise be recoverable, and the receiver keeps the first it sees.
    ///
    /// Without it the receiver cannot distinguish a genuine commitment from one crafted so the
    /// *combined* deposit key is known to the sender alone. See
    /// [`hopr_protocol_pix::SsaCommitmentProof`].
    pub commitment_proof: Option<K>,
    /// Contains the serialized coefficient commitments of multiple polynomials,
    /// all belonging to the same `coefficient_index` in each polynomial.
    ///
    /// This might not be the complete set yet and might require multiple messages to deliver
    /// the complete commitment to the given coefficient of all polynomials for the given SSA.
    pub coefficient_commitments: std::collections::HashMap<hopr_protocol_pix::PolynomialIndex, G>,
}

impl<I: serde::Serialize + Clone, G: Clone, K: Clone> SsaClientCommitmentMessage<I, G, K> {
    /// Uses given the `session_id` and an [`SsaCommitment`] that will be split across multiple messages.
    ///
    /// The returned messages are ordered by coefficient index, making sure the constant terms
    /// of the polynomials are delivered first.
    pub fn new_multiple<S: hopr_protocol_pix::PixSpec>(
        session_id: I,
        commitment: SsaCommitment<S, S::Pseudonym>,
    ) -> Result<Vec<Self>, StartProtocolError>
    where
        G: From<hopr_protocol_pix::PixGroupRepr<S>>,
        K: From<hopr_protocol_pix::SsaCommitmentProof<S>>,
    {
        let ssa_index = commitment.ssa_id.ssa_index();
        let commitment_proof = K::from(commitment.commitment_proof);

        // A single message can only carry a limited number of coefficient commitments so that the
        // resulting encoded message still fits within a HOPR packet payload. The commitments of a
        // single coefficient (across all polynomials) might therefore need to be split across
        // multiple messages. The encode layout this is derived from is documented on the helper.
        let SsaCommitChunking {
            max_commitments_per_message,
            max_constant_terms_per_message,
        } = StartProtocol::<I, (), (), G, K, ()>::ssa_commit_chunking(&session_id)?;

        // Group the transposed verifiers by coefficient index, each group sorted by polynomial
        // index. A `BTreeMap` keeps the coefficient order deterministic; the inner sort is what
        // lets a block of polynomials be addressed as the same slice range in every coefficient.
        let mut by_coefficient: std::collections::BTreeMap<u16, Vec<(hopr_protocol_pix::PolynomialIndex, G)>> =
            std::collections::BTreeMap::new();

        for (coefficient_index, coefficients) in commitment.verifiers {
            let entry = by_coefficient.entry(coefficient_index).or_default();
            for (poly_index, coefficient_commitment) in coefficients {
                entry.push((poly_index, G::from(coefficient_commitment)));
            }
        }
        for coefficients in by_coefficient.values_mut() {
            coefficients.sort_unstable_by_key(|(poly_index, _)| *poly_index);
        }

        let mut messages = Vec::new();
        let mut push_chunk = |coefficient_index: u16, chunk: &[(hopr_protocol_pix::PolynomialIndex, G)]| {
            messages.push(Self {
                session_id: session_id.clone(),
                ssa_index,
                coefficient_index,
                // Presence is decided by the coefficient index, matching what the decoder expects.
                commitment_proof: (coefficient_index == 0).then(|| commitment_proof.clone()),
                coefficient_commitments: chunk.iter().cloned().collect(),
            });
        };

        // Phase 1 — every polynomial's constant term, so the Exit can derive the SSA deposit
        // address as early as possible. This is the one coefficient that must be delivered across
        // *all* polynomials before anything else, because the address is their sum.
        let constant_terms = by_coefficient.remove(&0).unwrap_or_default();
        for chunk in constant_terms.chunks(max_constant_terms_per_message) {
            push_chunk(0, chunk);
        }

        // Phase 2 — any remaining coefficients, a block of polynomials at a time.
        //
        // Dead in practice: PIX commits to constant terms only, so `by_coefficient` is empty after
        // the `remove(&0)` above and this loop body never runs. It is kept because the wire format
        // still admits higher coefficient indices, and because the block-major layout is the
        // non-obvious part — emitting one polynomial's row at a time would waste most of each
        // packet, while walking coefficient-major would complete no polynomial until the very last
        // message.
        let num_polys = constant_terms
            .len()
            .max(by_coefficient.values().map(|c| c.len()).max().unwrap_or(0));
        for block_start in (0..num_polys).step_by(max_commitments_per_message) {
            for (&coefficient_index, coefficients) in &by_coefficient {
                if block_start >= coefficients.len() {
                    continue;
                }
                let block_end = (block_start + max_commitments_per_message).min(coefficients.len());
                push_chunk(coefficient_index, &coefficients[block_start..block_end]);
            }
        }

        Ok(messages)
    }
}

/// Sent by the Server to deliver the commitment to possibly multiple new Session Stealth Addresses (SSAs).
///
/// This message is typically sent for the first time right after the [`StartEstablished`] message
/// if PIX capabilities are indicated in the [`StartInitiation`] message, and the Server accepts it.
///
/// It is then subsequently sent every time the Server needs the next batch of SSAs
/// (with indices strictly greater than in the last batch) to be committed to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaServerCommitmentMessage<I, G, D> {
    /// Session ID.
    pub session_id: I,
    /// Parameters of the PIX protocol the server requires, packed by
    /// [`PixParams::to_u32`](hopr_protocol_pix::PixParams::to_u32).
    ///
    /// Deliberately the raw word rather than a [`PixParams`](hopr_protocol_pix::PixParams): the
    /// codec stays total, so an out-of-range value from a peer reaches the session layer and is
    /// answered with a `SessionError` instead of being dropped as an undecodable packet. Use
    /// [`dimensions`](Self::dimensions) to read it.
    pub params: u32,
    /// Deposit/payment data for the PIX session, carried in CBOR.
    ///
    /// Currently set to [`Default::default`]. Must be preserved through encode/decode.
    pub deposit_data: D,
    /// Server's serialized commitments to the SSAs, ordered by the SSA index.
    pub commitments: std::collections::BTreeMap<hopr_protocol_pix::SsaIndex, G>,
}

impl<I, G, D> SsaServerCommitmentMessage<I, G, D> {
    /// Convenience constructor for creating a new `SsaServerCommitmentMessage`.
    pub fn new(
        session_id: I,
        params: hopr_protocol_pix::PixParams,
        commitments: impl IntoIterator<Item = (hopr_protocol_pix::SsaIndex, G)>,
        deposit_data: D,
    ) -> Self {
        Self {
            session_id,
            params: params.to_u32(),
            deposit_data,
            commitments: commitments.into_iter().collect(),
        }
    }

    /// The PIX dimensions this request was made under, and the curve suite they are dimensions of.
    ///
    /// Fails if the peer packed values outside the protocol ranges, or named a curve suite that does
    /// not exist. The name predates the suite and is kept because the callers read it for the
    /// dimensions; the suite rides along because both sides must agree on it too.
    pub fn dimensions(&self) -> errors::Result<hopr_protocol_pix::PixParams> {
        hopr_protocol_pix::PixParams::try_from_u32(self.params)
            .map_err(|error| StartProtocolError::ParseError(format!("ssa_req.params: {error}")))
    }
}

/// Keep-alive message for a Session with the identifier `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeepAliveMessage<I> {
    /// Session ID.
    pub session_id: I,
    /// Additional flags that govern how the `additional_data` field is interpreted or 0.
    pub flags: KeepAliveFlags,
    /// Additional data (usually `flags` dependent), ignored if `0`.
    pub additional_data: u64,
}

/// [Flags](KeepAliveFlag) that can be sent via the [`KeepAliveMessage`].
///
/// The flags can define the meaning of the `additional_data` field.
pub type KeepAliveFlags = flagset::FlagSet<KeepAliveFlag>;

flagset::flags! {
    /// Individual flags that can be set in a [`KeepAliveMessage`].
    pub enum KeepAliveFlag: u8 {
        /// The `additional_data` field contains load balancer target information.
        ///
        /// The value of `additional_data` represents the optimal number of SURBs that the
        /// Session Initiator wishes to maintain at the Session Recipient.
        ///
        /// Mutually exclusive with `BalancerState`.
        BalancerTarget = 0x01,
        /// The `additional_data` field contains load balancer state information.
        ///
        /// The value of `additional_data` represents the current number of SURBs
        /// that the Session Recipient estimates to have.
        ///
        /// Mutually exclusive with `BalancerTarget`.
        BalancerState = 0x02,
    }
}

impl<I> KeepAliveMessage<I> {
    /// The minimum number of SURBs a [`KeepAliveMessage`] must be able to carry.
    pub const MIN_SURBS_PER_MESSAGE: usize = HoprPacket::MAX_SURBS_IN_PACKET;
}

impl<I> From<I> for KeepAliveMessage<I> {
    fn from(value: I) -> Self {
        Self {
            session_id: value,
            flags: None.into(),
            additional_data: 0,
        }
    }
}

impl<I, T, C, G, K, D> StartProtocol<I, T, C, G, K, D> {
    /// Maximum number of SSAs that can be requested in a single SsaRequest message.
    ///
    /// Derived from the SsaRequest encode layout with minimal CBOR deposit_data and session_id:
    /// header(4) + params(4) + deposit_data(1 for CBOR null) + num_commitments(2) = 11 overhead;
    /// (PAYLOAD_SIZE - 11) / (SsaIndex + commitment_repr) = (1030 - 11) / (4 + 33) = 27.
    /// Since a zero-length session_id is the smallest possible, any non-empty session_id
    /// only makes this bound tighter, making it a safe decode limit.
    pub const MAX_SSAS_PER_REQUEST: u16 = ((ApplicationData::PAYLOAD_SIZE - 11)
        / (size_of::<hopr_protocol_pix::SsaIndex>() + Self::PIX_COEFF_COMMITMENT_REPR_SIZE))
        as u16;
    /// Size of the PIX coefficient commitment representation in bytes.
    pub const PIX_COEFF_COMMITMENT_REPR_SIZE: usize = size_of::<G>();
    /// Size of the serialized client SSA commitment proof of knowledge in bytes.
    ///
    /// Carried only by constant-term `SsaCommit` messages, so it reduces the entry budget of those
    /// and of no others.
    pub const PIX_COMMITMENT_PROOF_SIZE: usize = size_of::<K>();
    /// Size of the Start protocol message header in bytes.
    pub const START_HEADER_SIZE: usize =
        size_of_val(&Self::START_PROTOCOL_MESSAGE_TAG) + size_of::<u8>() + size_of::<u16>();
    /// Fixed [`Tag`] of every protocol message.
    pub const START_PROTOCOL_MESSAGE_TAG: Tag = Tag::Reserved(ReservedTag::SessionStart as u64);
    /// Current version of the Start protocol.
    pub const START_PROTOCOL_VERSION: u8 = 0x03;

    /// How many commitment entries one [`SsaCommit`](StartProtocol::SsaCommit) message can carry,
    /// for each of the two delivery phases.
    ///
    /// This is the bound
    /// [`SsaClientCommitmentMessage::new_multiple`] chunks by, exposed so that a caller predicting
    /// the message count for a commitment does not have to restate the encode layout. Restating it
    /// is error-prone in a way that hides: the copy this replaced used `SsaIndex` (4 B) where an
    /// `SsaCommit` entry prefix is a `PolynomialIndex` (2 B), against a different fixed overhead,
    /// and agreed with the encoder only because the two mistakes floored to the same divisor at the
    /// dimensions it ran at.
    ///
    /// Takes the `session_id` itself rather than a precomputed length: its CBOR encoding is part of
    /// the layout, so leaving the caller to measure it is the same footgun one level up.
    pub fn ssa_commit_chunking(session_id: &I) -> Result<SsaCommitChunking, StartProtocolError>
    where
        I: serde::Serialize,
    {
        // A single message can only carry a limited number of coefficient commitments
        // so that the resulting encoded message still fits within a HOPR packet payload.
        // The commitments of a single coefficient (across all polynomials) might therefore
        // need to be split across multiple messages.
        //
        // The bound is derived from the actual SsaCommit encode layout:
        //   header:     version(1) + disc(1) + data_len(2) = 4
        //   fixed:      ssa_index(4) + coefficient_index(2) + poly_count(2) = 8
        //   proof:      PIX_COMMITMENT_PROOF_SIZE, constant-term messages only
        //   per-entry:  PolynomialIndex(2) + commitment_repr(PIX_COEFF_COMMITMENT_REPR_SIZE)
        //   trailer:    CBOR-encoded session_id
        let header_and_fixed: usize = 4 + 4 + 2 + 2; // header + ssa_index + coeff_index + num_polys
        let per_entry = size_of::<hopr_protocol_pix::PolynomialIndex>() + Self::PIX_COEFF_COMMITMENT_REPR_SIZE;
        // Compute the exact CBOR size of the session_id for this instantiation.
        let cbor_session_id_size = serde_cbor_2::to_vec(session_id)?.len();
        let budget = ApplicationData::PAYLOAD_SIZE.saturating_sub(header_and_fixed + cbor_session_id_size);

        Ok(SsaCommitChunking {
            max_commitments_per_message: (budget / per_entry).max(1),
            // Constant-term messages additionally carry the proof, so they fit fewer entries. Phase
            // 2 keeps the full budget, which is what the block size in `new_multiple` is aligned to;
            // only phase 1 pays for the proof, costing a handful of extra messages per cycle.
            max_constant_terms_per_message: (budget.saturating_sub(Self::PIX_COMMITMENT_PROOF_SIZE) / per_entry).max(1),
        })
    }
}

/// How many commitment entries one `SsaCommit` message can carry, per delivery phase.
///
/// Returned by [`StartProtocol::ssa_commit_chunking`]. The two are easy to confuse and differ only
/// by the proof of knowledge, so they are named rather than returned as a pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsaCommitChunking {
    /// Phase 2 — higher-coefficient messages, which get the full entry budget.
    pub max_commitments_per_message: usize,
    /// Phase 1 — constant-term messages, which also carry the proof of knowledge and so fit fewer.
    pub max_constant_terms_per_message: usize,
}

impl<I, T, C, G, K, D> StartProtocol<I, T, C, G, K, D>
where
    I: serde::Serialize,
    T: serde::Serialize,
    C: Into<u8>,
    G: AsRef<[u8]>,
    K: AsRef<[u8]>,
    D: serde::Serialize,
{
    /// Tries to encode the message into binary format and [`Tag`]
    pub fn encode(self) -> errors::Result<(Tag, Box<[u8]>)> {
        let mut out = Vec::with_capacity(ApplicationData::PAYLOAD_SIZE);
        out.push(Self::START_PROTOCOL_VERSION);
        out.push(StartProtocolDiscriminants::from(&self) as u8);

        let mut data = Vec::with_capacity(ApplicationData::PAYLOAD_SIZE - 2);
        match self {
            StartProtocol::StartSession(init) => {
                data.extend_from_slice(&init.challenge.to_be_bytes());
                data.push(init.capabilities.into());
                data.extend_from_slice(&init.additional_data.to_be_bytes());
                let target = serde_cbor_2::to_vec(&init.target)?;
                data.extend_from_slice(&target);
            }
            StartProtocol::SessionEstablished(est) => {
                data.extend_from_slice(&est.orig_challenge.to_be_bytes());
                let session_id = serde_cbor_2::to_vec(&est.session_id)?;
                data.extend(session_id);
            }
            StartProtocol::SessionError(err) => {
                match err.identifier {
                    ErrorIdentifier::Challenge(challenge) => {
                        data.push(ErrorIdentifierDiscriminants::Challenge as u8);
                        data.extend_from_slice(&challenge.to_be_bytes());
                    }
                    ErrorIdentifier::SessionId(id) => {
                        data.push(ErrorIdentifierDiscriminants::SessionId as u8);
                        let id_bytes = serde_cbor_2::to_vec(&id)?;
                        data.extend(id_bytes);
                    }
                }
                data.push(err.reason as u8);
            }
            StartProtocol::KeepAlive(ping) => {
                data.push(ping.flags.bits());
                data.extend_from_slice(&ping.additional_data.to_be_bytes());
                let session_id = serde_cbor_2::to_vec(&ping.session_id)?;
                data.extend(session_id);
            }
            StartProtocol::SsaCommit(commit) => {
                data.extend_from_slice(&commit.ssa_index.get().to_be_bytes());
                data.extend_from_slice(&commit.coefficient_index.to_be_bytes());

                let num_polys = commit.coefficient_commitments.len() as hopr_protocol_pix::PolynomialIndex;
                data.extend_from_slice(&num_polys.to_be_bytes());

                // The proof goes immediately after the fixed prefix, and only on constant-term
                // messages. Its presence is implied by `coefficient_index == 0` rather than by a
                // flag, so the two must agree — a mismatch would desynchronise the decoder.
                match (commit.coefficient_index, &commit.commitment_proof) {
                    (0, Some(proof)) => {
                        let proof = proof.as_ref();
                        if proof.len() != Self::PIX_COMMITMENT_PROOF_SIZE {
                            return Err(StartProtocolError::ParseError("commitment_proof_size".into()));
                        }
                        data.extend_from_slice(proof);
                    }
                    (0, None) => return Err(StartProtocolError::ParseError("missing commitment_proof".into())),
                    (_, Some(_)) => {
                        return Err(StartProtocolError::ParseError(
                            "commitment_proof on a non-constant term".into(),
                        ));
                    }
                    (_, None) => {}
                }

                let session_id = serde_cbor_2::to_vec(&commit.session_id)?;
                let total_coeff_commit_len = (size_of::<hopr_protocol_pix::PolynomialIndex>()
                    + Self::PIX_COEFF_COMMITMENT_REPR_SIZE)
                    * commit.coefficient_commitments.len();

                // Remaining payload budget: the final `out` buffer contains
                // version (1) + disc (1) + data_len (2) + data contents = 4 + data.len(),
                // which must fit within PAYLOAD_SIZE.  Check using explicit arithmetic
                // rather than Vec::spare_capacity_mut() which reflects pre-allocation.
                let avail_space = ApplicationData::PAYLOAD_SIZE.saturating_sub(4 + data.len() + session_id.len());
                if commit.coefficient_commitments.is_empty() || total_coeff_commit_len > avail_space {
                    return Err(StartProtocolError::NumberOfCommitments);
                }

                for (index, commitment) in commit.coefficient_commitments {
                    let commitment_repr = commitment.as_ref();
                    if commitment_repr.len() != Self::PIX_COEFF_COMMITMENT_REPR_SIZE {
                        return Err(StartProtocolError::ParseError("commitment_repr_size".into()));
                    }

                    // Prepending 16-bit representation of the polynomial index
                    // will possibly consume less space than putting an entire 1024-bit bitmap
                    // of polynomials present in each message (assuming 1024 polynomials per SSA).
                    data.extend_from_slice(&index.to_be_bytes());
                    data.extend_from_slice(commitment_repr);
                }

                data.extend(session_id);
            }
            StartProtocol::SsaRequest(req) => {
                data.extend_from_slice(&req.params.to_be_bytes());
                let deposit_data = serde_cbor_2::to_vec(&req.deposit_data)?;
                data.extend_from_slice(&deposit_data);

                let num_commitments = req.commitments.len() as u16;
                data.extend_from_slice(&num_commitments.to_be_bytes());

                let session_id = serde_cbor_2::to_vec(&req.session_id)?;

                let required_size = (size_of::<hopr_protocol_pix::SsaIndex>() + Self::PIX_COEFF_COMMITMENT_REPR_SIZE)
                    * req.commitments.len();

                // Remaining payload budget: the final `out` buffer contains
                // version (1) + disc (1) + data_len (2) + data contents = 4 + data.len(),
                // which must fit within PAYLOAD_SIZE.  Check using explicit arithmetic
                // rather than Vec::spare_capacity_mut() which reflects pre-allocation.
                let avail_space = ApplicationData::PAYLOAD_SIZE.saturating_sub(4 + data.len() + session_id.len());
                if req.commitments.is_empty() || required_size > avail_space {
                    return Err(StartProtocolError::NumberOfCommitments);
                }

                for (ssa_index, commitment) in req.commitments {
                    let commitment_repr = commitment.as_ref();
                    if commitment_repr.len() != Self::PIX_COEFF_COMMITMENT_REPR_SIZE {
                        return Err(StartProtocolError::ParseError("commitment_repr_size".into()));
                    }

                    data.extend_from_slice(&ssa_index.get().to_be_bytes());
                    data.extend_from_slice(commitment_repr);
                }

                data.extend(session_id);
            }
        }

        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend(data);

        Ok((Self::START_PROTOCOL_MESSAGE_TAG, out.into_boxed_slice()))
    }
}

impl<I, T, C, G, K, D> StartProtocol<I, T, C, G, K, D>
where
    I: for<'de> serde::Deserialize<'de>,
    T: for<'de> serde::Deserialize<'de>,
    C: TryFrom<u8>,
    G: for<'a> TryFrom<&'a [u8]>,
    K: for<'a> TryFrom<&'a [u8]>,
    D: for<'de> serde::Deserialize<'de>,
{
    /// Tries to decode the message from the binary representation and [`Tag`].
    ///
    /// The `tag` must be currently [`START_PROTOCOL_MESSAGE_TAG`](Self::START_PROTOCOL_MESSAGE_TAG)
    /// and version [`START_PROTOCOL_VERSION`](Self::START_PROTOCOL_VERSION).
    pub fn decode(tag: Tag, data: &[u8]) -> errors::Result<Self> {
        if tag != Self::START_PROTOCOL_MESSAGE_TAG {
            return Err(StartProtocolError::UnknownTag);
        }

        if data.len() < 5 {
            return Err(StartProtocolError::InvalidLength);
        }

        if data[0] != Self::START_PROTOCOL_VERSION {
            return Err(StartProtocolError::InvalidVersion);
        }

        let disc = data[1];
        let len = u16::from_be_bytes(
            data[2..4]
                .try_into()
                .map_err(|_| StartProtocolError::ParseError("len".into()))?,
        ) as usize;
        let data_offset = 2 + size_of::<u16>();

        if data.len() != data_offset + len {
            return Err(StartProtocolError::InvalidLength);
        }
        let body = &data[data_offset..data_offset + len];

        Ok(
            match StartProtocolDiscriminants::from_repr(disc).ok_or(StartProtocolError::UnknownMessage)? {
                StartProtocolDiscriminants::StartSession => {
                    if body.len() < size_of::<StartChallenge>() + 1 + size_of::<u64>() {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    StartProtocol::StartSession(StartInitiation {
                        challenge: StartChallenge::from_be_bytes(
                            body[..size_of::<StartChallenge>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("init.challenge".into()))?,
                        ),
                        capabilities: body[size_of::<StartChallenge>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("init.capabilities".into()))?,
                        additional_data: u64::from_be_bytes(
                            body[size_of::<StartChallenge>() + 1..size_of::<StartChallenge>() + 1 + size_of::<u64>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("init.additional_data".into()))?,
                        ),
                        target: serde_cbor_2::from_slice(&body[size_of::<StartChallenge>() + 1 + size_of::<u64>()..])?,
                    })
                }
                StartProtocolDiscriminants::SessionEstablished => {
                    if body.len() < size_of::<StartChallenge>() {
                        return Err(StartProtocolError::InvalidLength);
                    }
                    StartProtocol::SessionEstablished(StartEstablished {
                        orig_challenge: StartChallenge::from_be_bytes(
                            body[..size_of::<StartChallenge>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("est.challenge".into()))?,
                        ),
                        session_id: serde_cbor_2::from_slice(&body[size_of::<StartChallenge>()..])?,
                    })
                }
                StartProtocolDiscriminants::SessionError => {
                    if body.is_empty() {
                        return Err(StartProtocolError::InvalidLength);
                    }
                    let (identifier, reason_start) = match ErrorIdentifierDiscriminants::from_repr(body[0])
                        .ok_or(StartProtocolError::ParseError("err.identifier_tag".into()))?
                    {
                        ErrorIdentifierDiscriminants::Challenge => {
                            if body.len() < 1 + size_of::<StartChallenge>() + 1 {
                                return Err(StartProtocolError::InvalidLength);
                            }
                            let challenge = StartChallenge::from_be_bytes(
                                body[1..1 + size_of::<StartChallenge>()]
                                    .try_into()
                                    .map_err(|_| StartProtocolError::ParseError("err.challenge".into()))?,
                            );
                            (ErrorIdentifier::Challenge(challenge), 1 + size_of::<StartChallenge>())
                        }
                        ErrorIdentifierDiscriminants::SessionId => {
                            if body.len() < 2 {
                                return Err(StartProtocolError::InvalidLength);
                            }
                            // Reason byte is the last byte; CBOR session_id is everything
                            // between the tag byte and the reason.
                            let reason_start = body.len().saturating_sub(1);
                            let session_id: I = serde_cbor_2::from_slice(&body[1..reason_start])?;
                            (ErrorIdentifier::SessionId(session_id), reason_start)
                        }
                    };
                    StartProtocol::SessionError(StartErrorType {
                        identifier,
                        reason: StartErrorReason::from_repr(body[reason_start])
                            .ok_or(StartProtocolError::ParseError("err.reason".into()))?,
                    })
                }
                StartProtocolDiscriminants::KeepAlive => {
                    if body.len() < 1 + size_of::<u64>() {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    StartProtocol::KeepAlive(KeepAliveMessage {
                        flags: KeepAliveFlags::new(body[0])
                            .map_err(|_| StartProtocolError::ParseError("ka.flags".into()))?,
                        additional_data: u64::from_be_bytes(
                            body[1..1 + size_of::<u64>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("ka.additional_data".into()))?,
                        ),
                        session_id: serde_cbor_2::from_slice(&body[1 + size_of::<u64>()..])?,
                    })
                }
                StartProtocolDiscriminants::SsaCommit => {
                    if body.len()
                        <= size_of::<hopr_protocol_pix::SsaIndex>()
                            + size_of::<hopr_protocol_pix::CoefficientIndex>()
                            + 2 * size_of::<hopr_protocol_pix::PolynomialIndex>()
                            + Self::PIX_COEFF_COMMITMENT_REPR_SIZE
                    {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    let ssa: hopr_protocol_pix::SsaIndex = hopr_protocol_pix::RawSsaIndex::from_be_bytes(
                        body[..size_of::<hopr_protocol_pix::SsaIndex>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("ssa_index".into()))?,
                    )
                    .try_into()
                    .map_err(|_| StartProtocolError::ParseError("ssa_index is 0".into()))?;
                    let coefficient_index = hopr_protocol_pix::CoefficientIndex::from_be_bytes(
                        body[size_of::<hopr_protocol_pix::SsaIndex>()
                            ..size_of::<hopr_protocol_pix::SsaIndex>()
                                + size_of::<hopr_protocol_pix::CoefficientIndex>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("coefficient_index".into()))?,
                    );
                    let num_polys =
                        hopr_protocol_pix::PolynomialIndex::from_be_bytes(
                            body[size_of::<hopr_protocol_pix::SsaIndex>()
                                + size_of::<hopr_protocol_pix::CoefficientIndex>()
                                ..size_of::<hopr_protocol_pix::SsaIndex>()
                                    + size_of::<hopr_protocol_pix::CoefficientIndex>()
                                    + size_of::<hopr_protocol_pix::PolynomialIndex>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("polynomial_index".into()))?,
                        );
                    if num_polys == 0 || num_polys > MAX_POLYS_PER_SSA {
                        return Err(StartProtocolError::NumberOfCommitments);
                    }

                    let mut fixed_prefix_size = size_of::<hopr_protocol_pix::SsaIndex>()
                        + size_of::<hopr_protocol_pix::CoefficientIndex>()
                        + size_of::<hopr_protocol_pix::PolynomialIndex>();

                    // The proof sits right after the fixed prefix on constant-term messages only,
                    // with no presence flag — the coefficient index is what says whether it is
                    // there, so the encoder is required to keep the two in agreement.
                    let commitment_proof = if coefficient_index == 0 {
                        if body.len() <= fixed_prefix_size + Self::PIX_COMMITMENT_PROOF_SIZE {
                            return Err(StartProtocolError::InvalidLength);
                        }
                        let proof =
                            K::try_from(&body[fixed_prefix_size..fixed_prefix_size + Self::PIX_COMMITMENT_PROOF_SIZE])
                                .map_err(|_| StartProtocolError::ParseError("commitment_proof".into()))?;
                        fixed_prefix_size += Self::PIX_COMMITMENT_PROOF_SIZE;
                        Some(proof)
                    } else {
                        None
                    };

                    let mut coefficient_commitments = {
                        // Derive the maximum number of polynomial entries this packet
                        // can carry from the same per-entry constraints as the encoder
                        // (see `new_multiple`).  Enforce it before allocating, so
                        // unreasonably large `num_polys` values that pass the
                        // `MAX_POLYS_PER_SSA` guard are still caught by the wire limit.
                        let per_entry =
                            size_of::<hopr_protocol_pix::PolynomialIndex>() + Self::PIX_COEFF_COMMITMENT_REPR_SIZE;
                        // Availability: remaining body after the fixed prefix fields
                        // (ssa_index + coefficient_index + num_polys), reserving at
                        // least one byte for the trailing CBOR session_id.
                        let avail = body.len().saturating_sub(fixed_prefix_size);
                        let max_by_payload = avail.saturating_sub(1) / per_entry;
                        if num_polys as usize > max_by_payload {
                            return Err(StartProtocolError::NumberOfCommitments);
                        }
                        std::collections::HashMap::with_capacity(num_polys as usize)
                    };
                    let mut next_offset = fixed_prefix_size;
                    for _ in 0..num_polys {
                        // Still needs to be space left for Session ID at the end of commitments
                        if body.len()
                            <= next_offset
                                + size_of::<hopr_protocol_pix::PolynomialIndex>()
                                + Self::PIX_COEFF_COMMITMENT_REPR_SIZE
                        {
                            return Err(StartProtocolError::InvalidLength);
                        }

                        let index = hopr_protocol_pix::PolynomialIndex::from_be_bytes(
                            body[next_offset..next_offset + size_of::<hopr_protocol_pix::PolynomialIndex>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("polynomial_index".into()))?,
                        );
                        next_offset += size_of::<hopr_protocol_pix::PolynomialIndex>();

                        let commitment =
                            G::try_from(&body[next_offset..next_offset + Self::PIX_COEFF_COMMITMENT_REPR_SIZE])
                                .map_err(|_| StartProtocolError::ParseError("commitment".into()))?;
                        next_offset += Self::PIX_COEFF_COMMITMENT_REPR_SIZE;

                        if coefficient_commitments.insert(index, commitment).is_some() {
                            return Err(StartProtocolError::DuplicateCommitment);
                        }
                    }

                    StartProtocol::SsaCommit(SsaClientCommitmentMessage {
                        session_id: serde_cbor_2::from_slice(&body[next_offset..])?,
                        ssa_index: ssa,
                        coefficient_index,
                        commitment_proof,
                        coefficient_commitments,
                    })
                }
                StartProtocolDiscriminants::SsaRequest => {
                    if body.len() <= size_of::<u32>() + 1 {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    let params = u32::from_be_bytes(
                        body[..size_of::<u32>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("params".into()))?,
                    );

                    // deposit_data is CBOR — decode using a deserializer that tracks its
                    // byte offset so we can skip only its size and leave the rest of the body
                    // (num_commitments + entries + session_id) untouched.
                    let mut de = serde_cbor_2::Deserializer::from_slice(&body[size_of::<u32>()..]);
                    let deposit_data: D = serde::Deserialize::deserialize(&mut de)
                        .map_err(|e| StartProtocolError::ParseError(format!("deposit_data: {e}")))?;
                    let deposit_data_len = de.byte_offset();
                    let mut next_offset = size_of::<u32>() + deposit_data_len;

                    if body.len() <= next_offset + size_of::<u16>() {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    let num_commitments = u16::from_be_bytes(
                        body[next_offset..next_offset + size_of::<u16>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("num_commitments".into()))?,
                    );
                    if num_commitments == 0 || num_commitments > Self::MAX_SSAS_PER_REQUEST {
                        return Err(StartProtocolError::NumberOfCommitments);
                    }
                    next_offset += size_of::<u16>();

                    let mut commitments = std::collections::BTreeMap::new();
                    for _ in 0..num_commitments {
                        if body.len()
                            <= next_offset
                                + size_of::<hopr_protocol_pix::SsaIndex>()
                                + Self::PIX_COEFF_COMMITMENT_REPR_SIZE
                        {
                            return Err(StartProtocolError::InvalidLength);
                        }

                        let ssa_index: hopr_protocol_pix::SsaIndex = hopr_protocol_pix::RawSsaIndex::from_be_bytes(
                            body[next_offset..next_offset + size_of::<hopr_protocol_pix::SsaIndex>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("ssa_index".into()))?,
                        )
                        .try_into()
                        .map_err(|_| StartProtocolError::ParseError("ssa_index is 0".into()))?;
                        next_offset += size_of::<hopr_protocol_pix::SsaIndex>();

                        let commitment =
                            G::try_from(&body[next_offset..next_offset + Self::PIX_COEFF_COMMITMENT_REPR_SIZE])
                                .map_err(|_| StartProtocolError::ParseError("commitment".into()))?;
                        next_offset += Self::PIX_COEFF_COMMITMENT_REPR_SIZE;

                        if commitments.insert(ssa_index, commitment).is_some() {
                            return Err(StartProtocolError::DuplicateCommitment);
                        }
                    }

                    StartProtocol::SsaRequest(SsaServerCommitmentMessage {
                        session_id: serde_cbor_2::from_slice(&body[next_offset..])?,
                        params,
                        deposit_data,
                        commitments,
                    })
                }
            },
        )
    }
}

impl<I, T, C, G, K, D> TryFrom<StartProtocol<I, T, C, G, K, D>> for ApplicationData
where
    I: serde::Serialize + for<'de> serde::Deserialize<'de>,
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    C: Into<u8> + TryFrom<u8>,
    G: AsRef<[u8]> + for<'a> TryFrom<&'a [u8]>,
    K: AsRef<[u8]> + for<'a> TryFrom<&'a [u8]>,
    D: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Error = StartProtocolError;

    fn try_from(value: StartProtocol<I, T, C, G, K, D>) -> Result<Self, Self::Error> {
        let (application_tag, plain_text) = value.encode()?;
        Ok(ApplicationData::new(application_tag, plain_text.into_vec())?)
    }
}

impl<I, T, C, G, K, D> TryFrom<ApplicationData> for StartProtocol<I, T, C, G, K, D>
where
    I: serde::Serialize + for<'de> serde::Deserialize<'de>,
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    C: Into<u8> + TryFrom<u8>,
    G: AsRef<[u8]> + for<'a> TryFrom<&'a [u8]>,
    K: AsRef<[u8]> + for<'a> TryFrom<&'a [u8]>,
    D: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Error = StartProtocolError;

    fn try_from(value: ApplicationData) -> Result<Self, Self::Error> {
        Self::decode(value.application_tag, &value.plain_text)
    }
}

#[cfg(test)]
mod tests {
    use hopr_crypto_packet::{
        HoprPixSpec,
        prelude::{HoprPacket, HoprPixCommitmentProof, HoprPixGroupElement},
    };
    use hopr_protocol_app::prelude::Tag;
    use hopr_protocol_pix::{EntryShareGenerator, PolynomialIndex, SsaGeneratorConfig, SsaIndex, SsaShareGenerator};
    use hopr_types::{crypto::prelude::SimplePseudonym, crypto_random::Randomizable};

    use super::*;

    /// A minimal deposit data type for tests (CBOR-encodes as a single byte).
    type MinimalDeposit = ();

    #[test]
    fn start_protocol_start_session_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::StartSession(StartInitiation {
            challenge: 0,
            target: "127.0.0.1:1234".to_string(),
            capabilities: Default::default(),
            additional_data: 0x12345678,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), (), (), MinimalDeposit>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_message_start_session_message_should_allow_for_at_least_two_surbs() -> anyhow::Result<()> {
        let msg =
            StartProtocol::<i32, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::StartSession(StartInitiation {
                challenge: 0,
                target: "127.0.0.1:1234".to_string(),
                capabilities: 0xff,
                additional_data: 0xffffffff,
            });

        // Two SURBs are needed because if the server wants to establish PIX, it needs to send an additional
        // SsaRequest message next to the SessionEstablished message.
        let len = msg.encode()?.1.len();
        assert!(
            HoprPacket::max_surbs_with_message(len) >= 2,
            "StartSession message size ({len}) must allow for at least 2 SURBs in packet",
        );

        Ok(())
    }

    #[test]
    fn start_protocol_session_established_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::SessionEstablished(StartEstablished {
            orig_challenge: 0,
            session_id: 10_i32,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), (), (), MinimalDeposit>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_session_error_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::SessionError(StartErrorType {
            identifier: ErrorIdentifier::Challenge(10),
            reason: StartErrorReason::NoSlotsAvailable,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), (), (), MinimalDeposit>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);

        // Also test SessionId variant
        let msg_3 = StartProtocol::SessionError(StartErrorType {
            identifier: ErrorIdentifier::SessionId(42_i32),
            reason: StartErrorReason::UnacceptablePixParams,
        });

        let (tag, msg) = msg_3.clone().encode()?;
        let msg_4 = StartProtocol::<i32, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::decode(tag, &msg)?;
        assert_eq!(msg_3, msg_4);

        Ok(())
    }

    #[test]
    fn start_protocol_session_ssa_request_message_should_encode_and_decode() -> anyhow::Result<()> {
        let mut commitments = std::collections::BTreeMap::new();
        for i in 1..=10 {
            commitments.insert(i.try_into()?, [0u8; 33]);
        }

        let msg_1 = StartProtocol::SsaRequest(SsaServerCommitmentMessage {
            session_id: 0xfeedbeef,
            params: 0xfeedbeef,
            deposit_data: MinimalDeposit::default(),
            commitments,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), [u8; 33], [u8; 65], MinimalDeposit>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<u32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::decode(tag, &msg)?;
        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    /// The round-trip test above deliberately uses an out-of-range sentinel to prove the codec is
    /// total. This one goes through the constructor and the accessor, which is what production uses
    /// and what actually pins the packed layout across `encode`/`decode`.
    // `MinimalDeposit` is `()` — the instantiation that carries no deposit data — so passing
    // `MinimalDeposit::default()` as the generic `deposit_data` argument is literally passing a
    // unit value. That is the point of this instantiation, not an oversight.
    #[allow(clippy::unit_arg)]
    #[test]
    fn start_protocol_session_ssa_request_message_should_preserve_pix_params() -> anyhow::Result<()> {
        let params = hopr_protocol_pix::PixParams::try_new(8192, 64, 32, hopr_protocol_pix::PixSuite::BabyJubJub)?;
        let msg_1 = StartProtocol::<u32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::SsaRequest(
            SsaServerCommitmentMessage::new(
                0xfeedbeef_u32,
                params,
                [(1.try_into()?, [0u8; 33]), (2.try_into()?, [1u8; 33])],
                MinimalDeposit::default(),
            ),
        );

        let (tag, msg) = msg_1.clone().encode()?;
        let msg_2 = StartProtocol::<u32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::decode(tag, &msg)?;
        assert_eq!(msg_1, msg_2);

        let StartProtocol::SsaRequest(decoded) = msg_2 else {
            anyhow::bail!("expected an SsaRequest");
        };
        assert_eq!(params, decoded.dimensions()?);
        Ok(())
    }

    /// A peer can put anything in the `params` word, so reading it must fail rather than silently
    /// yield nonsense dimensions.
    #[test]
    fn start_protocol_ssa_request_dimensions_should_reject_out_of_range_params() {
        let msg: SsaServerCommitmentMessage<u32, [u8; 33], MinimalDeposit> = SsaServerCommitmentMessage {
            session_id: 0xfeedbeef,
            params: 0xfeedbeef,
            deposit_data: MinimalDeposit::default(),
            commitments: Default::default(),
        };
        assert!(matches!(msg.dimensions(), Err(StartProtocolError::ParseError(_))));
    }

    #[test]
    fn start_protocol_session_ssa_request_message_should_fail_on_too_many_commitments() -> anyhow::Result<()> {
        let mut commitments = std::collections::BTreeMap::new();
        // A single commitment is 4 + 33 = 37 bytes.
        // Payload size is 432 bytes.
        // Header + params + num_commitments + session_id will take some space.
        // Let's add many commitments to exceed the limit.
        for i in 1..=100 {
            commitments.insert(i.try_into()?, [0u8; 33]);
        }

        let msg =
            StartProtocol::<u32, (), u8, [u8; 33], [u8; 65], MinimalDeposit>::SsaRequest(SsaServerCommitmentMessage {
                session_id: 0xfeedbeef,
                params: 0xfeedbeef,
                deposit_data: MinimalDeposit::default(),
                commitments,
            });

        assert!(matches!(msg.encode(), Err(StartProtocolError::NumberOfCommitments)));
        Ok(())
    }

    /// Pins [`StartProtocol::ssa_commit_chunking`] against a fully determined instantiation.
    ///
    /// The bound's only other consumers ask *it* how many messages to expect, so nothing else can
    /// catch a change in the arithmetic. Here every input is fixed, so the outputs can be stated
    /// outright: a change to the layout has to come through this test.
    #[test]
    fn ssa_commit_chunking_should_match_the_encode_layout() -> anyhow::Result<()> {
        type Spec = StartProtocol<i32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>;

        // header(4) + ssa_index(4) + coeff_index(2) + num_polys(2) = 12, plus the CBOR session id:
        // 0xfeedeef exceeds u16, so CBOR spends a 1-byte prefix and 4 bytes of payload on it.
        let budget = ApplicationData::PAYLOAD_SIZE - 12 - 5;
        // `PolynomialIndex` is a `u16` — an `SsaIndex` prefix belongs to `SsaRequest`, not here.
        let per_entry = size_of::<PolynomialIndex>() + Spec::PIX_COEFF_COMMITMENT_REPR_SIZE;

        let chunking = Spec::ssa_commit_chunking(&0xfeedeef)?;

        assert_eq!(budget / per_entry, chunking.max_commitments_per_message);
        assert_eq!(
            (budget - Spec::PIX_COMMITMENT_PROOF_SIZE) / per_entry,
            chunking.max_constant_terms_per_message
        );
        // Stated as literals too, so that a change reaching *both* the encoder and the derivation
        // above still has to be acknowledged here.
        assert_eq!(92, chunking.max_commitments_per_message);
        assert_eq!(90, chunking.max_constant_terms_per_message);

        // The proof is carried by constant-term messages only, so phase 1 must never fit more
        // entries than phase 2 — the invariant `new_multiple`'s two loops rely on.
        assert!(chunking.max_constant_terms_per_message < chunking.max_commitments_per_message);

        Ok(())
    }

    #[test]
    fn start_protocol_session_ssa_commit_message_should_encode_and_decode() -> anyhow::Result<()> {
        assert_eq!(
            33,
            StartProtocol::<i32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::PIX_COEFF_COMMITMENT_REPR_SIZE
        );
        // MAX_SSAS_PER_REQUEST must never drop below 25, which is the number of SSA commitments
        // `start_protocol_messages_must_fit_within_hopr_packet` packs into an `SsaRequest`.
        let ssas_per_request =
            StartProtocol::<i32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::MAX_SSAS_PER_REQUEST;
        assert!(
            ssas_per_request >= 25,
            "MAX_SSAS_PER_REQUEST={ssas_per_request} is too small to encode 25 SSA commitments",
        );

        // An `SsaCommit` entry is `(PolynomialIndex, commitment)`, and `PolynomialIndex` is a `u16`
        // — the encoder writes `index.to_be_bytes()` and the decoder consumes
        // `size_of::<PolynomialIndex>()`. The four-byte prefix belongs to `SsaRequest`, whose
        // entries carry an `SsaIndex`; using it here under-fills the message by two bytes an entry,
        // so this would stop being the maximum-size encode it is written to be.
        let max_coeffs = (ApplicationData::PAYLOAD_SIZE
            - StartProtocol::<i32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::START_HEADER_SIZE)
            / (size_of::<PolynomialIndex>()
                + StartProtocol::<i32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::PIX_COEFF_COMMITMENT_REPR_SIZE);

        // A non-zero coefficient index, so this message carries no proof and the entry budget is
        // the full one.
        let msg_1 = StartProtocol::SsaCommit(SsaClientCommitmentMessage {
            session_id: 0xfeedeef,
            ssa_index: hopr_protocol_pix::SsaIndex::MAX,
            coefficient_index: hopr_protocol_pix::CoefficientIndex::MAX,
            commitment_proof: None::<[u8; 65]>,
            coefficient_commitments: (0..max_coeffs).map(|i| (i as PolynomialIndex, [0u8; 33])).collect(),
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), [u8; 33], [u8; 65], MinimalDeposit>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<u32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::decode(tag, &msg)?;
        assert_eq!(msg_1, msg_2);

        Ok(())
    }

    #[test]
    fn start_protocol_keep_alive_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::KeepAlive(KeepAliveMessage {
            session_id: 10_i32,
            flags: None.into(),
            additional_data: 0xffffffff,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), (), (), MinimalDeposit>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);

        let msg_1 = StartProtocol::KeepAlive(KeepAliveMessage {
            session_id: 10_i32,
            flags: KeepAliveFlag::BalancerTarget.into(),
            additional_data: 0xffffffff,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), (), (), MinimalDeposit>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_messages_must_fit_within_hopr_packet() -> anyhow::Result<()> {
        let msg =
            StartProtocol::<i32, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::StartSession(StartInitiation {
                challenge: StartChallenge::MAX,
                target: "example-of-a-very-very-long-second-level-name.on-a-very-very-long-domain-name.info:65530"
                    .to_string(),
                capabilities: 0x80,
                additional_data: 0xffffffff,
            });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "StartSession must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<String, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::SessionEstablished(
            StartEstablished {
                orig_challenge: StartChallenge::MAX,
                session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
            },
        );

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionEstablished must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg =
            StartProtocol::<String, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::SessionError(StartErrorType {
                identifier: ErrorIdentifier::Challenge(StartChallenge::MAX),
                reason: StartErrorReason::NoSlotsAvailable,
            });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionError(Challenge) must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg =
            StartProtocol::<String, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::SessionError(StartErrorType {
                identifier: ErrorIdentifier::SessionId(
                    "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
                ),
                reason: StartErrorReason::UnacceptablePixParams,
            });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionError(SessionId) must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        // The deposit_data field (64 bytes) slightly reduces the per-request capacity, but
        // 23 commitments must still fit alongside a realistic long session-id.
        let mut commitments = std::collections::BTreeMap::new();
        for i in 1..24 {
            commitments.insert(i.try_into()?, [0u8; 33]);
        }

        let msg = StartProtocol::<String, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::SsaRequest(
            SsaServerCommitmentMessage {
                session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
                params: 0xfeedbeef,
                deposit_data: MinimalDeposit::default(),
                commitments,
            },
        );
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SsaRequest must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<String, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::SsaCommit(
            SsaClientCommitmentMessage {
                session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
                ssa_index: SsaIndex::MAX,
                coefficient_index: hopr_protocol_pix::CoefficientIndex::MAX,
                commitment_proof: None,
                coefficient_commitments: (0..24).map(|i| (i as PolynomialIndex, [0u8; 33])).collect(),
            },
        );
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SsaRequest must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        // The same message as a constant-term one: it additionally carries the proof, so it must
        // still fit with fewer entries.
        let msg = StartProtocol::<String, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::SsaCommit(
            SsaClientCommitmentMessage {
                session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
                ssa_index: SsaIndex::MAX,
                coefficient_index: 0,
                commitment_proof: Some([0u8; 65]),
                coefficient_commitments: (0..22).map(|i| (i as PolynomialIndex, [0u8; 33])).collect(),
            },
        );
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SsaCommit carrying a proof must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg =
            StartProtocol::<String, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::KeepAlive(KeepAliveMessage {
                session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
                flags: None.into(),
                additional_data: 0,
            });
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "KeepAlive must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        Ok(())
    }

    #[test]
    fn start_protocol_message_keep_alive_message_should_allow_for_maximum_surbs() -> anyhow::Result<()> {
        // The slack left after `MAX_SURBS_IN_PACKET` SURBs is `PAYLOAD_SIZE % HoprSurb::SIZE`
        // (38 bytes at the current packet size), so how long a session id may be while the message
        // still maxes out its SURBs tracks the packet size rather than being a constant. The real
        // instantiation uses a `HoprPseudonym` and has far more room than this generic `String`.
        let msg =
            StartProtocol::<String, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::KeepAlive(KeepAliveMessage {
                session_id: "long-session-id-abcde".to_string(),
                flags: None.into(),
                additional_data: 0,
            });
        let len = msg.encode()?.1.len();
        assert_eq!(
            KeepAliveMessage::<String>::MIN_SURBS_PER_MESSAGE,
            HoprPacket::MAX_SURBS_IN_PACKET
        );
        assert!(
            HoprPacket::max_surbs_with_message(len) >= KeepAliveMessage::<String>::MIN_SURBS_PER_MESSAGE,
            "KeepAlive message size ({}) must allow for at least {} SURBs in packet",
            len,
            KeepAliveMessage::<String>::MIN_SURBS_PER_MESSAGE
        );

        Ok(())
    }

    /// Session ID used by the hand-framed malformed-message tests below. Its concrete type is
    /// `u32`, matching the `I` parameter those tests decode with.
    const MALFORMED_SESSION_ID: u32 = 0xfeedbeef;

    /// Type the hand-framed bodies are decoded as: 33-byte commitments and a 65-byte proof, the
    /// same shape the production instantiation uses.
    type Decoder = StartProtocol<u32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>;

    /// Wraps a raw body in the Start protocol envelope (version, discriminant, 16-bit body length)
    /// exactly as `encode` does. Malformed-input tests need bodies the encoder would refuse to
    /// produce, so they frame them by hand.
    fn frame(disc: StartProtocolDiscriminants, body: &[u8]) -> Vec<u8> {
        let mut out = vec![Decoder::START_PROTOCOL_VERSION, disc as u8];
        out.extend_from_slice(&(body.len() as u16).to_be_bytes());
        out.extend_from_slice(body);
        out
    }

    /// Builds an `SsaCommit` body: fixed prefix, optional proof, entries, trailing CBOR session id.
    /// `declared_polys` is written to the wire as-is, so it can disagree with `entries` — which is
    /// the whole point of the tests that use it.
    fn ssa_commit_body(
        coefficient_index: u16,
        declared_polys: u16,
        proof: Option<&[u8]>,
        entries: &[(u16, [u8; 33])],
    ) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&1u32.to_be_bytes());
        body.extend_from_slice(&coefficient_index.to_be_bytes());
        body.extend_from_slice(&declared_polys.to_be_bytes());
        if let Some(proof) = proof {
            body.extend_from_slice(proof);
        }
        for (index, commitment) in entries {
            body.extend_from_slice(&index.to_be_bytes());
            body.extend_from_slice(commitment);
        }
        body.extend(serde_cbor_2::to_vec(&MALFORMED_SESSION_ID).expect("session id must serialize"));
        body
    }

    /// The `SsaRequest` counterpart of [`ssa_commit_body`], whose entries are keyed by SSA index.
    fn ssa_request_body(declared_commitments: u16, entries: &[(u32, [u8; 33])]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&0u32.to_be_bytes());
        // CBOR-encoded `()` (`null`), which is the single byte 0xF6
        body.push(0xf6);
        body.extend_from_slice(&declared_commitments.to_be_bytes());
        for (ssa_index, commitment) in entries {
            body.extend_from_slice(&ssa_index.to_be_bytes());
            body.extend_from_slice(commitment);
        }
        body.extend(serde_cbor_2::to_vec(&MALFORMED_SESSION_ID).expect("session id must serialize"));
        body
    }

    fn decode_framed(disc: StartProtocolDiscriminants, body: &[u8]) -> errors::Result<Decoder> {
        Decoder::decode(Decoder::START_PROTOCOL_MESSAGE_TAG, &frame(disc, body))
    }

    /// A commitment or proof whose byte length is free to disagree with `size_of`, which is what
    /// the encoder's size guards compare against. A fixed-size array can never be the wrong length,
    /// so those guards are unreachable without a type like this.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct VarLenBytes(Vec<u8>);

    impl AsRef<[u8]> for VarLenBytes {
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }

    fn is_parse_error(error: &StartProtocolError, what: &str) -> bool {
        matches!(error, StartProtocolError::ParseError(context) if context == what)
    }

    /// The proof rides on constant-term messages only, and its presence is implied by the
    /// coefficient index rather than by a flag. An encoder that let the two disagree would
    /// desynchronise the decoder, so all three disagreements must be refused.
    #[test]
    fn start_protocol_encode_should_refuse_a_proof_that_disagrees_with_the_coefficient_index() {
        let missing = StartProtocol::<u32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::SsaCommit(
            SsaClientCommitmentMessage {
                session_id: MALFORMED_SESSION_ID,
                ssa_index: SsaIndex::MIN,
                coefficient_index: 0,
                commitment_proof: None,
                coefficient_commitments: [(0u16, [0u8; 33])].into_iter().collect(),
            },
        );
        assert!(
            matches!(missing.encode(), Err(ref e) if is_parse_error(e, "missing commitment_proof")),
            "a constant-term message without a proof must be refused"
        );

        let unexpected = StartProtocol::<u32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::SsaCommit(
            SsaClientCommitmentMessage {
                session_id: MALFORMED_SESSION_ID,
                ssa_index: SsaIndex::MIN,
                coefficient_index: 1,
                commitment_proof: Some([0u8; 65]),
                coefficient_commitments: [(0u16, [0u8; 33])].into_iter().collect(),
            },
        );
        assert!(
            matches!(unexpected.encode(), Err(ref e) if is_parse_error(e, "commitment_proof on a non-constant term")),
            "a proof on a non-constant term must be refused"
        );

        let wrong_size = StartProtocol::<u32, String, u8, VarLenBytes, VarLenBytes, MinimalDeposit>::SsaCommit(
            SsaClientCommitmentMessage {
                session_id: MALFORMED_SESSION_ID,
                ssa_index: SsaIndex::MIN,
                coefficient_index: 0,
                commitment_proof: Some(VarLenBytes(vec![0u8; 7])),
                coefficient_commitments: [(
                    0u16,
                    VarLenBytes(vec![
                        0u8;
                        StartProtocol::<
                            u32,
                            String,
                            u8,
                            VarLenBytes,
                            VarLenBytes,
                            MinimalDeposit,
                        >::PIX_COEFF_COMMITMENT_REPR_SIZE
                    ]),
                )]
                .into_iter()
                .collect(),
            },
        );
        assert!(
            matches!(wrong_size.encode(), Err(ref e) if is_parse_error(e, "commitment_proof_size")),
            "a proof of the wrong length must be refused"
        );
    }

    /// Entries are read back at a fixed stride, so a commitment of the wrong length would shift
    /// every field after it. Both message kinds carry entries and both must reject it.
    #[test]
    fn start_protocol_encode_should_refuse_wrongly_sized_commitments() -> anyhow::Result<()> {
        let commit = StartProtocol::<u32, String, u8, VarLenBytes, VarLenBytes, MinimalDeposit>::SsaCommit(
            SsaClientCommitmentMessage {
                session_id: MALFORMED_SESSION_ID,
                ssa_index: SsaIndex::MIN,
                coefficient_index: 1,
                commitment_proof: None,
                coefficient_commitments: [(0u16, VarLenBytes(vec![0u8; 7]))].into_iter().collect(),
            },
        );
        assert!(
            matches!(commit.encode(), Err(ref e) if is_parse_error(e, "commitment_repr_size")),
            "SsaCommit must refuse a commitment of the wrong length"
        );

        let request = StartProtocol::<u32, String, u8, VarLenBytes, VarLenBytes, MinimalDeposit>::SsaRequest(
            SsaServerCommitmentMessage {
                session_id: MALFORMED_SESSION_ID,
                params: 0,
                deposit_data: MinimalDeposit::default(),
                commitments: [(SsaIndex::MIN, VarLenBytes(vec![0u8; 7]))].into_iter().collect(),
            },
        );
        assert!(
            matches!(request.encode(), Err(ref e) if is_parse_error(e, "commitment_repr_size")),
            "SsaRequest must refuse a commitment of the wrong length"
        );

        Ok(())
    }

    /// A commitment message with nothing to commit to is a wasted packet, and the decoder rejects
    /// `num_polys == 0` in any case — so the encoder must not emit one.
    #[test]
    fn start_protocol_encode_should_refuse_an_ssa_commit_without_commitments() {
        let msg = StartProtocol::<u32, String, u8, [u8; 33], [u8; 65], MinimalDeposit>::SsaCommit(
            SsaClientCommitmentMessage {
                session_id: MALFORMED_SESSION_ID,
                ssa_index: SsaIndex::MIN,
                coefficient_index: 1,
                commitment_proof: None,
                coefficient_commitments: Default::default(),
            },
        );

        assert!(matches!(msg.encode(), Err(StartProtocolError::NumberOfCommitments)));
    }

    /// Everything below decodes bodies a peer could send but the encoder would never produce. The
    /// codec has to stay total: each one is answered with an error, never a panic or a silent
    /// misparse of the fields that follow.
    #[test]
    fn start_protocol_decode_should_reject_truncated_session_errors() {
        // A `Challenge` identifier without room for the trailing reason byte.
        let mut body = vec![ErrorIdentifierDiscriminants::Challenge as u8];
        body.extend_from_slice(&[0u8; size_of::<StartChallenge>()]);
        assert!(matches!(
            decode_framed(StartProtocolDiscriminants::SessionError, &body),
            Err(StartProtocolError::InvalidLength)
        ));

        // A `SessionId` identifier with nothing at all after the tag byte.
        assert!(matches!(
            decode_framed(
                StartProtocolDiscriminants::SessionError,
                &[ErrorIdentifierDiscriminants::SessionId as u8]
            ),
            Err(StartProtocolError::InvalidLength)
        ));
    }

    #[test]
    fn start_protocol_decode_should_reject_an_ssa_commit_shorter_than_its_fixed_prefix() {
        assert!(matches!(
            decode_framed(StartProtocolDiscriminants::SsaCommit, &[0u8; 10]),
            Err(StartProtocolError::InvalidLength)
        ));
    }

    /// `num_polys` is attacker-controlled and sizes an allocation, so it is bounded before it is
    /// used — both at zero and at `MAX_POLYS_PER_SSA`.
    #[test]
    fn start_protocol_decode_should_reject_an_out_of_range_polynomial_count() {
        for declared in [0, MAX_POLYS_PER_SSA + 1] {
            let body = ssa_commit_body(1, declared, None, &[(0, [0u8; 33])]);
            assert!(
                matches!(
                    decode_framed(StartProtocolDiscriminants::SsaCommit, &body),
                    Err(StartProtocolError::NumberOfCommitments)
                ),
                "num_polys={declared} must be rejected"
            );
        }
    }

    /// A count that survives the `MAX_POLYS_PER_SSA` bound can still be more than the packet can
    /// physically hold; the wire limit is what catches it before the allocation.
    #[test]
    fn start_protocol_decode_should_reject_more_polynomials_than_the_body_can_hold() {
        let body = ssa_commit_body(1, 100, None, &[(0, [0u8; 33]), (1, [1u8; 33])]);

        assert!(matches!(
            decode_framed(StartProtocolDiscriminants::SsaCommit, &body),
            Err(StartProtocolError::NumberOfCommitments)
        ));
    }

    /// A constant-term message declares its proof by its coefficient index alone, so a body with no
    /// room for one after the fixed prefix is malformed rather than proof-less.
    #[test]
    fn start_protocol_decode_should_reject_a_constant_term_without_room_for_its_proof() {
        // Fixed prefix plus exactly the proof: nothing left for entries or the session id.
        let body = ssa_commit_body(0, 1, Some(&[0u8; 65]), &[]);

        assert!(matches!(
            decode_framed(StartProtocolDiscriminants::SsaCommit, &body[..8 + 65]),
            Err(StartProtocolError::InvalidLength)
        ));
    }

    /// Repeating a polynomial index would silently overwrite the earlier commitment, so the decoder
    /// refuses the message instead of picking a winner.
    #[test]
    fn start_protocol_decode_should_reject_duplicate_polynomial_commitments() {
        let body = ssa_commit_body(1, 2, None, &[(7, [0u8; 33]), (7, [1u8; 33])]);

        assert!(matches!(
            decode_framed(StartProtocolDiscriminants::SsaCommit, &body),
            Err(StartProtocolError::DuplicateCommitment)
        ));
    }

    #[test]
    fn start_protocol_decode_should_reject_an_ssa_request_shorter_than_its_fixed_prefix() {
        assert!(matches!(
            decode_framed(StartProtocolDiscriminants::SsaRequest, &[0u8; 5]),
            Err(StartProtocolError::InvalidLength)
        ));
    }

    #[test]
    fn start_protocol_decode_should_reject_an_out_of_range_commitment_count() {
        for declared in [0, Decoder::MAX_SSAS_PER_REQUEST + 1] {
            let body = ssa_request_body(declared, &[(1, [0u8; 33])]);
            assert!(
                matches!(
                    decode_framed(StartProtocolDiscriminants::SsaRequest, &body),
                    Err(StartProtocolError::NumberOfCommitments)
                ),
                "num_commitments={declared} must be rejected"
            );
        }
    }

    /// `SsaRequest` has no payload-derived bound on its count, so a body that promises more entries
    /// than it carries must be caught while walking them.
    #[test]
    fn start_protocol_decode_should_reject_an_ssa_request_that_promises_more_than_it_carries() {
        let body = ssa_request_body(3, &[(1, [0u8; 33])]);

        assert!(matches!(
            decode_framed(StartProtocolDiscriminants::SsaRequest, &body),
            Err(StartProtocolError::InvalidLength)
        ));
    }

    #[test]
    fn start_protocol_decode_should_reject_duplicate_ssa_commitments() {
        let body = ssa_request_body(2, &[(1, [0u8; 33]), (1, [1u8; 33])]);

        assert!(matches!(
            decode_framed(StartProtocolDiscriminants::SsaRequest, &body),
            Err(StartProtocolError::DuplicateCommitment)
        ));
    }

    /// An SSA index of zero has no representation in `SsaIndex`, and it reaches the decoder as a
    /// plain `u32` — so both message kinds must turn it into an error rather than unwrap it.
    #[test]
    fn start_protocol_decode_should_reject_a_zero_ssa_index() {
        let mut body = ssa_commit_body(1, 1, None, &[(0, [0u8; 33])]);
        body[..4].copy_from_slice(&0u32.to_be_bytes());
        assert!(
            matches!(decode_framed(StartProtocolDiscriminants::SsaCommit, &body), Err(ref e) if is_parse_error(e, "ssa_index is 0")),
        );

        let body = ssa_request_body(1, &[(0, [0u8; 33])]);
        assert!(
            matches!(decode_framed(StartProtocolDiscriminants::SsaRequest, &body), Err(ref e) if is_parse_error(e, "ssa_index is 0")),
        );
    }

    #[test]
    fn start_protocol_new_multiple_messages_should_encode_and_decode() -> anyhow::Result<()> {
        // Generate a real SSA commitment using the same setup as the PIX
        // `test_generator_reconstructor`, but with 2048 polynomials per SSA and threshold 64.
        let generator = SsaShareGenerator::<HoprPixSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2048,
            threshold: 64,
            surplus_shares: 0,
        });

        let pseudonym = SimplePseudonym::random();
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        type DummySessionId = [u8; 20];

        let session_id: DummySessionId = Default::default();
        let messages: Vec<SsaClientCommitmentMessage<DummySessionId, HoprPixGroupElement, HoprPixCommitmentProof>> =
            SsaClientCommitmentMessage::new_multiple::<HoprPixSpec>(session_id, commitment)?;

        // PIX commits to constant terms only, so every message carries coefficient index 0 — the
        // wire format still admits others, nothing produces them. 2048 commitments do not fit in
        // one packet, so the pass spans many messages.
        assert!(messages.len() > 1);
        assert!(
            messages.iter().all(|m| m.coefficient_index == 0),
            "only constant-term commitments are ever emitted"
        );

        // Between them they must carry exactly one commitment per polynomial.
        let total: usize = messages.iter().map(|m| m.coefficient_commitments.len()).sum();
        assert_eq!(2048, total);
        let distinct: std::collections::BTreeSet<_> = messages
            .iter()
            .flat_map(|m| m.coefficient_commitments.keys().copied())
            .collect();
        assert_eq!(2048, distinct.len(), "each polynomial must be committed exactly once");

        // The proof of knowledge rides on every constant-term message: presence is implied by the
        // coefficient index rather than a flag, so the two must never disagree.
        assert!(
            messages.iter().all(|m| m.commitment_proof.is_some()),
            "the proof must accompany every constant-term message"
        );

        let mut max_encoded_size = 0;

        for message in messages {
            let msg_1 = StartProtocol::<
                DummySessionId,
                String,
                u8,
                HoprPixGroupElement,
                HoprPixCommitmentProof,
                MinimalDeposit,
            >::SsaCommit(message);

            let (tag, encoded) = msg_1.clone().encode()?;
            let expected: Tag =
                StartProtocol::<(), (), (), HoprPixGroupElement, HoprPixCommitmentProof, MinimalDeposit>::START_PROTOCOL_MESSAGE_TAG;
            assert_eq!(tag, expected);

            assert!(
                encoded.len() <= ApplicationData::PAYLOAD_SIZE,
                "encoded SsaCommit message ({} bytes) exceeds PAYLOAD_SIZE ({})",
                encoded.len(),
                ApplicationData::PAYLOAD_SIZE
            );
            max_encoded_size = max_encoded_size.max(encoded.len());

            let msg_2 = StartProtocol::<
                DummySessionId,
                String,
                u8,
                HoprPixGroupElement,
                HoprPixCommitmentProof,
                MinimalDeposit,
            >::decode(tag, &encoded)?;
            assert_eq!(msg_1, msg_2);
        }

        // The packing must be tight: the largest encoded message must leave less than one entry's
        // worth of headroom, or `new_multiple` is under-filling packets and emitting more of them
        // than the cycle needs.
        let per_entry = size_of::<hopr_protocol_pix::PolynomialIndex>()
            + StartProtocol::<DummySessionId, (), (), HoprPixGroupElement, HoprPixCommitmentProof, MinimalDeposit>::PIX_COEFF_COMMITMENT_REPR_SIZE;
        let headroom = ApplicationData::PAYLOAD_SIZE - max_encoded_size;
        assert!(
            headroom < per_entry,
            "largest encoded SsaCommit ({max_encoded_size} bytes) leaves {headroom} bytes of headroom, enough for \
             another {per_entry}-byte entry"
        );

        Ok(())
    }

    /// The whole commitment is `polys` constant terms, so the message count is decided purely by
    /// how many fit in a packet alongside the proof of knowledge. That bound is what
    /// `MIN_COMMITMENTS_PER_SSA_COMMIT_MSG` in `transport/session/src/manager.rs` mirrors to size
    /// the Start ingress channel, where an overflow silently kills a cycle — so it is worth
    /// pinning here rather than leaving it an implementation detail of `new_multiple`.
    #[test]
    fn start_protocol_ssa_commit_messages_should_cover_every_polynomial_exactly_once() -> anyhow::Result<()> {
        const POLYS: u16 = 2048;
        const THRESHOLD: u8 = 64;

        let generator = SsaShareGenerator::<HoprPixSpec>::new(SsaGeneratorConfig {
            polynomials_per_ssa: POLYS,
            threshold: THRESHOLD,
            surplus_shares: 0,
        });
        let pseudonym = SimplePseudonym::random();
        let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

        type DummySessionId = [u8; 20];
        let session_id: DummySessionId = Default::default();
        let messages: Vec<SsaClientCommitmentMessage<DummySessionId, HoprPixGroupElement, HoprPixCommitmentProof>> =
            SsaClientCommitmentMessage::new_multiple::<HoprPixSpec>(session_id, commitment)?;

        // Every polynomial committed exactly once, no duplicates and no omissions.
        let mut seen = std::collections::BTreeMap::<hopr_protocol_pix::PolynomialIndex, usize>::new();
        for message in &messages {
            for poly_index in message.coefficient_commitments.keys() {
                *seen.entry(*poly_index).or_default() += 1;
            }
        }
        assert_eq!(POLYS as usize, seen.len(), "every polynomial must be committed");
        assert!(
            seen.values().all(|&count| count == 1),
            "no polynomial may be committed twice — the Exit rejects the second as a duplicate"
        );

        // Every message but the last must be packed to the same width, and that width is the
        // per-message figure the channel sizing is derived from.
        let widths: Vec<usize> = messages.iter().map(|m| m.coefficient_commitments.len()).collect();
        let full = widths[0];
        assert!(
            widths[..widths.len() - 1].iter().all(|&w| w == full),
            "all but the trailing message must be full, got {widths:?}"
        );
        assert_eq!(
            messages.len(),
            (POLYS as usize).div_ceil(full),
            "the message count must be exactly what the packing implies"
        );

        Ok(())
    }

    #[test]
    fn start_protocol_keep_alive_truncated_lengths_should_not_panic() {
        let msg =
            StartProtocol::<String, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::KeepAlive(KeepAliveMessage {
                session_id: "test-session".to_string(),
                flags: None.into(),
                additional_data: 0,
            });
        let (tag, encoded) = msg.encode().expect("encode must succeed");
        let full_len = encoded.len();

        for trunc_len in 4..full_len {
            let result = StartProtocol::<String, String, u8, Box<[u8]>, Box<[u8]>, MinimalDeposit>::decode(
                tag,
                &encoded[..trunc_len],
            );
            assert!(
                result.is_err(),
                "truncated keep-alive at length {trunc_len}/{full_len} should return error, got {result:?}"
            );
        }
    }
}
