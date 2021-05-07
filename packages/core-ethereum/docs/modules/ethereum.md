[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / ethereum

# Module: ethereum

## Table of contents

### Type aliases

- [ChainWrapper](ethereum.md#chainwrapper)
- [Receipt](ethereum.md#receipt)

### Functions

- [createChainWrapper](ethereum.md#createchainwrapper)

## Type aliases

### ChainWrapper

Ƭ **ChainWrapper**: *PromiseValue*<ReturnType<*typeof* [*createChainWrapper*](ethereum.md#createchainwrapper)\>\>

Defined in: [packages/core-ethereum/src/ethereum.ts:281](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/ethereum.ts#L281)

___

### Receipt

Ƭ **Receipt**: *string*

Defined in: [packages/core-ethereum/src/ethereum.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/ethereum.ts#L27)

## Functions

### createChainWrapper

▸ **createChainWrapper**(`providerURI`: *string*, `privateKey`: Uint8Array): *Promise*<{ `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => [*HoprChannels*](../classes/contracts_hoprchannels.hoprchannels.md) ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *any*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `openChannel`: (`me`: *any*, `counterparty`: *any*, `amount`: *any*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *any*, `ackTicket`: *any*, `ticket`: *any*) => *Promise*<string\> ; `setCommitment`: (`comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => [*HoprChannels*](../classes/contracts_hoprchannels.hoprchannels.md) ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `providerURI` | *string* |
| `privateKey` | Uint8Array |

**Returns:** *Promise*<{ `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => [*HoprChannels*](../classes/contracts_hoprchannels.hoprchannels.md) ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *any*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `openChannel`: (`me`: *any*, `counterparty`: *any*, `amount`: *any*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *any*, `ackTicket`: *any*, `ticket`: *any*) => *Promise*<string\> ; `setCommitment`: (`comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => [*HoprChannels*](../classes/contracts_hoprchannels.hoprchannels.md) ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }\>

Defined in: [packages/core-ethereum/src/ethereum.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/ethereum.ts#L29)
