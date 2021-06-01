[@hoprnet/hopr-core-ethereum](README.md) / Exports

# @hoprnet/hopr-core-ethereum

## Table of contents

### Classes

- [Channel](classes/channel.md)
- [Indexer](classes/indexer.md)
- [default](classes/default.md)

### Type aliases

- [RedeemTicketResponse](modules.md#redeemticketresponse)
- [RoutingChannel](modules.md#routingchannel)

## Type aliases

### RedeemTicketResponse

Ƭ **RedeemTicketResponse**: { `ackTicket`: AcknowledgedTicket ; `receipt`: *string* ; `status`: ``"SUCCESS"``  } \| { `message`: *string* ; `status`: ``"FAILURE"``  } \| { `error`: Error \| *string* ; `status`: ``"ERROR"``  }

Defined in: [core-ethereum/src/index.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L24)

___

### RoutingChannel

Ƭ **RoutingChannel**: [source: PeerId, destination: PeerId, stake: Balance]

Defined in: [core-ethereum/src/indexer/index.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L14)
