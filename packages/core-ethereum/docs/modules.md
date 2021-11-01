[@hoprnet/hopr-core-ethereum](README.md) / Exports

# @hoprnet/hopr-core-ethereum

## Table of contents

### Classes

- [ChannelEntry](classes/ChannelEntry.md)
- [Indexer](classes/Indexer.md)
- [default](classes/default.md)

### Type aliases

- [RedeemTicketResponse](modules.md#redeemticketresponse)

### Variables

- [CONFIRMATIONS](modules.md#confirmations)
- [INDEXER\_BLOCK\_RANGE](modules.md#indexer_block_range)

### Functions

- [bumpCommitment](modules.md#bumpcommitment)
- [createChainWrapper](modules.md#createchainwrapper)
- [findCommitmentPreImage](modules.md#findcommitmentpreimage)
- [initializeCommitment](modules.md#initializecommitment)

## Type aliases

### RedeemTicketResponse

Ƭ **RedeemTicketResponse**: { `ackTicket`: `AcknowledgedTicket` ; `receipt`: `string` ; `status`: ``"SUCCESS"``  } \| { `message`: `string` ; `status`: ``"FAILURE"``  } \| { `error`: `Error` \| `string` ; `status`: ``"ERROR"``  }

#### Defined in

[packages/core-ethereum/src/index.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L28)

## Variables

### CONFIRMATIONS

• **CONFIRMATIONS**: ``8``

#### Defined in

[packages/core-ethereum/src/constants.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L7)

___

### INDEXER\_BLOCK\_RANGE

• **INDEXER\_BLOCK\_RANGE**: ``2000``

#### Defined in

[packages/core-ethereum/src/constants.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L9)

## Functions

### bumpCommitment

▸ **bumpCommitment**(`db`, `channelId`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | `HoprDB` |
| `channelId` | `Hash` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/commitment.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L39)

___

### createChainWrapper

▸ **createChainWrapper**(`providerURI`, `privateKey`, `checkDuplicate?`): `Promise`<`Object`\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `providerURI` | `string` | `undefined` |
| `privateKey` | `Uint8Array` | `undefined` |
| `checkDuplicate` | `Boolean` | `true` |

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L40)

___

### findCommitmentPreImage

▸ **findCommitmentPreImage**(`db`, `channelId`): `Promise`<`Hash`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | `HoprDB` |
| `channelId` | `Hash` |

#### Returns

`Promise`<`Hash`\>

#### Defined in

[packages/core-ethereum/src/commitment.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L24)

___

### initializeCommitment

▸ **initializeCommitment**(`db`, `channelId`, `getChainCommitment`, `setChainCommitment`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | `HoprDB` |
| `channelId` | `Hash` |
| `getChainCommitment` | `GetCommitment` |
| `setChainCommitment` | `SetCommitment` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/commitment.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L60)
