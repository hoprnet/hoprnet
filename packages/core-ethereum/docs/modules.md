[@hoprnet/hopr-core-ethereum](README.md) / Exports

# @hoprnet/hopr-core-ethereum

## Table of contents

### Classes

- [ChainWrapperSingleton](classes/ChainWrapperSingleton.md)
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
| `gasPrice?` | `number` |
| `maxConfirmations?` | `number` |
| `network` | `string` |
| `provider` | `string` |

#### Defined in

[packages/core-ethereum/src/index.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L45)

___

### ChainWrapper

Ƭ **ChainWrapper**: `Awaited`<`ReturnType`<typeof [`createChainWrapper`](modules.md#createchainwrapper)\>\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L27)

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

▸ **createChainWrapper**(`networkInfo`, `privateKey`, `checkDuplicate?`): `Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`address`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: `any` ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeChannelEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeTokenEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `networkInfo` | `Object` | `undefined` |
| `networkInfo.chainId` | `number` | `undefined` |
| `networkInfo.environment` | `string` | `undefined` |
| `networkInfo.gasPrice?` | `number` | `undefined` |
| `networkInfo.network` | `string` | `undefined` |
| `networkInfo.provider` | `string` | `undefined` |
| `privateKey` | `Uint8Array` | `undefined` |
| `checkDuplicate` | `Boolean` | `true` |

#### Returns

`Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`address`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: `any` ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeChannelEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeTokenEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L33)

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

▸ `Const` **useFixtures**(`ops?`): `Promise`<{ `COMMITTED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `OPENED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `chain`: { `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`address`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: `any` ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeChannelEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeTokenEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  } ; `db`: `HoprDB` ; `hoprChannels`: `HoprChannels` ; `hoprToken`: `HoprToken` ; `indexer`: `TestingIndexer` ; `newBlock`: () => `void` ; `newEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newTokenEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `provider`: `WebSocketProvider`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ops` | `Object` |
| `ops.id?` | `PublicKey` |
| `ops.latestBlockNumber?` | `number` |
| `ops.pastEvents?` | `TypedEvent`<`any`, `any`\>[] |

#### Returns

`Promise`<{ `COMMITTED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `OPENED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `chain`: { `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`address`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: `any` ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeChannelEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeTokenEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  } ; `db`: `HoprDB` ; `hoprChannels`: `HoprChannels` ; `hoprToken`: `HoprToken` ; `indexer`: `TestingIndexer` ; `newBlock`: () => `void` ; `newEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newTokenEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `provider`: `WebSocketProvider`  }\>

#### Defined in

[packages/core-ethereum/src/indexer/index.mock.ts:241](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.mock.ts#L241)
