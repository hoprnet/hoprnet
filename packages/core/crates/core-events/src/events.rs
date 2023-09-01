use libp2p_identity::PeerId;
use strum::{Display, EnumVariantNames};
use core_types::channels::ChannelEntry;

#[derive(Clone, Debug, Display, EnumVariantNames)]
pub enum HoprEvent {
    /// Emitted when a new channel has been opened (transitioned to the Open state).
    #[strum(serialize = "channel-opened")]
    ChannelOpened(ChannelEntry),

    /// Emitted when an existing channel transitioned to the PendingToClose state.
    #[strum(serialize = "channel-closed")]
    ChannelClosed(ChannelEntry),

    /// Emitted when a new peer has been discovered on-chain
    #[strum(serialize = "peer-discovered")]
    PeerDiscovered(PeerId)
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use std::str::FromStr;
    use libp2p_identity::PeerId;
    use wasm_bindgen::prelude::wasm_bindgen;
    use core_types::channels::ChannelEntry;

    #[wasm_bindgen]
    pub struct HoprEvent {
        pub(crate) w: super::HoprEvent
    }

    #[wasm_bindgen]
    impl HoprEvent {
        pub fn channel_opened(entry: &ChannelEntry) -> Self {
            HoprEvent { w: super::HoprEvent::ChannelOpened(entry.clone()) }
        }

        pub fn channel_closed(entry: &ChannelEntry) -> Self {
            HoprEvent { w: super::HoprEvent::ChannelClosed(entry.clone()) }
        }

        pub fn peer_discovered(entry: &str) -> Self {
            HoprEvent { w: super::HoprEvent::PeerDiscovered(PeerId::from_str(entry).expect("invalid peer id")) }
        }
    }
}