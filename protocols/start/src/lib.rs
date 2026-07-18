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

/// Error message in the Start protocol.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StartErrorType {
    /// Challenge that relates to this error.
    pub challenge: StartChallenge,
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
pub enum StartProtocol<I, T, C, G> {
    /// Request to initiate a new session.
    StartSession(StartInitiation<T, C>),
    /// Confirmation that a new session has been established by the counterparty.
    SessionEstablished(StartEstablished<I>),
    /// Client's message to fill Client commitments to establish a Session Stealth Address (SSA).
    SsaCommit(SsaClientCommitmentMessage<I, G>),
    /// Server-side commitment to Session Stealth Address (SSA).
    SsaRequest(SsaServerCommitmentMessage<I, G>),
    /// Counterparty could not establish a new session due to an error.
    SessionError(StartErrorType),
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
/// The client always begins sending a message with `coefficient_index` equal to 0 to
/// deliver the commitment to the SSA first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaClientCommitmentMessage<I, G> {
    /// Session ID.
    pub session_id: I,
    /// Index of the Session Stealth Address (SSA) that is being committed.
    pub ssa_index: hopr_protocol_pix::SsaIndex,
    /// Index of the polynomial coefficient that is being committed.
    ///
    /// Zero value indicates the polynomial constant term commitment, which when summed over
    /// all polynomials for a given [`SsaIndex`](hopr_protocol_pix::SsaIndex)
    /// results in the Client's SSA commitment.
    pub coefficient_index: u16,
    /// Contains the serialized coefficient commitments of multiple polynomials,
    /// all belonging to the same `coefficient_index` in each polynomial.
    ///
    /// This might not be the complete set yet and might require multiple messages to deliver
    /// the complete commitment to the given coefficient of all polynomials for the given SSA.
    pub coefficient_commitments: std::collections::HashMap<hopr_protocol_pix::PolynomialIndex, G>,
}

