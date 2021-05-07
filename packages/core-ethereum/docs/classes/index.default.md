[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [index](../modules/index.md) / default

# Class: default

[index](../modules/index.md).default

## Table of contents

### Constructors

- [constructor](index.default.md#constructor)

### Properties

- [CHAIN\_NAME](index.default.md#chain_name)
- [cachedGetBalance](index.default.md#cachedgetbalance)
- [cachedGetNativeBalance](index.default.md#cachedgetnativebalance)
- [indexer](index.default.md#indexer)
- [privateKey](index.default.md#privatekey)

### Methods

- [announce](index.default.md#announce)
- [getAccount](index.default.md#getaccount)
- [getAddress](index.default.md#getaddress)
- [getBalance](index.default.md#getbalance)
- [getChannel](index.default.md#getchannel)
- [getChannelsFromPeer](index.default.md#getchannelsfrompeer)
- [getChannelsOf](index.default.md#getchannelsof)
- [getNativeBalance](index.default.md#getnativebalance)
- [getPublicKey](index.default.md#getpublickey)
- [getPublicKeyOf](index.default.md#getpublickeyof)
- [getRandomChannel](index.default.md#getrandomchannel)
- [smartContractInfo](index.default.md#smartcontractinfo)
- [stop](index.default.md#stop)
- [uncachedGetBalance](index.default.md#uncachedgetbalance)
- [uncachedGetNativeBalance](index.default.md#uncachedgetnativebalance)
- [waitForPublicNodes](index.default.md#waitforpublicnodes)
- [withdraw](index.default.md#withdraw)
- [create](index.default.md#create)

## Constructors

### constructor

\+ **new default**(`chain`: { `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => [*HoprChannels*](contracts_hoprchannels.hoprchannels.md) ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *any*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `openChannel`: (`me`: *any*, `counterparty`: *any*, `amount`: *any*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *any*, `ackTicket`: *any*, `ticket`: *any*) => *Promise*<string\> ; `setCommitment`: (`comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => [*HoprChannels*](contracts_hoprchannels.hoprchannels.md) ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }, `db`: *HoprDB*, `indexer`: [*default*](indexer.default.md)): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `chain` | *object* |
| `chain.announce` | (`multiaddr`: Multiaddr) => *Promise*<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: *any*) => *Promise*<string\> |
| `chain.fundChannel` | (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> |
| `chain.getBalance` | (`address`: *Address*) => *Promise*<Balance\> |
| `chain.getChannels` | () => [*HoprChannels*](contracts_hoprchannels.hoprchannels.md) |
| `chain.getGenesisBlock` | () => *number* |
| `chain.getInfo` | () => *string* |
| `chain.getLatestBlockNumber` | () => *Promise*<number\> |
| `chain.getNativeBalance` | (`address`: *any*) => *Promise*<NativeBalance\> |
| `chain.getPrivateKey` | () => *Uint8Array* |
| `chain.getPublicKey` | () => *PublicKey* |
| `chain.getWallet` | () => *Wallet* |
| `chain.initiateChannelClosure` | (`counterparty`: *any*) => *Promise*<string\> |
| `chain.openChannel` | (`me`: *any*, `counterparty`: *any*, `amount`: *any*) => *Promise*<string\> |
| `chain.redeemTicket` | (`counterparty`: *any*, `ackTicket`: *any*, `ticket`: *any*) => *Promise*<string\> |
| `chain.setCommitment` | (`comm`: *Hash*) => *Promise*<string\> |
| `chain.subscribeBlock` | (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* |
| `chain.subscribeChannelEvents` | (`cb`: *any*) => [*HoprChannels*](contracts_hoprchannels.hoprchannels.md) |
| `chain.subscribeError` | (`cb`: *any*) => *void* |
| `chain.unsubscribe` | () => *void* |
| `chain.waitUntilReady` | () => *Promise*<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\> |
| `db` | *HoprDB* |
| `indexer` | [*default*](indexer.default.md) |

**Returns:** [*default*](index.default.md)

Defined in: [packages/core-ethereum/src/index.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L33)

## Properties

### CHAIN\_NAME

• `Readonly` **CHAIN\_NAME**: ``"HOPR on Ethereum"``= 'HOPR on Ethereum'

Defined in: [packages/core-ethereum/src/index.ts:39](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L39)

___

### cachedGetBalance

• `Private` **cachedGetBalance**: () => *Promise*<Balance\>

#### Type declaration

▸ (): *Promise*<Balance\>

**Returns:** *Promise*<Balance\>

Defined in: packages/utils/lib/cache.d.ts:1

Defined in: [packages/core-ethereum/src/index.ts:82](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L82)

___

### cachedGetNativeBalance

• `Private` **cachedGetNativeBalance**: () => *Promise*<NativeBalance\>

#### Type declaration

▸ (): *Promise*<NativeBalance\>

**Returns:** *Promise*<NativeBalance\>

Defined in: packages/utils/lib/cache.d.ts:1

Defined in: [packages/core-ethereum/src/index.ts:104](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L104)

___

### indexer

• **indexer**: [*default*](indexer.default.md)

___

### privateKey

• `Private` **privateKey**: *Uint8Array*

Defined in: [packages/core-ethereum/src/index.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L33)

## Methods

### announce

▸ **announce**(`multiaddr`: *Multiaddr*): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | *Multiaddr* |

**Returns:** *Promise*<string\>

Defined in: [packages/core-ethereum/src/index.ts:53](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L53)

___

### getAccount

▸ **getAccount**(`addr`: *Address*): *Promise*<AccountEntry\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<AccountEntry\>

Defined in: [packages/core-ethereum/src/index.ts:69](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L69)

___

### getAddress

▸ **getAddress**(): *Address*

**Returns:** *Address*

Defined in: [packages/core-ethereum/src/index.ts:91](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L91)

___

### getBalance

▸ **getBalance**(`useCache?`: *boolean*): *Promise*<Balance\>

Retrieves HOPR balance, optionally uses the cache.

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `useCache` | *boolean* | false |

**Returns:** *Promise*<Balance\>

HOPR balance

Defined in: [packages/core-ethereum/src/index.ts:87](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L87)

___

### getChannel

▸ **getChannel**(`src`: *PublicKey*, `counterparty`: *PublicKey*): [*Channel*](channel.channel-1.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `src` | *PublicKey* |
| `counterparty` | *PublicKey* |

**Returns:** [*Channel*](channel.channel-1.md)

Defined in: [packages/core-ethereum/src/index.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L49)

___

### getChannelsFromPeer

▸ **getChannelsFromPeer**(`p`: *PeerId*): *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `p` | *PeerId* |

**Returns:** *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)[]\>

Defined in: [packages/core-ethereum/src/index.ts:61](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L61)

___

### getChannelsOf

▸ **getChannelsOf**(`addr`: *Address*): *Promise*<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<ChannelEntry[]\>

Defined in: [packages/core-ethereum/src/index.ts:65](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L65)

___

### getNativeBalance

▸ **getNativeBalance**(`useCache?`: *boolean*): *Promise*<NativeBalance\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `useCache` | *boolean* | false |

**Returns:** *Promise*<NativeBalance\>

Defined in: [packages/core-ethereum/src/index.ts:108](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L108)

___

### getPublicKey

▸ **getPublicKey**(): *PublicKey*

**Returns:** *PublicKey*

Defined in: [packages/core-ethereum/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L95)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`: *Address*): *Promise*<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<PublicKey\>

Defined in: [packages/core-ethereum/src/index.ts:73](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L73)

___

### getRandomChannel

▸ **getRandomChannel**(): *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)\>

**Returns:** *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)\>

Defined in: [packages/core-ethereum/src/index.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L77)

___

### smartContractInfo

▸ **smartContractInfo**(): *string*

**Returns:** *string*

Defined in: [packages/core-ethereum/src/index.ts:112](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L112)

___

### stop

▸ **stop**(): *Promise*<void\>

Stops the connector.

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/index.ts:44](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L44)

___

### uncachedGetBalance

▸ `Private` **uncachedGetBalance**(): *Promise*<Balance\>

**Returns:** *Promise*<Balance\>

Defined in: [packages/core-ethereum/src/index.ts:81](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L81)

___

### uncachedGetNativeBalance

▸ `Private` **uncachedGetNativeBalance**(): *Promise*<NativeBalance\>

Retrieves ETH balance, optionally uses the cache.

**Returns:** *Promise*<NativeBalance\>

ETH balance

Defined in: [packages/core-ethereum/src/index.ts:103](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L103)

___

### waitForPublicNodes

▸ **waitForPublicNodes**(): *Promise*<Multiaddr[]\>

**Returns:** *Promise*<Multiaddr[]\>

Defined in: [packages/core-ethereum/src/index.ts:116](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L116)

___

### withdraw

▸ **withdraw**(`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `currency` | ``"NATIVE"`` \| ``"HOPR"`` |
| `recipient` | *string* |
| `amount` | *string* |

**Returns:** *Promise*<string\>

Defined in: [packages/core-ethereum/src/index.ts:57](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L57)

___

### create

▸ `Static` **create**(`db`: *HoprDB*, `privateKey`: *Uint8Array*, `options?`: { `maxConfirmations?`: *number* ; `provider?`: *string*  }): *Promise*<[*default*](index.default.md)\>

Creates an uninitialised instance.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `db` | *HoprDB* | database instance |
| `privateKey` | *Uint8Array* | that is used to derive that on-chain identity |
| `options?` | *object* | - |
| `options.maxConfirmations?` | *number* | - |
| `options.provider?` | *string* | provider URI that is used to connect to the blockchain |

**Returns:** *Promise*<[*default*](index.default.md)\>

a promise resolved to the connector

Defined in: [packages/core-ethereum/src/index.ts:128](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/index.ts#L128)
