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
| `announce` | (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `finalizeChannelClosure` | (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `getAllQueuingTransactionRequests` | () => `TransactionRequest`[] |
| `getBalance` | (`address`: `Address`) => `Promise`<`Balance`\> |
| `getChannels` | () => `HoprChannels` |
| `getGenesisBlock` | () => `number` |
| `getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } |
| `getLatestBlockNumber` | `any` |
| `getNativeBalance` | (`address`: `Address`) => `Promise`<`NativeBalance`\> |
| `getNativeTokenTransactionInBlock` | (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> |
| `getPrivateKey` | () => `Uint8Array` |
| `getPublicKey` | () => `PublicKey` |
| `getWallet` | () => `Wallet` |
| `initiateChannelClosure` | (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `subscribeBlock` | (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` |
| `subscribeChannelEvents` | (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` |
| `subscribeError` | (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` |
| `subscribeTokenEvents` | (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` |
| `unsubscribe` | () => `void` |
| `updateConfirmedTransaction` | (`hash`: `string`) => `void` |
| `waitUntilReady` | () => `Promise`<`Network`\> |
| `withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |

#### Defined in

[packages/core-ethereum/src/chain.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/chain.ts#L7)

## Methods

### create

▸ `Static` **create**(`networkInfo`, `privateKey`, `checkDuplicate?`): `Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`address`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: `any` ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeChannelEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeTokenEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

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

#### Returns

`Promise`<{ `announce`: (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `finalizeChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `fundChannel`: (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `getAllQueuingTransactionRequests`: () => `TransactionRequest`[] ; `getBalance`: (`address`: `Address`) => `Promise`<`Balance`\> ; `getChannels`: () => `HoprChannels` ; `getGenesisBlock`: () => `number` ; `getInfo`: () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } ; `getLatestBlockNumber`: `any` ; `getNativeBalance`: (`address`: `Address`) => `Promise`<`NativeBalance`\> ; `getNativeTokenTransactionInBlock`: (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> ; `getPrivateKey`: () => `Uint8Array` ; `getPublicKey`: () => `PublicKey` ; `getWallet`: () => `Wallet` ; `initiateChannelClosure`: (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `openChannel`: (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `redeemTicket`: (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `setCommitment`: (`counterparty`: `Address`, `comm`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> ; `subscribeBlock`: (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeChannelEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeError`: (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` ; `subscribeTokenEvents`: (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` ; `unsubscribe`: () => `void` ; `updateConfirmedTransaction`: (`hash`: `string`) => `void` ; `waitUntilReady`: () => `Promise`<`Network`\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\>  }\>

#### Defined in

[packages/core-ethereum/src/chain.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/chain.ts#L9)
