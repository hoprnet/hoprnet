[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / default

# Class: default

## Table of contents

### Constructors

- [constructor](default.md#constructor)

### Properties

- [CHAIN\_NAME](default.md#chain_name)
- [cachedGetBalance](default.md#cachedgetbalance)
- [cachedGetNativeBalance](default.md#cachedgetnativebalance)
- [indexer](default.md#indexer)
- [privateKey](default.md#privatekey)

### Methods

- [announce](default.md#announce)
- [getAccount](default.md#getaccount)
- [getAddress](default.md#getaddress)
- [getBalance](default.md#getbalance)
- [getChannel](default.md#getchannel)
- [getChannelsFromPeer](default.md#getchannelsfrompeer)
- [getChannelsOf](default.md#getchannelsof)
- [getNativeBalance](default.md#getnativebalance)
- [getPublicKey](default.md#getpublickey)
- [getPublicKeyOf](default.md#getpublickeyof)
- [getRandomChannel](default.md#getrandomchannel)
- [smartContractInfo](default.md#smartcontractinfo)
- [stop](default.md#stop)
- [uncachedGetBalance](default.md#uncachedgetbalance)
- [uncachedGetNativeBalance](default.md#uncachedgetnativebalance)
- [waitForPublicNodes](default.md#waitforpublicnodes)
- [withdraw](default.md#withdraw)
- [create](default.md#create)

## Constructors

### constructor

\+ **new default**(`chain`: { `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => *HoprChannels* ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *any*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `openChannel`: (`me`: *any*, `counterparty`: *any*, `amount`: *any*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *any*, `ackTicket`: *any*, `ticket`: *any*) => *Promise*<string\> ; `setCommitment`: (`comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => *HoprChannels* ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }, `db`: *HoprDB*, `indexer`: [*Indexer*](indexer.md)): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `chain` | *object* |
| `chain.announce` | (`multiaddr`: Multiaddr) => *Promise*<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: *any*) => *Promise*<string\> |
| `chain.fundChannel` | (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> |
| `chain.getBalance` | (`address`: *Address*) => *Promise*<Balance\> |
| `chain.getChannels` | () => *HoprChannels* |
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
| `chain.subscribeChannelEvents` | (`cb`: *any*) => *HoprChannels* |
| `chain.subscribeError` | (`cb`: *any*) => *void* |
| `chain.unsubscribe` | () => *void* |
| `chain.waitUntilReady` | () => *Promise*<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\> |
| `db` | *HoprDB* |
| `indexer` | [*Indexer*](indexer.md) |

**Returns:** [*default*](default.md)

Defined in: [core-ethereum/src/index.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L33)

## Properties

### CHAIN\_NAME

• `Readonly` **CHAIN\_NAME**: ``"HOPR on Ethereum"``= 'HOPR on Ethereum'

Defined in: [core-ethereum/src/index.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L39)

___

### cachedGetBalance

• `Private` **cachedGetBalance**: () => *Promise*<Balance\>

#### Type declaration

▸ (): *Promise*<Balance\>

**Returns:** *Promise*<Balance\>

Defined in: utils/lib/cache.d.ts:1

Defined in: [core-ethereum/src/index.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L82)

___

### cachedGetNativeBalance

• `Private` **cachedGetNativeBalance**: () => *Promise*<NativeBalance\>

#### Type declaration

▸ (): *Promise*<NativeBalance\>

**Returns:** *Promise*<NativeBalance\>

Defined in: utils/lib/cache.d.ts:1

Defined in: [core-ethereum/src/index.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L104)

___

### indexer

• **indexer**: [*Indexer*](indexer.md)

___

### privateKey

• `Private` **privateKey**: *Uint8Array*

Defined in: [core-ethereum/src/index.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L33)

## Methods

### announce

▸ **announce**(`multiaddr`: *Multiaddr*): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | *Multiaddr* |

**Returns:** *Promise*<string\>

Defined in: [core-ethereum/src/index.ts:53](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L53)

___

### getAccount

▸ **getAccount**(`addr`: *Address*): *Promise*<AccountEntry\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<AccountEntry\>

Defined in: [core-ethereum/src/index.ts:69](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L69)

___

### getAddress

▸ **getAddress**(): *Address*

**Returns:** *Address*

Defined in: [core-ethereum/src/index.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L91)

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

Defined in: [core-ethereum/src/index.ts:87](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L87)

___

### getChannel

▸ **getChannel**(`src`: *PublicKey*, `counterparty`: *PublicKey*): [*Channel*](channel.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `src` | *PublicKey* |
| `counterparty` | *PublicKey* |

**Returns:** [*Channel*](channel.md)

Defined in: [core-ethereum/src/index.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L49)

___

### getChannelsFromPeer

▸ **getChannelsFromPeer**(`p`: *PeerId*): *Promise*<[*RoutingChannel*](../modules.md#routingchannel)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `p` | *PeerId* |

**Returns:** *Promise*<[*RoutingChannel*](../modules.md#routingchannel)[]\>

Defined in: [core-ethereum/src/index.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L61)

___

### getChannelsOf

▸ **getChannelsOf**(`addr`: *Address*): *Promise*<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<ChannelEntry[]\>

Defined in: [core-ethereum/src/index.ts:65](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L65)

___

### getNativeBalance

▸ **getNativeBalance**(`useCache?`: *boolean*): *Promise*<NativeBalance\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `useCache` | *boolean* | false |

**Returns:** *Promise*<NativeBalance\>

Defined in: [core-ethereum/src/index.ts:108](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L108)

___

### getPublicKey

▸ **getPublicKey**(): *PublicKey*

**Returns:** *PublicKey*

Defined in: [core-ethereum/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L95)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`: *Address*): *Promise*<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<PublicKey\>

Defined in: [core-ethereum/src/index.ts:73](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L73)

___

### getRandomChannel

▸ **getRandomChannel**(): *Promise*<[*RoutingChannel*](../modules.md#routingchannel)\>

**Returns:** *Promise*<[*RoutingChannel*](../modules.md#routingchannel)\>

Defined in: [core-ethereum/src/index.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L77)

___

### smartContractInfo

▸ **smartContractInfo**(): *string*

**Returns:** *string*

Defined in: [core-ethereum/src/index.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L112)

___

### stop

▸ **stop**(): *Promise*<void\>

Stops the connector.

**Returns:** *Promise*<void\>

Defined in: [core-ethereum/src/index.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L44)

___

### uncachedGetBalance

▸ `Private` **uncachedGetBalance**(): *Promise*<Balance\>

**Returns:** *Promise*<Balance\>

Defined in: [core-ethereum/src/index.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L81)

___

### uncachedGetNativeBalance

▸ `Private` **uncachedGetNativeBalance**(): *Promise*<NativeBalance\>

Retrieves ETH balance, optionally uses the cache.

**Returns:** *Promise*<NativeBalance\>

ETH balance

Defined in: [core-ethereum/src/index.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L103)

___

### waitForPublicNodes

▸ **waitForPublicNodes**(): *Promise*<Multiaddr[]\>

**Returns:** *Promise*<Multiaddr[]\>

Defined in: [core-ethereum/src/index.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L116)

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

Defined in: [core-ethereum/src/index.ts:57](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L57)

___

### create

▸ `Static` **create**(`db`: *HoprDB*, `privateKey`: *Uint8Array*, `options?`: { `maxConfirmations?`: *number* ; `provider?`: *string*  }): *Promise*<[*default*](default.md)\>

Creates an uninitialised instance.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `db` | *HoprDB* | database instance |
| `privateKey` | *Uint8Array* | that is used to derive that on-chain identity |
| `options?` | *object* | - |
| `options.maxConfirmations?` | *number* | - |
| `options.provider?` | *string* | provider URI that is used to connect to the blockchain |

**Returns:** *Promise*<[*default*](default.md)\>

a promise resolved to the connector

Defined in: [core-ethereum/src/index.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L128)
