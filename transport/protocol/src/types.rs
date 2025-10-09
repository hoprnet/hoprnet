use std::sync::Arc;
use ringbuffer::RingBuffer;
use hopr_crypto_packet::HoprSurb;
use hopr_crypto_packet::prelude::{HoprSenderId, HoprSurbId, PacketSignals};
use hopr_crypto_types::prelude::{HalfKey, HalfKeyChallenge, OffchainPublicKey, PacketTag};
use hopr_internal_types::prelude::{Acknowledgement, HoprPseudonym};
use crate::traits::SurbStore;

/// Packet that is being sent out by us
pub struct OutgoingPacket {
    pub next_hop: OffchainPublicKey,
    pub ack_challenge: HalfKeyChallenge,
    pub data: Box<[u8]>,
}

impl std::fmt::Debug for OutgoingPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutgoingPacket")
            .field("next_hop", &self.next_hop)
            .field("ack_challenge", &self.ack_challenge)
            .finish_non_exhaustive()
    }
}

/// Contains some miscellaneous information about a received packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AuxiliaryPacketInfo {
    /// Packet signals that the packet carried.
    ///
    /// Zero if no signal flags were specified.
    pub packet_signals: PacketSignals,
    /// Number of SURBs that the packet carried.
    pub num_surbs: usize,
}

pub struct IncomingFinalPacket {
    pub packet_tag: PacketTag,
    pub previous_hop: OffchainPublicKey,
    pub sender: HoprPseudonym,
    pub plain_text: Box<[u8]>,
    pub ack_key: HalfKey,
    pub info: AuxiliaryPacketInfo,
}

impl std::fmt::Debug for IncomingFinalPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingFinalPacket")
            .field("packet_tag", &self.packet_tag)
            .field("previous_hop", &self.previous_hop)
            .field("sender", &self.sender)
            .field("ack_key", &self.ack_key)
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

pub struct IncomingForwardedPacket {
    pub packet_tag: PacketTag,
    pub previous_hop: OffchainPublicKey,
    pub next_hop: OffchainPublicKey,
    pub data: Box<[u8]>,
    /// Acknowledgement payload to be sent to the previous hop
    pub ack_key: HalfKey,
}

impl std::fmt::Debug for IncomingForwardedPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingForwardedPacket")
            .field("packet_tag", &self.packet_tag)
            .field("previous_hop", &self.previous_hop)
            .field("next_hop", &self.next_hop)
            .field("ack_key", &self.ack_key)
            .finish_non_exhaustive()
    }
}

pub struct IncomingAcknowledgementPacket {
    pub packet_tag: PacketTag,
    pub previous_hop: OffchainPublicKey,
    pub ack: Acknowledgement,
}

impl std::fmt::Debug for IncomingAcknowledgementPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingAcknowledgementPacket")
            .field("packet_tag", &self.packet_tag)
            .field("previous_hop", &self.previous_hop)
            .field("ack", &self.ack)
            .finish()
    }
}

#[derive(Debug)]
pub enum IncomingPacket {
    /// Packet is intended for us
    Final(Box<IncomingFinalPacket>),
    /// Packet must be forwarded
    Forwarded(Box<IncomingForwardedPacket>),
    /// The packet contains an acknowledgement of a delivered packet.
    Acknowledgement(Box<IncomingAcknowledgementPacket>),
}

impl IncomingPacket {
    pub fn packet_tag(&self) -> &PacketTag {
        match self {
            IncomingPacket::Final(f) => &f.packet_tag,
            IncomingPacket::Forwarded(f) => &f.packet_tag,
            IncomingPacket::Acknowledgement(f) => &f.packet_tag, 
        }
    }
    
    pub fn previous_hop(&self) -> &OffchainPublicKey {
        match self {
            IncomingPacket::Final(f) => &f.previous_hop,
            IncomingPacket::Forwarded(f) => &f.previous_hop,
            IncomingPacket::Acknowledgement(f) => &f.previous_hop, 
        }
    }
}