impl<I: Clone, G: Clone> SsaClientCommitmentMessage<I, G> {
    /// Uses given the `session_id` and an [`SsaCommitment`] that will be split across multiple messages.
    ///
    /// The returned messages are ordered by coefficient index, making sure the constant terms
    /// of the polynomials are delivered first.
    pub fn new_multiple<S: hopr_protocol_pix::PixSpec>(
        session_id: I,
        commitment: SsaCommitment<S, S::Pseudonym>,
    ) -> Vec<Self>
    where
        G: From<hopr_protocol_pix::PixGroupRepr<S>>,
    {
        let ssa_index = commitment.ssa_id.ssa_index();

        // A single message can only carry a limited number of coefficient commitments so that the
        // resulting encoded message still fits within a HOPR packet payload. The commitments of a
        // single coefficient (across all polynomials) might therefore need to be split across
        // multiple messages. The bound below mirrors the conservative estimate used by the encoder
        // (it intentionally over-reserves space for the per-commitment polynomial index and the
        // message header), guaranteeing that every produced message encodes successfully.
        let max_commitments_per_message = ((ApplicationData::PAYLOAD_SIZE
            - StartProtocol::<I, (), (), G>::START_HEADER_SIZE)
            / (size_of::<hopr_protocol_pix::SsaIndex>() + size_of::<G>()))
        .max(1);

        // Group the transposed verifiers by their coefficient index. A `BTreeMap` is used to
        // guarantee that the resulting messages are ordered by coefficient index, making sure the
        // constant terms (coefficient index 0) of the polynomials are delivered first.
        let mut by_coefficient: std::collections::BTreeMap<u16, Vec<(hopr_protocol_pix::PolynomialIndex, G)>> =
            std::collections::BTreeMap::new();

        for (coefficient_index, coefficients) in commitment.verifiers {
            let entry = by_coefficient.entry(coefficient_index).or_default();
            for (poly_index, coefficient_commitment) in coefficients {
                entry.push((poly_index, G::from(coefficient_commitment)));
            }
        }

        let mut messages = Vec::new();
        for (coefficient_index, coefficients) in by_coefficient {
            // Split the commitments of this coefficient into chunks that each fit within a packet.
            for chunk in coefficients.chunks(max_commitments_per_message) {
                messages.push(Self {
                    session_id: session_id.clone(),
                    ssa_index,
                    coefficient_index,
                    coefficient_commitments: chunk.iter().cloned().collect(),
                });
            }
        }

        messages
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
pub struct SsaServerCommitmentMessage<I, G> {
    /// Session ID.
    pub session_id: I,
    /// Parameters of the PIX protocol the server requires.
    ///
    /// Upper half is the number of polynomials per SSA, lower half is the polynomial threshold (= degree + 1).
    pub params: u32,
    /// Server's serialized commitments to the SSAs, ordered by the SSA index.
    pub commitments: std::collections::BTreeMap<hopr_protocol_pix::SsaIndex, G>,
}

impl<I, G> SsaServerCommitmentMessage<I, G> {
    /// Convenience constructor for creating a new `SsaServerCommitmentMessage`.
    pub fn new(
        session_id: I,
        polys_per_ssa: u16,
        shares_per_poly: u16,
        commitments: impl IntoIterator<Item = (hopr_protocol_pix::SsaIndex, G)>,
    ) -> Self {
        Self {
            session_id,
            params: ((polys_per_ssa as u32) << 16) | shares_per_poly as u32,
            commitments: commitments.into_iter().collect(),
        }
    }

    /// Number of polynomials required to reconstruct an SSA.
    pub fn polys_per_ssa(&self) -> u16 {
        (self.params >> 16) as u16
    }

    /// Number of shares required to reconstruct a single polynomial.
    pub fn shares_per_poly(&self) -> u16 {
        self.params as u16
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

impl<I, T, C, G> StartProtocol<I, T, C, G> {
    /// Size of the PIX coefficient commitment representation in bytes.
    pub const PIX_COEFF_COMMITMENT_REPR_SIZE: usize = size_of::<G>();
    /// Size of the Start protocol message header in bytes.
    pub const START_HEADER_SIZE: usize =
        size_of_val(&Self::START_PROTOCOL_MESSAGE_TAG) + size_of::<u8>() + size_of::<u16>();
    /// Fixed [`Tag`] of every protocol message.
    pub const START_PROTOCOL_MESSAGE_TAG: Tag = Tag::Reserved(ReservedTag::SessionStart as u64);
    /// Current version of the Start protocol.
    pub const START_PROTOCOL_VERSION: u8 = 0x03;
}

impl<I, T, C, G> StartProtocol<I, T, C, G>
where
    I: serde::Serialize,
    T: serde::Serialize,
    C: Into<u8>,
    G: AsRef<[u8]>,
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
                data.extend_from_slice(&err.challenge.to_be_bytes());
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

                let session_id = serde_cbor_2::to_vec(&commit.session_id)?;
                let total_coeff_commit_len = (size_of::<hopr_protocol_pix::PolynomialIndex>() + size_of::<G>())
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
                    debug_assert_eq!(commitment_repr.len(), Self::PIX_COEFF_COMMITMENT_REPR_SIZE);

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

                let num_commitments = req.commitments.len() as u16;
                data.extend_from_slice(&num_commitments.to_be_bytes());

                let session_id = serde_cbor_2::to_vec(&req.session_id)?;

                let required_size = (size_of::<hopr_protocol_pix::SsaIndex>() + size_of::<G>()) * req.commitments.len();

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
                    debug_assert_eq!(commitment_repr.len(), Self::PIX_COEFF_COMMITMENT_REPR_SIZE);

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

impl<I, T, C, G> StartProtocol<I, T, C, G>
where
    I: for<'de> serde::Deserialize<'de>,
    T: for<'de> serde::Deserialize<'de>,
    C: TryFrom<u8>,
    G: for<'a> TryFrom<&'a [u8]>,
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

        Ok(
            match StartProtocolDiscriminants::from_repr(disc).ok_or(StartProtocolError::UnknownMessage)? {
                StartProtocolDiscriminants::StartSession => {
                    if data.len() <= data_offset + size_of::<StartChallenge>() + 1 + size_of::<u64>() {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    StartProtocol::StartSession(StartInitiation {
                        challenge: StartChallenge::from_be_bytes(
                            data[data_offset..data_offset + size_of::<StartChallenge>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("init.challenge".into()))?,
                        ),
                        capabilities: data[data_offset + size_of::<StartChallenge>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("init.capabilities".into()))?,
                        additional_data: u64::from_be_bytes(
                            data[data_offset + size_of::<StartChallenge>() + 1
                                ..data_offset + size_of::<StartChallenge>() + 1 + size_of::<u64>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("init.additional_data".into()))?,
                        ),
                        target: serde_cbor_2::from_slice(
                            &data[data_offset + size_of::<StartChallenge>() + 1 + size_of::<u64>()..],
                        )?,
                    })
                }
                StartProtocolDiscriminants::SessionEstablished => {
                    if data.len() <= data_offset + size_of::<StartChallenge>() {
                        return Err(StartProtocolError::InvalidLength);
                    }
                    StartProtocol::SessionEstablished(StartEstablished {
                        orig_challenge: StartChallenge::from_be_bytes(
                            data[data_offset..data_offset + size_of::<StartChallenge>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("est.challenge".into()))?,
                        ),
                        session_id: serde_cbor_2::from_slice(&data[data_offset + size_of::<StartChallenge>()..])?,
                    })
                }
                StartProtocolDiscriminants::SessionError => {
                    if data.len() < data_offset + size_of::<StartChallenge>() + 1 {
                        return Err(StartProtocolError::InvalidLength);
                    }
                    StartProtocol::SessionError(StartErrorType {
                        challenge: StartChallenge::from_be_bytes(
                            data[data_offset..data_offset + size_of::<StartChallenge>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("err.challenge".into()))?,
                        ),
                        reason: StartErrorReason::from_repr(data[data_offset + size_of::<StartChallenge>()])
                            .ok_or(StartProtocolError::ParseError("err.reason".into()))?,
                    })
                }
                StartProtocolDiscriminants::KeepAlive => {
                    if data.len() < data_offset + 1 + size_of::<u64>() {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    StartProtocol::KeepAlive(KeepAliveMessage {
                        flags: KeepAliveFlags::new(data[data_offset])
                            .map_err(|_| StartProtocolError::ParseError("ka.flags".into()))?,
                        additional_data: u64::from_be_bytes(
                            data[data_offset + 1..data_offset + 1 + size_of::<u64>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("ka.additional_data".into()))?,
                        ),
                        session_id: serde_cbor_2::from_slice(&data[data_offset + 1 + size_of::<u64>()..])?,
                    })
                }
                StartProtocolDiscriminants::SsaCommit => {
                    if data.len()
                        <= data_offset
                            + size_of::<hopr_protocol_pix::SsaIndex>()
                            + size_of::<hopr_protocol_pix::CoefficientIndex>()
                            + 2 * size_of::<hopr_protocol_pix::PolynomialIndex>()
                            + Self::PIX_COEFF_COMMITMENT_REPR_SIZE
                    {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    let ssa: hopr_protocol_pix::SsaIndex = hopr_protocol_pix::RawSsaIndex::from_be_bytes(
                        data[data_offset..data_offset + size_of::<hopr_protocol_pix::SsaIndex>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("ssa_index".into()))?,
                    )
                    .try_into()
                    .map_err(|_| StartProtocolError::ParseError("ssa_index is 0".into()))?;
                    let coefficient_index = hopr_protocol_pix::CoefficientIndex::from_be_bytes(
                        data[data_offset + size_of::<hopr_protocol_pix::SsaIndex>()
                            ..data_offset
                                + size_of::<hopr_protocol_pix::SsaIndex>()
                                + size_of::<hopr_protocol_pix::CoefficientIndex>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("coefficient_index".into()))?,
                    );
                    let num_polys = hopr_protocol_pix::PolynomialIndex::from_be_bytes(
                        data[data_offset
                            + size_of::<hopr_protocol_pix::SsaIndex>()
                            + size_of::<hopr_protocol_pix::CoefficientIndex>()
                            ..data_offset
                                + size_of::<hopr_protocol_pix::SsaIndex>()
                                + size_of::<hopr_protocol_pix::CoefficientIndex>()
                                + size_of::<hopr_protocol_pix::PolynomialIndex>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("polynomial_index".into()))?,
                    );
                    if num_polys == 0 || num_polys > MAX_POLYS_PER_SSA {
                        return Err(StartProtocolError::NumberOfCommitments);
                    }

                    let mut coefficient_commitments = std::collections::HashMap::with_capacity(num_polys as usize);
                    let mut next_offset = data_offset
                        + size_of::<hopr_protocol_pix::SsaIndex>()
                        + size_of::<hopr_protocol_pix::CoefficientIndex>()
                        + size_of::<hopr_protocol_pix::PolynomialIndex>();
                    while coefficient_commitments.len() < num_polys as usize {
                        // Still needs to be space left for Session ID at the end of commitments
                        if data.len()
                            <= next_offset
                                + size_of::<hopr_protocol_pix::PolynomialIndex>()
                                + Self::PIX_COEFF_COMMITMENT_REPR_SIZE
                        {
                            return Err(StartProtocolError::InvalidLength);
                        }

                        let index = hopr_protocol_pix::PolynomialIndex::from_be_bytes(
                            data[next_offset..next_offset + size_of::<hopr_protocol_pix::PolynomialIndex>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("polynomial_index".into()))?,
                        );
                        next_offset += size_of::<hopr_protocol_pix::PolynomialIndex>();

                        let commitment =
                            G::try_from(&data[next_offset..next_offset + Self::PIX_COEFF_COMMITMENT_REPR_SIZE])
                                .map_err(|_| StartProtocolError::ParseError("commitment".into()))?;
                        next_offset += Self::PIX_COEFF_COMMITMENT_REPR_SIZE;

                        coefficient_commitments.insert(index, commitment);
                    }

                    StartProtocol::SsaCommit(SsaClientCommitmentMessage {
                        session_id: serde_cbor_2::from_slice(&data[next_offset..])?,
                        ssa_index: ssa,
                        coefficient_index,
                        coefficient_commitments,
                    })
                }
                StartProtocolDiscriminants::SsaRequest => {
                    if data.len() <= data_offset + size_of::<u32>() + size_of::<u16>() {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    let params = u32::from_be_bytes(
                        data[data_offset..data_offset + size_of::<u32>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("params".into()))?,
                    );
                    let mut next_offset = data_offset + size_of::<u32>();

                    let num_commitments = u16::from_be_bytes(
                        data[next_offset..next_offset + size_of::<u16>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("num_commitments".into()))?,
                    );
                    next_offset += size_of::<u16>();

                    let mut commitments = std::collections::BTreeMap::new();
                    while commitments.len() < num_commitments as usize {
                        if data.len()
                            <= next_offset
                                + size_of::<hopr_protocol_pix::SsaIndex>()
                                + Self::PIX_COEFF_COMMITMENT_REPR_SIZE
                        {
                            return Err(StartProtocolError::InvalidLength);
                        }

                        let ssa_index: hopr_protocol_pix::SsaIndex = hopr_protocol_pix::RawSsaIndex::from_be_bytes(
                            data[next_offset..next_offset + size_of::<hopr_protocol_pix::SsaIndex>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("ssa_index".into()))?,
                        )
                        .try_into()
                        .map_err(|_| StartProtocolError::ParseError("ssa_index is 0".into()))?;
                        next_offset += size_of::<hopr_protocol_pix::SsaIndex>();

                        let commitment =
                            G::try_from(&data[next_offset..next_offset + Self::PIX_COEFF_COMMITMENT_REPR_SIZE])
                                .map_err(|_| StartProtocolError::ParseError("commitment".into()))?;
                        next_offset += Self::PIX_COEFF_COMMITMENT_REPR_SIZE;

                        commitments.insert(ssa_index, commitment);
                    }

                    StartProtocol::SsaRequest(SsaServerCommitmentMessage {
                        session_id: serde_cbor_2::from_slice(&data[next_offset..])?,
                        params,
                        commitments,
                    })
                }
            },
        )
    }
}

impl<I, T, C, G> TryFrom<StartProtocol<I, T, C, G>> for ApplicationData
where
    I: serde::Serialize + for<'de> serde::Deserialize<'de>,
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    C: Into<u8> + TryFrom<u8>,
    G: AsRef<[u8]> + for<'a> TryFrom<&'a [u8]>,
{
    type Error = StartProtocolError;

    fn try_from(value: StartProtocol<I, T, C, G>) -> Result<Self, Self::Error> {
        let (application_tag, plain_text) = value.encode()?;
        Ok(ApplicationData::new(application_tag, plain_text.into_vec())?)
    }
}

impl<I, T, C, G> TryFrom<ApplicationData> for StartProtocol<I, T, C, G>
where
    I: serde::Serialize + for<'de> serde::Deserialize<'de>,
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    C: Into<u8> + TryFrom<u8>,
    G: AsRef<[u8]> + for<'a> TryFrom<&'a [u8]>,
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
        prelude::{HoprPacket, HoprPixGroupElement},
    };
    use hopr_protocol_app::prelude::Tag;
    use hopr_protocol_pix::{EntryShareGenerator, PolynomialIndex, SsaGeneratorConfig, SsaIndex, SsaShareGenerator};
    use hopr_types::{crypto::prelude::SimplePseudonym, crypto_random::Randomizable};

    use super::*;

    #[test]
    fn start_protocol_start_session_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::StartSession(StartInitiation {
            challenge: 0,
            target: "127.0.0.1:1234".to_string(),
            capabilities: Default::default(),
            additional_data: 0x12345678,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_message_start_session_message_should_allow_for_at_least_two_surbs() -> anyhow::Result<()> {
        let msg = StartProtocol::<i32, String, u8, Box<[u8]>>::StartSession(StartInitiation {
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
        let expected: Tag = StartProtocol::<(), (), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_session_error_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::SessionError(StartErrorType {
            challenge: 10,
            reason: StartErrorReason::NoSlotsAvailable,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
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
            commitments,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), [u8; 33]>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<u32, String, u8, [u8; 33]>::decode(tag, &msg)?;
        assert_eq!(msg_1, msg_2);
        Ok(())
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

        let msg = StartProtocol::<u32, (), u8, [u8; 33]>::SsaRequest(SsaServerCommitmentMessage {
            session_id: 0xfeedbeef,
            params: 0xfeedbeef,
            commitments,
        });

        assert!(matches!(msg.encode(), Err(StartProtocolError::NumberOfCommitments)));
        Ok(())
    }

    #[test]
    fn start_protocol_session_ssa_commit_message_should_encode_and_decode() -> anyhow::Result<()> {
        assert_eq!(
            33,
            StartProtocol::<i32, String, u8, [u8; 33]>::PIX_COEFF_COMMITMENT_REPR_SIZE
        );

        let max_coeffs = (ApplicationData::PAYLOAD_SIZE
            - StartProtocol::<i32, String, u8, [u8; 33]>::START_HEADER_SIZE)
            / (size_of::<u32>() + StartProtocol::<i32, String, u8, [u8; 33]>::PIX_COEFF_COMMITMENT_REPR_SIZE);

        let msg_1 = StartProtocol::SsaCommit(SsaClientCommitmentMessage {
            session_id: 0xfeedeef,
            ssa_index: hopr_protocol_pix::SsaIndex::MAX,
            coefficient_index: hopr_protocol_pix::CoefficientIndex::MAX,
            coefficient_commitments: (0..max_coeffs).map(|i| (i as PolynomialIndex, [0u8; 33])).collect(),
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), [u8; 33]>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<u32, String, u8, [u8; 33]>::decode(tag, &msg)?;
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
        let expected: Tag = StartProtocol::<(), (), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);

        let msg_1 = StartProtocol::KeepAlive(KeepAliveMessage {
            session_id: 10_i32,
            flags: KeepAliveFlag::BalancerTarget.into(),
            additional_data: 0xffffffff,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8, Box<[u8]>>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_messages_must_fit_within_hopr_packet() -> anyhow::Result<()> {
        let msg = StartProtocol::<i32, String, u8, Box<[u8]>>::StartSession(StartInitiation {
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

        let msg = StartProtocol::<String, String, u8, Box<[u8]>>::SessionEstablished(StartEstablished {
            orig_challenge: StartChallenge::MAX,
            session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionEstablished must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<String, String, u8, Box<[u8]>>::SessionError(StartErrorType {
            challenge: StartChallenge::MAX,
            reason: StartErrorReason::NoSlotsAvailable,
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionError must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let mut commitments = std::collections::BTreeMap::new();
        for i in 1..26 {
            commitments.insert(i.try_into()?, [0u8; 33]);
        }

        let msg = StartProtocol::<String, String, u8, [u8; 33]>::SsaRequest(SsaServerCommitmentMessage {
            session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
            params: 0xfeedbeef,
            commitments,
        });
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SsaRequest must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<String, String, u8, [u8; 33]>::SsaCommit(SsaClientCommitmentMessage {
            session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
            ssa_index: SsaIndex::MAX,
            coefficient_index: hopr_protocol_pix::CoefficientIndex::MAX,
            coefficient_commitments: (0..24).map(|i| (i as PolynomialIndex, [0u8; 33])).collect(),
        });
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SsaRequest must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<String, String, u8, Box<[u8]>>::KeepAlive(KeepAliveMessage {
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
        let msg = StartProtocol::<String, String, u8, Box<[u8]>>::KeepAlive(KeepAliveMessage {
            session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
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
        let messages: Vec<SsaClientCommitmentMessage<DummySessionId, HoprPixGroupElement>> =
            SsaClientCommitmentMessage::new_multiple::<HoprPixSpec>(session_id, commitment);

        // Since 2048 polynomials per coefficient cannot fit into a single packet, the commitments
        // of each coefficient are split across multiple messages, so there are far more messages
        // than the threshold (= number of coefficient indices). The constant terms (coefficient
        // index 0) must still be delivered first.
        assert!(messages.len() > 64);
        assert_eq!(0, messages[0].coefficient_index);

        // Each coefficient index in 0..threshold must carry exactly 2048 commitments in total.
        let mut commitments_per_coefficient = std::collections::BTreeMap::<u16, usize>::new();
        for message in &messages {
            *commitments_per_coefficient
                .entry(message.coefficient_index)
                .or_default() += message.coefficient_commitments.len();
        }
        assert_eq!(
            (0u16..64).collect::<Vec<_>>(),
            commitments_per_coefficient.keys().copied().collect::<Vec<_>>()
        );
        assert!(commitments_per_coefficient.values().all(|&count| count == 2048));

        for message in messages {
            let msg_1 = StartProtocol::<DummySessionId, String, u8, HoprPixGroupElement>::SsaCommit(message);

            let (tag, encoded) = msg_1.clone().encode()?;
            let expected: Tag = StartProtocol::<(), (), (), HoprPixGroupElement>::START_PROTOCOL_MESSAGE_TAG;
            assert_eq!(tag, expected);

            let msg_2 = StartProtocol::<DummySessionId, String, u8, HoprPixGroupElement>::decode(tag, &encoded)?;
            assert_eq!(msg_1, msg_2);
        }

        Ok(())
    }

    #[test]
    fn start_protocol_keep_alive_truncated_lengths_should_not_panic() {
        let msg = StartProtocol::<String, String, u8, Box<[u8]>>::KeepAlive(KeepAliveMessage {
            session_id: "test-session".to_string(),
            flags: None.into(),
            additional_data: 0,
        });
        let (tag, encoded) = msg.encode().expect("encode must succeed");
        let full_len = encoded.len();

        for trunc_len in 4..full_len {
            let result = StartProtocol::<String, String, u8, Box<[u8]>>::decode(tag, &encoded[..trunc_len]);
            assert!(
                result.is_err(),
                "truncated keep-alive at length {trunc_len}/{full_len} should return error, got {result:?}"
            );
        }
    }
}
