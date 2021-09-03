[@hoprnet/hopr-core-ethereum](README.md) / Exports

# @hoprnet/hopr-core-ethereum

## Table of contents

### Classes

- [Channel](classes/Channel.md)
- [ChannelEntry](classes/ChannelEntry.md)
- [Indexer](classes/Indexer.md)
- [default](classes/default.md)

### Type aliases

- [RedeemTicketResponse](modules.md#redeemticketresponse)

### Variables

- [CONFIRMATIONS](modules.md#confirmations)
- [INDEXER\_BLOCK\_RANGE](modules.md#indexer_block_range)

### Functions

- [createChainWrapper](modules.md#createchainwrapper)

## Type aliases

### RedeemTicketResponse

Ƭ **RedeemTicketResponse**: { `ackTicket`: `AcknowledgedTicket` ; `receipt`: `string` ; `status`: ``"SUCCESS"``  } \| { `message`: `string` ; `status`: ``"FAILURE"``  } \| { `error`: `Error` \| `string` ; `status`: ``"ERROR"``  }

#### Defined in

[packages/core-ethereum/src/index.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L25)

## Variables

### CONFIRMATIONS

• `Const` **CONFIRMATIONS**: ``8``

#### Defined in

[packages/core-ethereum/src/constants.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L7)

___

### INDEXER\_BLOCK\_RANGE

• `Const` **INDEXER\_BLOCK\_RANGE**: ``2000``

#### Defined in

[packages/core-ethereum/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L8)

## Functions

### createChainWrapper

▸ **createChainWrapper**(`providerURI`, `privateKey`): `Promise`<`Object`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `providerURI` | `string` |
| `privateKey` | `Uint8Array` |

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L40)
