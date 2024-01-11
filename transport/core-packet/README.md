# core-packet

This crate contains the main packet processing functionality for the HOPR protocol.
It implements the following important protocol building blocks:

- SPHINX packet format (module `packet`)
- Proof of Relay (module `por`)

and namely it also implements (module `interactions`):

- AcknowledgementInteraction
- PacketInteraction

which are the most important high-level building blocks of the protocol.

Finally, it also implements a utility function which is used to validate tickets (module `validation`).

All the functionalities are heavily dependent on `core-ethereum-db` crate.

The currently used implementation is selected using the `CurrentSphinxSuite` type in the `packet` module.

The implementation can be easily extended for different elliptic curves (or even arithmetic multiplicative groups).
In particular, as soon as there's way to represent `Ed448` PeerIDs, it would be easy to create e.g. `X448Suite`.

## Interactions

- AcknowledgementInteraction
- PacketInteraction

These types internally work using 2 queues (TX, RX). As soon as acknowledgement or packet is received from the transport layer, it can be enqueued to the RX queue (`received_packet` or `received_acknowledgement`).
Similarly, a packet or acknowledgement can be enqueued for sending (via `send_packet` or `send_acknowledgement`).

Both TX and RX queues are bounded (currently capped to 2048 entries for both queues and interactions) and en-queuing can be done in either `wait`-mode or `fail-fast` mode if either queue is full. In fail-fast mode, the caller can decide whether to keep trying (e.g. when sending a packet) or whether to discard the data (e.g. when receiving packets).

The processing of RX queues can be done by awaiting `handle_outgoing_packets`, `handle_outgoing_acknowledgements` or `handle_incoming_packets` and `handle_incoming_acknowledgements`.
