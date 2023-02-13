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

- [sampleChainOptions](modules.md#samplechainoptions)

### Functions

- [bumpCommitment](modules.md#bumpcommitment)
- [createChainWrapper](modules.md#createchainwrapper)
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

[packages/core-ethereum/src/index.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L47)

___

### ChainWrapper

Ƭ **ChainWrapper**: `Awaited`<`ReturnType`<typeof [`createChainWrapper`](modules.md#createchainwrapper)\>\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L56)

___

### RedeemTicketResponse

Ƭ **RedeemTicketResponse**: { `ackTicket`: `AcknowledgedTicket` ; `receipt`: `string` ; `status`: ``"SUCCESS"``  } \| { `message`: `string` ; `status`: ``"FAILURE"``  } \| { `error`: `Error` \| `string` ; `status`: ``"ERROR"``  }

#### Defined in

[packages/core-ethereum/src/index.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L32)

## Variables

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

#### Defined in

[packages/core-ethereum/src/commitment.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L40)

___

### createChainWrapper

▸ **createChainWrapper**(`deploymentExtract`, `networkInfo`, `privateKey`, `checkDuplicate?`, `txTimeout?`): `Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getAllUnconfirmedHash`: () => `string`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = deploymentExtract.hoprChannelsAddress; `hoprNetworkRegistryAddress`: `string` = deploymentExtract.hoprNetworkRegistryAddress; `hoprTokenAddress`: `string` = deploymentExtract.hoprTokenAddress; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getNetworkRegistry`: () => `HoprNetworkRegistry` ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getTimestamp`: (`blockNumber`: `number`) => `Promise`<`number`\> ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `deploymentExtract` | `DeploymentExtract` | `undefined` |
| `networkInfo` | `Object` | `undefined` |
| `networkInfo.chainId` | `number` | `undefined` |
| `networkInfo.environment` | `string` | `undefined` |
| `networkInfo.maxFeePerGas` | `string` | `undefined` |
| `networkInfo.maxPriorityFeePerGas` | `string` | `undefined` |
| `networkInfo.network` | `string` | `undefined` |
| `networkInfo.provider` | `string` | `undefined` |
| `privateKey` | `Uint8Array` | `undefined` |
| `checkDuplicate` | `Boolean` | `true` |
| `txTimeout` | `number` | `constants.TX_CONFIRMATION_WAIT` |

#### Returns

`Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getAllUnconfirmedHash`: () => `string`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = deploymentExtract.hoprChannelsAddress; `hoprNetworkRegistryAddress`: `string` = deploymentExtract.hoprNetworkRegistryAddress; `hoprTokenAddress`: `string` = deploymentExtract.hoprTokenAddress; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getNetworkRegistry`: () => `HoprNetworkRegistry` ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getTimestamp`: (`blockNumber`: `number`) => `Promise`<`number`\> ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L71)

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

[packages/core-ethereum/src/commitment.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L25)

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

[packages/core-ethereum/src/commitment.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L103)

___

### useFixtures

▸ **useFixtures**(`ops?`): `Promise`<{ `COMMITTED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `OPENED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `chain`: { `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getAllUnconfirmedHash`: () => `string`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = deploymentExtract.hoprChannelsAddress; `hoprNetworkRegistryAddress`: `string` = deploymentExtract.hoprNetworkRegistryAddress; `hoprTokenAddress`: `string` = deploymentExtract.hoprTokenAddress; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getNetworkRegistry`: () => `HoprNetworkRegistry` ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getTimestamp`: (`blockNumber`: `number`) => `Promise`<`number`\> ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  } ; `db`: `HoprDB` ; `hoprChannels`: `HoprChannels` ; `hoprRegistry`: `HoprNetworkRegistry` ; `hoprToken`: `HoprToken` ; `indexer`: `TestingIndexer` ; `newBlock`: () => `void` ; `newEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newRegistryEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newTokenEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `provider`: `WebSocketProvider`  }\>

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

`Promise`<{ `COMMITTED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `OPENED_CHANNEL`: [`ChannelEntry`](classes/ChannelEntry.md) ; `chain`: { `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getAllUnconfirmedHash`: () => `string`[] ; `getBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = deploymentExtract.hoprChannelsAddress; `hoprNetworkRegistryAddress`: `string` = deploymentExtract.hoprNetworkRegistryAddress; `hoprTokenAddress`: `string` = deploymentExtract.hoprTokenAddress; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`accountAddress`: `Address`) => `Promise`<`Balance`\> ; `getNetworkRegistry`: () => `HoprNetworkRegistry` ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getTimestamp`: (`blockNumber`: `number`) => `Promise`<`number`\> ; `getToken`: () => `HoprToken` ; `getTransactionsInBlock`: (`blockNumber`: `number`) => `Promise`<`string`[]\> ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `sendTransaction`: (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> ; `setCommitment`: (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  } ; `db`: `HoprDB` ; `hoprChannels`: `HoprChannels` ; `hoprRegistry`: `HoprNetworkRegistry` ; `hoprToken`: `HoprToken` ; `indexer`: `TestingIndexer` ; `newBlock`: () => `void` ; `newEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newRegistryEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `newTokenEvent`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` ; `provider`: `WebSocketProvider`  }\>

#### Defined in

[packages/core-ethereum/src/indexer/index.mock.ts:284](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.mock.ts#L284)
