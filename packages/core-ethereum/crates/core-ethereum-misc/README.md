# core-ethereum-misc

This crate contains various on-chain related modules and types:

- `chain`: create payloads for various transactions used throughout entire HOPR on-chain layer
- `constants`: constants related to on-chain operations
- `errors`: on-chain related error messages
- `network_registry`: implements the Network Registry check that's used for peer connection gating
- `redeem`: contains all ticket redemption logic
- `transaction_queue`: implements a queue of outgoing transactions that is executed one-by-one in the background

## `redeem`

There are 4 functions that can be used to redeem tickets:

- `redeem_all_tickets`
- `redeem_tickets_in_channel`
- `redeem_tickets_by_counterparty`
- `redeem_ticket`

The method first checks if the tickets are redeemable (= they are not in `BeingRedeemed` or `BeginAggregated` in the DB),
and if they are, their state is changed to `BeingRedeemed` (while having acquired the exclusive DB write lock).
Subsequently, the ticket in such state is transmitted into the `TransactionQueue` so the redemption soon is executed on-chain.
The functions return immediately, but provide futures that can be awaited in case the callers wishes to await the on-chain
confirmation of each ticket redemption.

## `transaction_queue`

The `TransactionQueue` object acts as general outgoing on-chain transaction MPSC queue. The queue is picked up
one-by-one in an infinite loop that's executed in `core-hopr`. Any component that gets a `TransactionSender` type,
can send new transaction requests to the queue via its `send` method.
A new `TransactionSender` can be obtained by calling `new_sender` method on the `TransactionQueue` and can be subsequently cloned.
The possible transactions that can be sent into the queue are declared in the `Transaction` enum.
The `send` method of `TransactionSender` returns a `TransactionComplete` future that can be awaited if the caller
wishes to await the transaction being confirmed.
