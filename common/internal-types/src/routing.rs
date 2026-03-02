use std::fmt::Formatter;

use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::HashFast;
use hopr_primitive_types::{
    bounded::{BoundedSize, BoundedVec},
    errors::GeneralError,
    prelude::BytesRepresentable,
};

use crate::{NodeId, path::ValidatedPath, prelude::HoprPseudonym};

/// Represents routing options in a mixnet with a maximum number of hops.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RoutingOptions {
    /// A fixed intermediate path consisting of at most [`RoutingOptions::MAX_INTERMEDIATE_HOPS`] hops.
    IntermediatePath(BoundedVec<NodeId, { RoutingOptions::MAX_INTERMEDIATE_HOPS }>),
    /// Random intermediate path with at least the given number of hops,
    /// but at most [`RoutingOptions::MAX_INTERMEDIATE_HOPS`].
    Hops(BoundedSize<{ RoutingOptions::MAX_INTERMEDIATE_HOPS }>),
}

impl RoutingOptions {
    /// The maximum number of hops this instance can represent.
    pub const MAX_INTERMEDIATE_HOPS: usize = 3;

    /// Inverts the intermediate path if this is an instance of [`RoutingOptions::IntermediatePath`].
    /// Otherwise, does nothing.
    pub fn invert(self) -> RoutingOptions {
        match self {
            RoutingOptions::IntermediatePath(v) => RoutingOptions::IntermediatePath(v.into_iter().rev().collect()),
            _ => self,
        }
    }

    /// Returns the number of hops this instance represents.
    /// This value is guaranteed to be between 0 and [`RoutingOptions::MAX_INTERMEDIATE_HOPS`].
    pub fn count_hops(&self) -> usize {
        match &self {
            RoutingOptions::IntermediatePath(v) => v.as_ref().len(),
            RoutingOptions::Hops(h) => (*h).into(),
        }
    }
}

/// Allows finding saved SURBs based on different criteria.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SurbMatcher {
    /// Try to find a SURB that has the exact given [`HoprSenderId`].
    Exact(HoprSenderId),
    /// Find any SURB with the given pseudonym.
    Pseudonym(HoprPseudonym),
}

impl SurbMatcher {
    /// Get the pseudonym part of the match.
    pub fn pseudonym(&self) -> HoprPseudonym {
        match self {
            SurbMatcher::Exact(id) => id.pseudonym(),
            SurbMatcher::Pseudonym(p) => *p,
        }
    }
}

impl From<HoprPseudonym> for SurbMatcher {
    fn from(value: HoprPseudonym) -> Self {
        Self::Pseudonym(value)
    }
}

impl From<&HoprPseudonym> for SurbMatcher {
    fn from(pseudonym: &HoprPseudonym) -> Self {
        (*pseudonym).into()
    }
}

/// Identifier for a path traversed using an allowed [`DestinationRouting`]
/// over the network.
pub type PathId = [u64; 5];

/// Routing information containing forward or return routing options.
///
/// Information in this object represents the minimum required basis
/// to generate forward paths and return paths.
///
/// See also [`RoutingOptions`].
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumIs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DestinationRouting {
    /// Forward routing using the destination node and path,
    /// with a possible return path.
    Forward {
        /// The destination node.
        destination: Box<NodeId>,
        /// Our pseudonym shown to the destination.
        ///
        /// If not given, it will be resolved as random.
        pseudonym: Option<HoprPseudonym>,
        /// The path to the destination.
        forward_options: RoutingOptions,
        /// Optional return path.
        return_options: Option<RoutingOptions>,
    },
    /// Return routing using a SURB with the given pseudonym.
    ///
    /// Will fail if no SURB for this pseudonym is found.
    Return(SurbMatcher),
}

impl DestinationRouting {
    /// Shortcut for routing that does not create any SURBs for a return path.
    pub fn forward_only<T: Into<NodeId>>(destination: T, forward_options: RoutingOptions) -> Self {
        Self::Forward {
            destination: Box::new(destination.into()),
            pseudonym: None,
            forward_options,
            return_options: None,
        }
    }
}

/// Contains the resolved routing information for the packet.
///
/// Instance of this object is typically constructed via some resolution of a
/// [`DestinationRouting`] instance.
///
/// It contains the actual forward and return paths for forward packets,
/// or an actual SURB `S` for return (reply) packets.
#[derive(Clone, Debug, strum::EnumIs)]
pub enum ResolvedTransportRouting<S> {
    /// Concrete routing information for a forward packet.
    Forward {
        /// Pseudonym of the sender.
        pseudonym: HoprPseudonym,
        /// Forward path.
        forward_path: ValidatedPath,
        /// Optional list of return paths.
        return_paths: Vec<ValidatedPath>,
    },
    /// Sender ID and the corresponding SURB.
    Return(HoprSenderId, S),
}

