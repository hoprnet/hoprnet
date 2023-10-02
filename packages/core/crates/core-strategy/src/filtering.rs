use core_types::channels::ChannelEntry;

/// Strategy meant just for filtering events based on different criteria for
/// other strategies that follow in a `MultiStrategy` chain.
pub struct FilteringStrategy<F>
where F: Fn(&ChannelEntry) -> bool {
    criteria: F
}