mod peers;

mod protocol;

mod tickets;

pub use peers::*;
pub use protocol::*;
pub use tickets::*;

/// Shorthand for the `chrono` based timestamp type used in the database.
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;

/// Complete set of HOPR node database APIs.
pub trait HoprNodeDbApi: HoprDbTicketOperations + HoprDbPeersOperations + HoprDbProtocolOperations {}

impl<T> HoprNodeDbApi for T where T: HoprDbTicketOperations + HoprDbPeersOperations + HoprDbProtocolOperations {}