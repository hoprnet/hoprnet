use core_types::channels::ChannelEntry;
use std::fmt::{Display, Formatter};
use utils_types::primitives::{Address, Balance};

/// A decision made by a strategy on each tick,
/// represents which channels should be closed and which should be opened.
/// Also indicates a number of maximum channels this strategy can open given the current network size.
/// Note that the number changes as the network size changes.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ChannelDecision {
    to_close: Vec<ChannelEntry>,
    to_open: Vec<(Address, Balance)>,
}

impl ChannelDecision {
    pub fn will_channel_be_closed(&self, counter_party: &Address) -> bool {
        self.to_close.iter().any(|c| &c.destination == counter_party)
    }

    pub fn will_address_be_opened(&self, address: &Address) -> bool {
        self.to_open.iter().any(|(addr, _)| addr == address)
    }

    pub fn add_to_close(&mut self, entry: ChannelEntry) {
        self.to_close.push(entry);
    }

    pub fn add_to_open(&mut self, address: Address, balance: Balance) {
        self.to_open.push((address, balance));
    }

    pub fn get_to_close(&self) -> &Vec<ChannelEntry> {
        &self.to_close
    }

    pub fn get_to_open(&self) -> &Vec<(Address, Balance)> {
        &self.to_open
    }
}

impl Display for ChannelDecision {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "channel decision: opening ({}), closing({})",
            self.to_open.len(),
            self.to_close.len()
        )
    }
}
