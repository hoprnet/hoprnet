[@hoprnet/hopr-core-ethereum](README.md) / Exports

# @hoprnet/hopr-core-ethereum

## Table of contents

### Classes

- [Channel](classes/channel.md)
- [ChannelEntry](classes/channelentry.md)
- [Indexer](classes/indexer.md)
- [default](classes/default.md)

### Type aliases

- [RedeemTicketResponse](modules.md#redeemticketresponse)

## Type aliases

### RedeemTicketResponse

Ƭ **RedeemTicketResponse**: { `ackTicket`: `AcknowledgedTicket` ; `receipt`: `string` ; `status`: ``"SUCCESS"``  } \| { `message`: `string` ; `status`: ``"FAILURE"``  } \| { `error`: `Error` \| `string` ; `status`: ``"ERROR"``  }

#### Defined in

[core-ethereum/src/index.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L24)
