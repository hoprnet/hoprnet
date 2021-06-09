[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / default

# Class: default

## Table of contents

### Constructors

- [constructor](default.md#constructor)

### Properties

- [CHAIN\_NAME](default.md#chain_name)
- [address](default.md#address)
- [cachedGetBalance](default.md#cachedgetbalance)
- [cachedGetNativeBalance](default.md#cachedgetnativebalance)
- [indexer](default.md#indexer)
- [privateKey](default.md#privatekey)
- [publicKey](default.md#publickey)

### Methods

- [announce](default.md#announce)
- [getAccount](default.md#getaccount)
- [getAddress](default.md#getaddress)
- [getBalance](default.md#getbalance)
- [getChannel](default.md#getchannel)
- [getChannelsOf](default.md#getchannelsof)
- [getNativeBalance](default.md#getnativebalance)
- [getOpenRoutingChannelsFromPeer](default.md#getopenroutingchannelsfrompeer)
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

• **new default**(`chain`, `db`, `indexer`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `chain` | `Object` |
| `chain.announce` | (`multiaddr`: `Multiaddr`) => `Promise`<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<string\> |
| `chain.fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<string\> |
| `chain.getBalance` | (`address`: `Address`) => `Promise`<Balance\> |
| `chain.getChannels` | () => `HoprChannels` |
| `chain.getGenesisBlock` | () => `number` |
| `chain.getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` ; `hoprTokenAddress`: `string` ; `network`: `Networks`  } |
| `chain.getLatestBlockNumber` | () => `Promise`<number\> |
| `chain.getNativeBalance` | (`address`: `Address`) => `Promise`<NativeBalance\> |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getWallet` | () => `Wallet` |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<string\> |
| `chain.openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<string\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<string\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<string\> |
| `chain.subscribeBlock` | (`cb`: `any`) => `JsonRpcProvider` \| `WebSocketProvider` |
| `chain.subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `chain.subscribeError` | (`cb`: `any`) => `void` |
| `chain.unsubscribe` | () => `void` |
| `chain.waitUntilReady` | () => `Promise`<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<string\> |
| `db` | `HoprDB` |
| `indexer` | [Indexer](indexer.md) |

#### Defined in

[core-ethereum/src/index.ts:43](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L43)

## Properties

### CHAIN\_NAME

• `Readonly` **CHAIN\_NAME**: ``"HOPR on Ethereum"``

#### Defined in

[core-ethereum/src/index.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L51)

___

### address

• `Private` **address**: `Address`

#### Defined in

[core-ethereum/src/index.ts:43](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L43)

___

### cachedGetBalance

• `Private` **cachedGetBalance**: () => `Promise`<Balance\>

#### Type declaration

▸ (): `Promise`<Balance\>

##### Returns

`Promise`<Balance\>

#### Defined in

[core-ethereum/src/index.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L94)

___

### cachedGetNativeBalance

• `Private` **cachedGetNativeBalance**: () => `Promise`<NativeBalance\>

#### Type declaration

▸ (): `Promise`<NativeBalance\>

##### Returns

`Promise`<NativeBalance\>

#### Defined in

[core-ethereum/src/index.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L116)

___

### indexer

• **indexer**: [Indexer](indexer.md)

___

### privateKey

• `Private` **privateKey**: `Uint8Array`

#### Defined in

[core-ethereum/src/index.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L41)

___

### publicKey

• `Private` **publicKey**: `PublicKey`

#### Defined in

[core-ethereum/src/index.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L42)

## Methods

### announce

▸ **announce**(`multiaddr`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | `Multiaddr` |

#### Returns

`Promise`<string\>

#### Defined in

[core-ethereum/src/index.ts:65](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L65)

___

### getAccount

▸ **getAccount**(`addr`): `Promise`<AccountEntry\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<AccountEntry\>

#### Defined in

[core-ethereum/src/index.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L81)

___

### getAddress

▸ **getAddress**(): `Address`

#### Returns

`Address`

#### Defined in

[core-ethereum/src/index.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L103)

___

### getBalance

▸ **getBalance**(`useCache?`): `Promise`<Balance\>

Retrieves HOPR balance, optionally uses the cache.

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `useCache` | `boolean` | false |

#### Returns

`Promise`<Balance\>

HOPR balance

#### Defined in

[core-ethereum/src/index.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L99)

___

### getChannel

▸ **getChannel**(`src`, `counterparty`): [Channel](channel.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `src` | `PublicKey` |
| `counterparty` | `PublicKey` |

#### Returns

[Channel](channel.md)

#### Defined in

[core-ethereum/src/index.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L61)

___

### getChannelsOf

▸ **getChannelsOf**(`addr`): `Promise`<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<ChannelEntry[]\>

#### Defined in

[core-ethereum/src/index.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L77)

___

### getNativeBalance

▸ **getNativeBalance**(`useCache?`): `Promise`<NativeBalance\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `useCache` | `boolean` | false |

#### Returns

`Promise`<NativeBalance\>

#### Defined in

[core-ethereum/src/index.ts:120](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L120)

___

### getOpenRoutingChannelsFromPeer

▸ **getOpenRoutingChannelsFromPeer**(`p`): `Promise`<[RoutingChannel](../modules.md#routingchannel)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `p` | `PeerId` |

#### Returns

`Promise`<[RoutingChannel](../modules.md#routingchannel)[]\>

#### Defined in

[core-ethereum/src/index.ts:73](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L73)

___

### getPublicKey

▸ **getPublicKey**(): `PublicKey`

#### Returns

`PublicKey`

#### Defined in

[core-ethereum/src/index.ts:107](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L107)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`): `Promise`<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<PublicKey\>

#### Defined in

[core-ethereum/src/index.ts:85](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L85)

___

### getRandomChannel

▸ **getRandomChannel**(): `Promise`<[RoutingChannel](../modules.md#routingchannel)\>

#### Returns

`Promise`<[RoutingChannel](../modules.md#routingchannel)\>

#### Defined in

[core-ethereum/src/index.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L89)

___

### smartContractInfo

▸ **smartContractInfo**(): `Object`

#### Returns

`Object`

| Name | Type |
| :------ | :------ |
| `channelClosureSecs` | `number` |
| `hoprChannelsAddress` | `string` |
| `hoprTokenAddress` | `string` |
| `network` | `string` |

#### Defined in

[core-ethereum/src/index.ts:124](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L124)

___

### stop

▸ **stop**(): `Promise`<void\>

Stops the connector.

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/index.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L56)

___

### uncachedGetBalance

▸ `Private` **uncachedGetBalance**(): `Promise`<Balance\>

#### Returns

`Promise`<Balance\>

#### Defined in

[core-ethereum/src/index.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L93)

___

### uncachedGetNativeBalance

▸ `Private` **uncachedGetNativeBalance**(): `Promise`<NativeBalance\>

Retrieves ETH balance, optionally uses the cache.

#### Returns

`Promise`<NativeBalance\>

ETH balance

#### Defined in

[core-ethereum/src/index.ts:115](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L115)

___

### waitForPublicNodes

▸ **waitForPublicNodes**(): `Promise`<Multiaddr[]\>

#### Returns

`Promise`<Multiaddr[]\>

#### Defined in

[core-ethereum/src/index.ts:133](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L133)

___

### withdraw

▸ **withdraw**(`currency`, `recipient`, `amount`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `currency` | ``"NATIVE"`` \| ``"HOPR"`` |
| `recipient` | `string` |
| `amount` | `string` |

#### Returns

`Promise`<string\>

#### Defined in

[core-ethereum/src/index.ts:69](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L69)

___

### create

▸ `Static` **create**(`db`, `privateKey`, `options?`): `Promise`<[default](default.md)\>

Creates an uninitialised instance.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `db` | `HoprDB` | database instance |
| `privateKey` | `Uint8Array` | that is used to derive that on-chain identity |
| `options?` | `Object` | - |
| `options.maxConfirmations?` | `number` | - |
| `options.provider?` | `string` | provider URI that is used to connect to the blockchain |

#### Returns

`Promise`<[default](default.md)\>

a promise resolved to the connector

#### Defined in

[core-ethereum/src/index.ts:145](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L145)
