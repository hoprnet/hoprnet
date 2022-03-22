[@hoprnet/hopr-core-ethereum](README.md) / Exports

# @hoprnet/hopr-core-ethereum

## Table of contents

### Classes

- [ChannelCommitmentInfo](classes/ChannelCommitmentInfo.md)
- [ChannelEntry](classes/ChannelEntry.md)
- [Indexer](classes/Indexer.md)
- [default](classes/default.md)

### Type aliases

- [ChainOptions](modules.md#chainoptions)
- [ChainWrapper](modules.md#chainwrapper)
- [RedeemTicketResponse](modules.md#redeemticketresponse)

### Variables

- [CONFIRMATIONS](modules.md#confirmations)
- [INDEXER\_BLOCK\_RANGE](modules.md#indexer_block_range)
- [sampleChainOptions](modules.md#samplechainoptions)

### Functions

- [bumpCommitment](modules.md#bumpcommitment)
- [createChainWrapper](modules.md#createchainwrapper)
- [createConnectorMock](modules.md#createconnectormock)
- [findCommitmentPreImage](modules.md#findcommitmentpreimage)
- [initializeCommitment](modules.md#initializecommitment)
- [useFixtures](modules.md#usefixtures)

## Type aliases

### ChainOptions

Ƭ **ChainOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `chainId` | `number` |
| `environment` | `string` |
| `gasPrice?` | `string` |
| `maxConfirmations?` | `number` |
| `network` | `string` |
| `provider` | `string` |

#### Defined in

[packages/core-ethereum/src/index.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L45)

___

### ChainWrapper

Ƭ **ChainWrapper**: `Awaited`<`ReturnType`<typeof [`createChainWrapper`](modules.md#createchainwrapper)\>\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L35)

___

### RedeemTicketResponse

Ƭ **RedeemTicketResponse**: { `ackTicket`: `AcknowledgedTicket` ; `receipt`: `string` ; `status`: ``"SUCCESS"``  } \| { `message`: `string` ; `status`: ``"FAILURE"``  } \| { `error`: `Error` \| `string` ; `status`: ``"ERROR"``  }

#### Defined in

[packages/core-ethereum/src/index.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L30)

## Variables

### CONFIRMATIONS

• **CONFIRMATIONS**: ``8``

#### Defined in

[packages/core-ethereum/src/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L6)

___

### INDEXER\_BLOCK\_RANGE

• **INDEXER\_BLOCK\_RANGE**: ``2000``

#### Defined in

[packages/core-ethereum/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L8)

___

### sampleChainOptions

• **sampleChainOptions**: [`ChainOptions`](modules.md#chainoptions)

#### Defined in

[packages/core-ethereum/src/ethereum.mock.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.mock.ts#L3)

## Functions

### bumpCommitment

▸ **bumpCommitment**(`db`, `channelId`, `newCommitment`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | `HoprDB` |
| `channelId` | `Hash` |
| `newCommitment` | `Hash` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/commitment.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L39)

___

### createChainWrapper

▸ **createChainWrapper**(`networkInfo`, `privateKey`, `checkDuplicate?`, `timeout?`): `Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`signedTransaction`: `string` \| `Promise`<`string`\>) => `Promise`<`TransactionResponse`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `networkInfo` | `Object` | `undefined` |
| `networkInfo.chainId` | `number` | `undefined` |
| `networkInfo.environment` | `string` | `undefined` |
| `networkInfo.gasPrice?` | `string` | `undefined` |
| `networkInfo.network` | `string` | `undefined` |
| `networkInfo.provider` | `string` | `undefined` |
| `privateKey` | `Uint8Array` | `undefined` |
| `checkDuplicate` | `Boolean` | `true` |
| `timeout` | `number` | `TX_CONFIRMATION_WAIT` |

#### Returns

`Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`signedTransaction`: `string` \| `Promise`<`string`\>) => `Promise`<`TransactionResponse`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L41)

___

### createConnectorMock

▸ **createConnectorMock**(`peer`): [`default`](classes/default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peer` | `PeerId` |

#### Returns

[`default`](classes/default.md)

#### Defined in

[packages/core-ethereum/src/index.mock.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.mock.ts#L8)

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

▸ **initializeCommitment**(`db`, `peerId`, `channelInfo`, `getChainCommitment`, `setChainCommitment`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | `HoprDB` |
| `peerId` | `PeerId` |
| `channelInfo` | [`ChannelCommitmentInfo`](classes/ChannelCommitmentInfo.md) |
| `getChainCommitment` | `GetCommitment` |
| `setChainCommitment` | `SetCommitment` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/commitment.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L102)

___

### useFixtures

▸ `Const` **useFixtures**(`ops?`): `Promise`<{ `COMMITTED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `OPENED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `chain`: { `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`signedTransaction`: `string` \| `Promise`<`string`\>) => `Promise`<`TransactionResponse`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  } ; `db`: `HoprDB` ; `hoprChannels`: `HoprChannels` ; `hoprToken`: `HoprToken` ; `indexer`: `TestingIndexer` ; `newBlock`: () => `void` ; `newEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newTokenEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `provider`: `WebSocketProvider`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ops` | `Object` |
| `ops.id?` | `PublicKey` |
| `ops.latestBlockNumber?` | `number` |
| `ops.pastEvents?` | `TypedEvent`<`any`, `any`\>[] |

#### Returns

`Promise`<{ `COMMITTED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `OPENED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `chain`: { `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`signedTransaction`: `string` \| `Promise`<`string`\>) => `Promise`<`TransactionResponse`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  } ; `db`: `HoprDB` ; `hoprChannels`: `HoprChannels` ; `hoprToken`: `HoprToken` ; `indexer`: `TestingIndexer` ; `newBlock`: () => `void` ; `newEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newTokenEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `provider`: `WebSocketProvider`  }\>

#### Defined in

[packages/core-ethereum/src/indexer/index.mock.ts:264](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.mock.ts#L264)
