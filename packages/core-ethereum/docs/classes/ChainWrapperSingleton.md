[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / ChainWrapperSingleton

# Class: ChainWrapperSingleton

## Table of contents

### Constructors

- [constructor](ChainWrapperSingleton.md#constructor)

### Properties

- [instance](ChainWrapperSingleton.md#instance)

### Methods

- [create](ChainWrapperSingleton.md#create)

## Constructors

### constructor

• `Private` **new ChainWrapperSingleton**()

#### Defined in

[packages/core-ethereum/src/chain.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/chain.ts#L8)

## Properties

### instance

▪ `Static` `Private` **instance**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `announce` | (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> |
| `finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> |
| `getAllQueuingTransactionRequests` | () => `TransactionRequest`[] |
| `getBalance` | (`address`: `Address`) => `Promise`<`Balance`\> |
| `getChannels` | () => `HoprChannels` |
| `getGenesisBlock` | () => `number` |
| `getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } |
| `getLatestBlockNumber` | () => `Promise`<`number`\> |
| `getNativeBalance` | (`address`: `Address`) => `Promise`<`NativeBalance`\> |
| `getNativeTokenTransactionInBlock` | (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> |
| `getPrivateKey` | () => `Uint8Array` |
| `getPublicKey` | () => `PublicKey` |
| `getWallet` | () => `Wallet` |
| `initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> |
| `redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> |
| `setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> |
| `subscribeBlock` | (`cb`: `any`) => `StaticJsonRpcProvider` \| `WebSocketProvider` |
| `subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `subscribeError` | (`cb`: `any`) => `void` |
| `subscribeTokenEvents` | (`cb`: `any`) => `HoprToken` |
| `unsubscribe` | () => `void` |
| `updateConfirmedTransaction` | (`hash`: `string`) => `void` |
| `waitUntilReady` | () => `Promise`<`Network`\> |
| `withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\> |

#### Defined in

[packages/core-ethereum/src/chain.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/chain.ts#L7)

## Methods

### create

▸ `Static` **create**(`networkInfo`, `privateKey`, `checkDuplicate?`): `Promise`<{ `announce`: (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`address`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: `any`) => `StaticJsonRpcProvider` \| `WebSocketProvider` ; `subscribeChannelEvents`: (`cb`: `any`) => `HoprChannels` ; `subscribeError`: (`cb`: `any`) => `void` ; `subscribeTokenEvents`: (`cb`: `any`) => `HoprToken` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\>  }\>

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

`Promise`<{ `announce`: (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`address`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: () => `Promise`<`number`\> ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: `any`) => `StaticJsonRpcProvider` \| `WebSocketProvider` ; `subscribeChannelEvents`: (`cb`: `any`) => `HoprChannels` ; `subscribeError`: (`cb`: `any`) => `void` ; `subscribeTokenEvents`: (`cb`: `any`) => `HoprToken` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\>  }\>

#### Defined in

[packages/core-ethereum/src/chain.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/chain.ts#L9)
