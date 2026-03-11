use hopr_api::chain::RedeemableTicket;

/// Backend for ticket storage queue.
///
/// The implementations must honor the natural ordering of tickets.
pub trait TicketQueue {
    type Error: std::error::Error + Send + 'static;
    /// Add a ticket to the queue.
    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error>;
    /// Remove and return the next ticket in-order from the queue.
    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error>;
    /// Return the next ticket in-order from the queue without removing it.
    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error>;
    /// Iterate over all tickets in the queue in **arbitrary** order.
    fn iter_unordered(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>>;
}

/// Simple in-memory ticket queue implementation using a binary heap.
///
/// This is suitable for testing where ticket persistence is not required.
#[derive(Clone, Debug, Default)]
pub struct MemoryTicketQueue(std::collections::BinaryHeap<RedeemableTicket>);

impl TicketQueue for MemoryTicketQueue {
    type Error = std::convert::Infallible;

    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error> {
        Ok(self.0.push(ticket))
    }

    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error> {
        Ok(self.0.pop())
    }

    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error> {
        Ok(self.0.peek().cloned())
    }

    fn iter_unordered(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>> {
        self.0.iter().cloned().map(Ok)
    }
}