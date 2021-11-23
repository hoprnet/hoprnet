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

[packages/core-ethereum/src/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L6)

___

### INDEXER\_BLOCK\_RANGE

• **INDEXER\_BLOCK\_RANGE**: ``2000``

#### Defined in

[packages/core-ethereum/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/constants.ts#L8)

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

▸ **createChainWrapper**(`networkInfo`, `privateKey`, `checkDuplicate?`): `Promise`<{ `announce`: (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> ; `getBalance`: (`address`: `Address`) => `any` ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `any` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: `any`) => `StaticJsonRpcProvider` \| `WebSocketProvider` ; `subscribeChannelEvents`: (`cb`: `any`) => `any` ; `subscribeError`: (`cb`: `any`) => `void` ; `subscribeTokenEvents`: (`cb`: `any`) => `any` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\>  }\>

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

`Promise`<{ `announce`: (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> ; `getBalance`: (`address`: `Address`) => `any` ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `any` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: `any`) => `StaticJsonRpcProvider` \| `WebSocketProvider` ; `subscribeChannelEvents`: (`cb`: `any`) => `any` ; `subscribeError`: (`cb`: `any`) => `void` ; `subscribeTokenEvents`: (`cb`: `any`) => `any` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\>  }\>

#### Defined in

[packages/core-ethereum/src/ethereum.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/ethereum.ts#L34)

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
