use std::fmt::{Display, Formatter};
use libp2p_identity::PeerId;
use core_types::channels::ChannelEntry;

#[derive(Clone, Debug)]
pub enum HoprEvent {
    ChannelOpened(ChannelEntry),
    ChannelClosed(ChannelEntry),
    PeerDiscovered(PeerId)
}

impl Display for HoprEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HoprEvent::ChannelOpened(_) => write!(f, "channel-opened"),
            HoprEvent::ChannelClosed(_) => write!(f, "channel-closed"),
            HoprEvent::PeerDiscovered(_) => write!(f, "peer-discovered")
        }
    }
}