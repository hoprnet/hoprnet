pub struct HoprTicketFactory<S> {
    store: std::sync::Arc<parking_lot::RwLock<S>>,
}

