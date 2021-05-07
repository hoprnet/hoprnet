[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / index

# Module: index

## Table of contents

### References

- [Channel](index.md#channel)
- [Indexer](index.md#indexer)
- [RoutingChannel](index.md#routingchannel)

### Classes

- [default](../classes/index.default.md)

### Type aliases

- [RedeemTicketResponse](index.md#redeemticketresponse)

## References

### Channel

Re-exports: [Channel](../classes/channel.channel-1.md)

---

### Indexer

Renames and exports: [default](../classes/indexer.default.md)

---

### RoutingChannel

Re-exports: [RoutingChannel](indexer.md#routingchannel)

## Type aliases

### RedeemTicketResponse

Ƭ **RedeemTicketResponse**: { `ackTicket`: AcknowledgedTicket ; `receipt`: _string_ ; `status`: `"SUCCESS"` } \| { `message`: _string_ ; `status`: `"FAILURE"` } \| { `error`: Error \| _string_ ; `status`: `"ERROR"` }

Defined in: [packages/core-ethereum/src/index.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L17)
