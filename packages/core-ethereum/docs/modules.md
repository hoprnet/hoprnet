[@hoprnet/hopr-core-ethereum](README.md) / Exports

# @hoprnet/hopr-core-ethereum

## Table of contents

### Classes

- [ChannelCommitmentInfo](classes/ChannelCommitmentInfo.md)
- [ChannelEntry](classes/ChannelEntry.md)
- [Indexer](classes/Indexer.md)
- [default](classes/default.md)

### Type Aliases

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

## Type Aliases

### ChainOptions

Ƭ **ChainOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `chainId` | `number` |
| `environment` | `string` |
| `maxConfirmations?` | `number` |
| `maxFeePerGas` | `string` |
| `maxPriorityFeePerGas` | `string` |
| `network` | `string` |
| `provider` | `string` |

#### Defined in

[packages/core-ethereum/src/index.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L44)

___

### ChainWrapper

Ƭ **ChainWrapper**: `Awaited`<`ReturnType`<typeof [`createChainWrapper`](modules.md#createchainwrapper)\>\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L52)

___

### RedeemTicketResponse

Ƭ **RedeemTicketResponse**: { `ackTicket`: `AcknowledgedTicket` ; `receipt`: `string` ; `status`: ``"SUCCESS"``  } \| { `message`: `string` ; `status`: ``"FAILURE"``  } \| { `error`: `Error` \| `string` ; `status`: ``"ERROR"``  }

#### Defined in

[packages/core-ethereum/src/index.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L29)

## Variables

### CONFIRMATIONS

• `Const` **CONFIRMATIONS**: ``8``

#### Defined in

[packages/core-ethereum/src/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L6)

___

### INDEXER\_BLOCK\_RANGE

• `Const` **INDEXER\_BLOCK\_RANGE**: ``2000``

#### Defined in

[packages/core-ethereum/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L8)

___

### sampleChainOptions

• `Const` **sampleChainOptions**: [`ChainOptions`](modules.md#chainoptions)

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

___

### createChainWrapper

▸ **createChainWrapper**(`networkInfo`, `privateKey`, `checkDuplicate?`, `txTimeout?`): `Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getAllUnconfirmedHash`: () => `string`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprNetworkRegistryAddress`: `string` = hoprNetworkRegistryDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getNetworkRegistry`: () => `HoprNetworkRegistry` ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getTimestamp`: (`blockNumber`: `number`) => `Promise`<`number`\> ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `networkInfo` | `Object` | `undefined` |
| `networkInfo.chainId` | `number` | `undefined` |
| `networkInfo.environment` | `string` | `undefined` |
| `networkInfo.maxFeePerGas` | `string` | `undefined` |
| `networkInfo.maxPriorityFeePerGas` | `string` | `undefined` |
| `networkInfo.network` | `string` | `undefined` |
| `networkInfo.provider` | `string` | `undefined` |
| `privateKey` | `Uint8Array` | `undefined` |
| `checkDuplicate` | `Boolean` | `true` |
| `txTimeout` | `number` | `TX_CONFIRMATION_WAIT` |

#### Returns

`Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getAllUnconfirmedHash`: () => `string`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprNetworkRegistryAddress`: `string` = hoprNetworkRegistryDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getNetworkRegistry`: () => `HoprNetworkRegistry` ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getTimestamp`: (`blockNumber`: `number`) => `Promise`<`number`\> ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

___

### createConnectorMock

▸ **createConnectorMock**(`peer`): [`default`](classes/default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peer` | `PeerId` |

#### Returns

[`default`](classes/default.md)

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

___

### useFixtures

▸ **useFixtures**(`ops?`): `Promise`<{ `COMMITTED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `OPENED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `chain`: { `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getAllUnconfirmedHash`: () => `string`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprNetworkRegistryAddress`: `string` = hoprNetworkRegistryDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getNetworkRegistry`: () => `HoprNetworkRegistry` ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getTimestamp`: (`blockNumber`: `number`) => `Promise`<`number`\> ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  } ; `db`: `HoprDB` ; `hoprChannels`: `HoprChannels` ; `hoprRegistry`: `HoprNetworkRegistry` ; `hoprToken`: `HoprToken` ; `indexer`: `TestingIndexer` ; `newBlock`: () => `void` ; `newEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newRegistryEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newTokenEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `provider`: `WebSocketProvider`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ops` | `Object` |
| `ops.id?` | `PublicKey` |
| `ops.latestBlockNumber?` | `number` |
| `ops.pastEvents?` | `TypedEvent`<`any`, `any`\>[] |
| `ops.pastHoprRegistryEvents?` | `TypedEvent`<`any`, `any`\>[] |
| `ops.pastHoprTokenEvents?` | `TypedEvent`<`any`, `any`\>[] |

#### Returns

`Promise`<{ `COMMITTED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `OPENED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `chain`: { `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getAllUnconfirmedHash`: () => `string`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprNetworkRegistryAddress`: `string` = hoprNetworkRegistryDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getNetworkRegistry`: () => `HoprNetworkRegistry` ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getTimestamp`: (`blockNumber`: `number`) => `Promise`<`number`\> ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  } ; `db`: `HoprDB` ; `hoprChannels`: `HoprChannels` ; `hoprRegistry`: `HoprNetworkRegistry` ; `hoprToken`: `HoprToken` ; `indexer`: `TestingIndexer` ; `newBlock`: () => `void` ; `newEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newRegistryEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newTokenEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `provider`: `WebSocketProvider`  }\>