/// Represents a single SURB along with its ID popped from the [`SurbRingBuffer`].
#[derive(Debug, Clone)]
pub struct PoppedSurb<S> {
    /// Complete SURB sender ID.
    pub id: HoprSurbId,
    /// The popped SURB.
    pub surb: S,
    /// Number of SURBs left in the RB after the pop.
    pub remaining: usize,
}

/// Ring buffer containing SURBs along with their IDs.
///
/// All these SURBs usually belong to the same pseudonym and are therefore identified
/// only by the [`HoprSurbId`].
#[derive(Clone, Debug)]
pub struct SurbRingBuffer<S>(Arc<parking_lot::Mutex<ringbuffer::AllocRingBuffer<(HoprSurbId, S)>>>);

impl<S> SurbRingBuffer<S> {
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(parking_lot::Mutex::new(ringbuffer::AllocRingBuffer::new(capacity))))
    }

    /// Push all SURBs with their IDs into the RB.
    ///
    /// Returns the total number of elements in the RB after the push.
    pub fn push<I: IntoIterator<Item = (HoprSurbId, S)>>(&self, surbs: I) -> usize {
        let mut rb = self.0.lock();
        rb.extend(surbs);
        rb.len()
    }

    /// Pop the latest SURB and its IDs from the RB.
    pub fn pop_one(&self) -> Option<PoppedSurb<S>> {
        let mut rb = self.0.lock();
        let (id, surb) = rb.dequeue()?;
        Some(PoppedSurb {
            id,
            surb,
            remaining: rb.len(),
        })
    }

    /// Check if the next SURB has the given ID and pop it from the RB.
    pub fn pop_one_if_has_id(&self, id: &HoprSurbId) -> Option<PoppedSurb<S>> {
        let mut rb = self.0.lock();

        if rb.peek().is_some_and(|(surb_id, _)| surb_id == id) {
            let (id, surb) = rb.dequeue()?;
            Some(PoppedSurb {
                id,
                surb,
                remaining: rb.len(),
            })
        } else {
            None
        }
    }
}

/// Contains a SURB found in the SURB ring buffer via  [`SurbStore::find_surb`].
#[derive(Debug)]
pub struct FoundSurb {
    /// Complete sender ID of the SURB.
    pub sender_id: HoprSenderId,
    /// The SURB itself.
    pub surb: HoprSurb,
    /// Number of SURBs remaining in the ring buffer with the same pseudonym.
    pub remaining: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surb_ring_buffer_must_drop_items_when_capacity_is_reached() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(3);
        rb.push([([1u8; 8], 0)]);
        rb.push([([2u8; 8], 0)]);
        rb.push([([3u8; 8], 0)]);
        rb.push([([4u8; 8], 0)]);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(2, popped.remaining);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([3u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([4u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        assert!(rb.pop_one().is_none());

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_must_be_fifo() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

        let len = rb.push([([1u8; 8], 0)]);
        assert_eq!(1, len);

        let len = rb.push([([2u8; 8], 0)]);
        assert_eq!(2, len);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([1u8; 8], popped.id);
        assert_eq!(1, popped.remaining);

        let popped = rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?;
        assert_eq!([2u8; 8], popped.id);
        assert_eq!(0, popped.remaining);

        let len = rb.push([([1u8; 8], 0), ([2u8; 8], 0)]);
        assert_eq!(2, len);

        assert_eq!([1u8; 8], rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?.id);
        assert_eq!([2u8; 8], rb.pop_one().ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }

    #[test]
    fn surb_ring_buffer_must_not_pop_if_id_does_not_match() -> anyhow::Result<()> {
        let rb = SurbRingBuffer::new(5);

        rb.push([([1u8; 8], 0)]);

        assert!(rb.pop_one_if_has_id(&[2u8; 8]).is_none());
        assert_eq!([1u8; 8], rb.pop_one_if_has_id(&[1u8; 8]).ok_or(anyhow::anyhow!("expected pop"))?.id);

        Ok(())
    }
}