use hopr_crypto_random::Randomizable;
use hopr_internal_types::{NodeId, protocol::HoprPseudonym};
use hopr_network_types::types::{DestinationRouting, RoutingOptions};
use hopr_primitive_types::bounded::BoundedVec;

pub struct TaggedDestinationRouting {
    /// The destination node.
    pub destination: Box<NodeId>,
    /// Pseudonym shown to the destination.
    pub pseudonym: HoprPseudonym,
    /// The path to the destination.
    pub forward_options: RoutingOptions,
    /// Optional return path.
    pub return_options: Option<RoutingOptions>,
}

impl TaggedDestinationRouting {
    pub fn neighbor(destination: Box<NodeId>) -> Self {
        Self {
            destination,
            pseudonym: HoprPseudonym::random(),
            forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
            return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
        }
    }

    pub fn loopback(me: Box<NodeId>, path: BoundedVec<NodeId, { RoutingOptions::MAX_INTERMEDIATE_HOPS }>) -> Self {
        Self {
            destination: me,
            pseudonym: HoprPseudonym::random(),
            forward_options: RoutingOptions::IntermediatePath(path),
            return_options: None,
        }
    }
}

impl From<TaggedDestinationRouting> for DestinationRouting {
    fn from(value: TaggedDestinationRouting) -> Self {
        DestinationRouting::Forward {
            destination: value.destination,
            pseudonym: Some(value.pseudonym),
            forward_options: value.forward_options,
            return_options: value.return_options,
        }
    }
}
