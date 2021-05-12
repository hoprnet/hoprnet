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

\+ **new default**(`chain`: { `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *Address*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => *HoprChannels* ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *Address*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *Address*) => *Promise*<string\> ; `openChannel`: (`me`: *Address*, `counterparty`: *Address*, `amount`: *Balance*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *Address*, `ackTicket`: *AcknowledgedTicket*, `ticket`: *Ticket*) => *Promise*<string\> ; `setCommitment`: (`comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => *HoprChannels* ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }, `db`: *HoprDB*, `indexer`: [*Indexer*](indexer.md)): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `chain` | *object* |
| `chain.announce` | (`multiaddr`: Multiaddr) => *Promise*<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: *Address*) => *Promise*<string\> |
| `chain.fundChannel` | (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> |
| `chain.getBalance` | (`address`: *Address*) => *Promise*<Balance\> |
| `chain.getChannels` | () => *HoprChannels* |
| `chain.getGenesisBlock` | () => *number* |
| `chain.getInfo` | () => *string* |
| `chain.getLatestBlockNumber` | () => *Promise*<number\> |
| `chain.getNativeBalance` | (`address`: *Address*) => *Promise*<NativeBalance\> |
| `chain.getPrivateKey` | () => *Uint8Array* |
| `chain.getPublicKey` | () => *PublicKey* |
| `chain.getWallet` | () => *Wallet* |
| `chain.initiateChannelClosure` | (`counterparty`: *Address*) => *Promise*<string\> |
| `chain.openChannel` | (`me`: *Address*, `counterparty`: *Address*, `amount`: *Balance*) => *Promise*<string\> |
| `chain.redeemTicket` | (`counterparty`: *Address*, `ackTicket`: *AcknowledgedTicket*, `ticket`: *Ticket*) => *Promise*<string\> |
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

Defined in: [core-ethereum/src/index.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L40)

## Properties

### CHAIN\_NAME

• `Readonly` **CHAIN\_NAME**: ``"HOPR on Ethereum"``= 'HOPR on Ethereum'

Defined in: [core-ethereum/src/index.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L46)

___

### cachedGetBalance

• `Private` **cachedGetBalance**: () => *Promise*<Balance\>

#### Type declaration

▸ (): *Promise*<Balance\>

**Returns:** *Promise*<Balance\>

Defined in: [core-ethereum/src/index.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L89)

___

### cachedGetNativeBalance

• `Private` **cachedGetNativeBalance**: () => *Promise*<NativeBalance\>

#### Type declaration

▸ (): *Promise*<NativeBalance\>

**Returns:** *Promise*<NativeBalance\>

Defined in: [core-ethereum/src/index.ts:111](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L111)

___

### indexer

• **indexer**: [*Indexer*](indexer.md)

___

### privateKey

• `Private` **privateKey**: *Uint8Array*

Defined in: [core-ethereum/src/index.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L40)

## Methods

### announce

▸ **announce**(`multiaddr`: *Multiaddr*): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | *Multiaddr* |

**Returns:** *Promise*<string\>

Defined in: [core-ethereum/src/index.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L60)

___

### getAccount

▸ **getAccount**(`addr`: *Address*): *Promise*<AccountEntry\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<AccountEntry\>

Defined in: [core-ethereum/src/index.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L76)

___

### getAddress

▸ **getAddress**(): *Address*

**Returns:** *Address*

Defined in: [core-ethereum/src/index.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L98)

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

Defined in: [core-ethereum/src/index.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L94)

___

### getChannel

▸ **getChannel**(`src`: *PublicKey*, `counterparty`: *PublicKey*): [*Channel*](channel.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `src` | *PublicKey* |
| `counterparty` | *PublicKey* |

**Returns:** [*Channel*](channel.md)

Defined in: [core-ethereum/src/index.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L56)

___

### getChannelsFromPeer

▸ **getChannelsFromPeer**(`p`: *PeerId*): *Promise*<[*RoutingChannel*](../modules.md#routingchannel)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `p` | *PeerId* |

**Returns:** *Promise*<[*RoutingChannel*](../modules.md#routingchannel)[]\>

Defined in: [core-ethereum/src/index.ts:68](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L68)

___

### getChannelsOf

▸ **getChannelsOf**(`addr`: *Address*): *Promise*<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<ChannelEntry[]\>

Defined in: [core-ethereum/src/index.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L72)

___

### getNativeBalance

▸ **getNativeBalance**(`useCache?`: *boolean*): *Promise*<NativeBalance\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `useCache` | *boolean* | false |

**Returns:** *Promise*<NativeBalance\>

Defined in: [core-ethereum/src/index.ts:115](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L115)

___

### getPublicKey

▸ **getPublicKey**(): *PublicKey*

**Returns:** *PublicKey*

Defined in: [core-ethereum/src/index.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L102)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`: *Address*): *Promise*<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<PublicKey\>

Defined in: [core-ethereum/src/index.ts:80](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L80)

___

### getRandomChannel

▸ **getRandomChannel**(): *Promise*<[*RoutingChannel*](../modules.md#routingchannel)\>

**Returns:** *Promise*<[*RoutingChannel*](../modules.md#routingchannel)\>

Defined in: [core-ethereum/src/index.ts:84](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L84)

___

### smartContractInfo

▸ **smartContractInfo**(): *string*

**Returns:** *string*

Defined in: [core-ethereum/src/index.ts:119](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L119)

___

### stop

▸ **stop**(): *Promise*<void\>

Stops the connector.

**Returns:** *Promise*<void\>

Defined in: [core-ethereum/src/index.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L51)

___

### uncachedGetBalance

▸ `Private` **uncachedGetBalance**(): *Promise*<Balance\>

**Returns:** *Promise*<Balance\>

Defined in: [core-ethereum/src/index.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L88)

___

### uncachedGetNativeBalance

▸ `Private` **uncachedGetNativeBalance**(): *Promise*<NativeBalance\>

Retrieves ETH balance, optionally uses the cache.

**Returns:** *Promise*<NativeBalance\>

ETH balance

Defined in: [core-ethereum/src/index.ts:110](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L110)

___

### waitForPublicNodes

▸ **waitForPublicNodes**(): *Promise*<Multiaddr[]\>

**Returns:** *Promise*<Multiaddr[]\>

Defined in: [core-ethereum/src/index.ts:123](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L123)

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

Defined in: [core-ethereum/src/index.ts:64](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L64)

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

Defined in: [core-ethereum/src/index.ts:135](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L135)
