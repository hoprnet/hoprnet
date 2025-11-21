mod peers;

mod tickets;

pub use peers::*;
pub use tickets::*;

/// Shorthand for the `chrono` based timestamp type used in the database.
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;

/// Complete set of HOPR node database APIs.
///
/// This trait is automatically implemented for types
/// that implement all the individual chain API traits to be implemented with the same error.
pub trait HoprNodeDbApi:
    HoprDbTicketOperations<Error = Self::NodeDbError>
    + HoprDbPeersOperations<Error = Self::NodeDbError>
    + HoprDbProtocolOperations<Error = Self::NodeDbError>
{
    type NodeDbError: std::error::Error + Send + Sync + 'static;
}

impl<T, E> HoprNodeDbApi for T
where
    T: HoprDbTicketOperations<Error = E> + HoprDbPeersOperations<Error = E> + HoprDbProtocolOperations<Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type NodeDbError = E;
}
