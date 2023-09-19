# core-protocol

Collection of objects and functionality allowing building of p2p or stream protocols for the higher business logic layers.

## Contents
Supported protocol configurations:
* `msg`
* `ack`
* `heartbeat`
* `ticket_aggregation`
* 
Supported protocol processors:
* `ticket_aggregation`

### `ticket_aggregation`
Ticket aggregation processing mechanism is responsible for ingesting the ticket aggregation related requests:
* Receive(PeerId, U),
* Reply(PeerId, std::result::Result<Ticket, String>, T),
* Send(PeerId, Vec<AcknowledgedTicket>, TicketAggregationFinalizer),

where `U` is the type of an aggregated ticket extractable (`ResponseChannel<Result<Ticket, String>>`) and `T` represents a network negotiated identifier (`RequestId`).

In broader context the protocol flow is as follows:
1. requesting ticket aggregation
   - the peer A desires to aggregate tickets, collects the tickets into a data collection and sends a request containing the collection to aggregate `Vec<AcknowledgedTicket>` to peer B using the `Send` mechanism

2. responding to ticket aggregation
   - peer B obtains the request from peer A, performs the ticket aggregation and returns a result of that operation in the form of `std::result::Result<Ticket, String>` using the `Reply` mechanism

3. accepting the aggregated ticket
   - peer A receives the aggregated ticket using the `Receive` mechanism

Furthermore, apart from the basic positive case scenario, standard mechanics of protocol communication apply:
- the requesting side can time out, if the responding side takes too long to provide an aggregated ticket, in which case the ticket is not considered aggregated, even if eventually an aggregated ticket is delivered
- the responder can fail to aggregate tickets in which case it replies with an error string describing the failure reason and it is the requester's responsibility to handle the negative case as well
  - in the absence of response, the requester will time out