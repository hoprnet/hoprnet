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

Ƭ **ChainWrapper**: _PromiseValue_<ReturnType<_typeof_ [_createChainWrapper_](ethereum.md#createchainwrapper)\>\>

Defined in: [packages/core-ethereum/src/ethereum.ts:281](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/ethereum.ts#L281)

---

### Receipt

Ƭ **Receipt**: _string_

Defined in: [packages/core-ethereum/src/ethereum.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/ethereum.ts#L27)

## Functions

### createChainWrapper

▸ **createChainWrapper**(`providerURI`: _string_, `privateKey`: Uint8Array): _Promise_<{ `announce`: (`multiaddr`: Multiaddr) => _Promise_<string\> ; `finalizeChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `fundChannel`: (`me`: _Address_, `counterparty`: _Address_, `myTotal`: _Balance_, `theirTotal`: _Balance_) => _Promise_<string\> ; `getBalance`: (`address`: _Address_) => _Promise_<Balance\> ; `getChannels`: () => [_HoprChannels_](../classes/contracts_hoprchannels.hoprchannels.md) ; `getGenesisBlock`: () => _number_ ; `getInfo`: () => _string_ ; `getLatestBlockNumber`: () => _Promise_<number\> ; `getNativeBalance`: (`address`: _any_) => _Promise_<NativeBalance\> ; `getPrivateKey`: () => _Uint8Array_ ; `getPublicKey`: () => _PublicKey_ ; `getWallet`: () => _Wallet_ ; `initiateChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `openChannel`: (`me`: _any_, `counterparty`: _any_, `amount`: _any_) => _Promise_<string\> ; `redeemTicket`: (`counterparty`: _any_, `ackTicket`: _any_, `ticket`: _any_) => _Promise_<string\> ; `setCommitment`: (`comm`: _Hash_) => _Promise_<string\> ; `subscribeBlock`: (`cb`: _any_) => _JsonRpcProvider_ \| _WebSocketProvider_ ; `subscribeChannelEvents`: (`cb`: _any_) => [_HoprChannels_](../classes/contracts_hoprchannels.hoprchannels.md) ; `subscribeError`: (`cb`: _any_) => _void_ ; `unsubscribe`: () => _void_ ; `waitUntilReady`: () => _Promise_<Network\> ; `withdraw`: (`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_) => _Promise_<string\> }\>

#### Parameters

| Name          | Type       |
| :------------ | :--------- |
| `providerURI` | _string_   |
| `privateKey`  | Uint8Array |

**Returns:** _Promise_<{ `announce`: (`multiaddr`: Multiaddr) => _Promise_<string\> ; `finalizeChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `fundChannel`: (`me`: _Address_, `counterparty`: _Address_, `myTotal`: _Balance_, `theirTotal`: _Balance_) => _Promise_<string\> ; `getBalance`: (`address`: _Address_) => _Promise_<Balance\> ; `getChannels`: () => [_HoprChannels_](../classes/contracts_hoprchannels.hoprchannels.md) ; `getGenesisBlock`: () => _number_ ; `getInfo`: () => _string_ ; `getLatestBlockNumber`: () => _Promise_<number\> ; `getNativeBalance`: (`address`: _any_) => _Promise_<NativeBalance\> ; `getPrivateKey`: () => _Uint8Array_ ; `getPublicKey`: () => _PublicKey_ ; `getWallet`: () => _Wallet_ ; `initiateChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `openChannel`: (`me`: _any_, `counterparty`: _any_, `amount`: _any_) => _Promise_<string\> ; `redeemTicket`: (`counterparty`: _any_, `ackTicket`: _any_, `ticket`: _any_) => _Promise_<string\> ; `setCommitment`: (`comm`: _Hash_) => _Promise_<string\> ; `subscribeBlock`: (`cb`: _any_) => _JsonRpcProvider_ \| _WebSocketProvider_ ; `subscribeChannelEvents`: (`cb`: _any_) => [_HoprChannels_](../classes/contracts_hoprchannels.hoprchannels.md) ; `subscribeError`: (`cb`: _any_) => _void_ ; `unsubscribe`: () => _void_ ; `waitUntilReady`: () => _Promise_<Network\> ; `withdraw`: (`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_) => _Promise_<string\> }\>

Defined in: [packages/core-ethereum/src/ethereum.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/ethereum.ts#L29)