impl<S> ResolvedTransportRouting<S> {
    /// Shortcut for routing that does not create any SURBs for a return path.
    pub fn forward_only(forward_path: ValidatedPath) -> Self {
        Self::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path,
            return_paths: vec![],
        }
    }

    /// Returns the number of return paths (SURBs) on the [`ResolvedTransportRouting::Forward`]
    /// variant, or always 0 on the [`ResolvedTransportRouting::Return`] variant.
    pub fn count_return_paths(&self) -> usize {
        match self {
            ResolvedTransportRouting::Forward { return_paths, .. } => return_paths.len(),
            ResolvedTransportRouting::Return(..) => 0,
        }
    }
}

/// Size of the [`HoprSurbId`] in bytes.
pub const SURB_ID_SIZE: usize = 8;

/// An ID that uniquely identifies SURB for a certain pseudonym.
pub type HoprSurbId = [u8; SURB_ID_SIZE];

/// Identifier of a single packet sender.
///
/// This consists of two parts:
/// - [`HoprSenderId::pseudonym`] of the sender
/// - [`HoprSenderId::surb_id`] is an identifier a single SURB that routes the packet back to the sender.
///
/// The `surb_id` always identifies a single SURB. The instance can be turned into a pseudorandom
/// sequence using [`HoprSenderId::into_sequence`] to create identifiers for more SURBs
/// with the same pseudonym.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoprSenderId(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl std::fmt::Debug for HoprSenderId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HoprSenderId")
            .field(&hex::encode(&self.0[0..HoprPseudonym::SIZE]))
            .field(&hex::encode(&self.0[HoprPseudonym::SIZE..]))
            .finish()
    }
}

impl HoprSenderId {
    pub fn new(pseudonym: &HoprPseudonym) -> Self {
        let mut ret: [u8; Self::SIZE] = hopr_crypto_random::random_bytes();
        ret[..HoprPseudonym::SIZE].copy_from_slice(pseudonym.as_ref());
        Self(ret)
    }

    pub fn from_pseudonym_and_id(pseudonym: &HoprPseudonym, id: HoprSurbId) -> Self {
        let mut ret = [0u8; Self::SIZE];
        ret[..HoprPseudonym::SIZE].copy_from_slice(pseudonym.as_ref());
        ret[HoprPseudonym::SIZE..HoprPseudonym::SIZE + SURB_ID_SIZE].copy_from_slice(&id);
        Self(ret)
    }

    pub fn pseudonym(&self) -> HoprPseudonym {
        HoprPseudonym::try_from(&self.0[..HoprPseudonym::SIZE]).expect("must have valid pseudonym")
    }

    pub fn surb_id(&self) -> HoprSurbId {
        self.0[HoprPseudonym::SIZE..HoprPseudonym::SIZE + SURB_ID_SIZE]
            .try_into()
            .expect("must have valid nonce")
    }

    /// Creates a pseudorandom sequence of IDs.
    ///
    /// Each item has the same [`pseudonym`](HoprSenderId::pseudonym)
    /// but different [`surb_id`](HoprSenderId::surb_id).
    ///
    /// The `surb_id` of the `n`-th item (n > 1) is computed as `Blake3(n-1 || I_prev)`
    /// where `I_prev` is the whole `n-1`-th ID, the `n` is represented as big-endian and
    /// `||` denotes byte-array concatenation.
    /// The first item (n = 1) is always `self`.
    ///
    /// The entropy of the whole pseudorandom sequence is completely given by `self` (the first
    /// item in the sequence). It follows that the next element of the sequence can be computed
    /// by just knowing any preceding element; therefore, the sequence is fully predictable
    /// once an element is known.
    pub fn into_sequence(self) -> impl Iterator<Item = Self> {
        std::iter::successors(Some((1u32, self)), |&(i, prev)| {
            let hash = HashFast::create(&[&i.to_be_bytes(), prev.as_ref()]);
            Some((
                i + 1,
                Self::from_pseudonym_and_id(&prev.pseudonym(), hash.as_ref()[0..SURB_ID_SIZE].try_into().unwrap()),
            ))
        })
        .map(|(_, v)| v)
    }
}

impl AsRef<[u8]> for HoprSenderId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for HoprSenderId {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| GeneralError::ParseError("HoprSenderId.size".into()))
    }
}

impl BytesRepresentable for HoprSenderId {
    const SIZE: usize = HoprPseudonym::SIZE + SURB_ID_SIZE;
}

impl hopr_crypto_random::Randomizable for HoprSenderId {
    fn random() -> Self {
        Self::new(&HoprPseudonym::random())
    }
}
