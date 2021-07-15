[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / default

# Class: default

## Hierarchy

- `EventEmitter`

  ↳ **`default`**

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
- [captureRejectionSymbol](default.md#capturerejectionsymbol)
- [captureRejections](default.md#capturerejections)
- [defaultMaxListeners](default.md#defaultmaxlisteners)
- [errorMonitor](default.md#errormonitor)

### Methods

- [addListener](default.md#addlistener)
- [announce](default.md#announce)
- [emit](default.md#emit)
- [eventNames](default.md#eventnames)
- [getAccount](default.md#getaccount)
- [getAddress](default.md#getaddress)
- [getBalance](default.md#getbalance)
- [getChannel](default.md#getchannel)
- [getChannelsFrom](default.md#getchannelsfrom)
- [getChannelsTo](default.md#getchannelsto)
- [getMaxListeners](default.md#getmaxlisteners)
- [getNativeBalance](default.md#getnativebalance)
- [getOpenChannelsFrom](default.md#getopenchannelsfrom)
- [getPublicKey](default.md#getpublickey)
- [getPublicKeyOf](default.md#getpublickeyof)
- [getRandomOpenChannel](default.md#getrandomopenchannel)
- [listenerCount](default.md#listenercount)
- [listeners](default.md#listeners)
- [off](default.md#off)
- [on](default.md#on)
- [once](default.md#once)
- [prependListener](default.md#prependlistener)
- [prependOnceListener](default.md#prependoncelistener)
- [rawListeners](default.md#rawlisteners)
- [removeAllListeners](default.md#removealllisteners)
- [removeListener](default.md#removelistener)
- [setMaxListeners](default.md#setmaxlisteners)
- [smartContractInfo](default.md#smartcontractinfo)
- [stop](default.md#stop)
- [uncachedGetBalance](default.md#uncachedgetbalance)
- [uncachedGetNativeBalance](default.md#uncachedgetnativebalance)
- [waitForPublicNodes](default.md#waitforpublicnodes)
- [withdraw](default.md#withdraw)
- [create](default.md#create)
- [listenerCount](default.md#listenercount)
- [on](default.md#on)
- [once](default.md#once)

## Constructors

### constructor

• **new default**(`chain`, `db`, `indexer`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `chain` | `Object` |
| `chain.announce` | (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> |
| `chain.finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `chain.fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> |
| `chain.getBalance` | (`address`: `Address`) => `Promise`<`Balance`\> |
| `chain.getChannels` | () => `HoprChannels` |
| `chain.getGenesisBlock` | () => `number` |
| `chain.getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` ; `hoprTokenAddress`: `string` ; `network`: `Networks`  } |
| `chain.getLatestBlockNumber` | () => `Promise`<`number`\> |
| `chain.getNativeBalance` | (`address`: `Address`) => `Promise`<`NativeBalance`\> |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getWallet` | () => `Wallet` |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `chain.openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> |
| `chain.subscribeBlock` | (`cb`: `any`) => `JsonRpcProvider` \| `WebSocketProvider` |
| `chain.subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `chain.subscribeError` | (`cb`: `any`) => `void` |
| `chain.unsubscribe` | () => `void` |
| `chain.waitUntilReady` | () => `Promise`<`Network`\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\> |
| `db` | `HoprDB` |
| `indexer` | [`Indexer`](indexer.md) |

#### Overrides

EventEmitter.constructor

#### Defined in

[core-ethereum/src/index.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L42)

## Properties

### CHAIN\_NAME

• `Readonly` **CHAIN\_NAME**: ``"HOPR on Ethereum"``

#### Defined in

[core-ethereum/src/index.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L51)

___

### address

• `Private` **address**: `Address`

#### Defined in

[core-ethereum/src/index.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L42)

___

### cachedGetBalance

• `Private` **cachedGetBalance**: () => `Promise`<`Balance`\>

#### Type declaration

▸ (): `Promise`<`Balance`\>

##### Returns

`Promise`<`Balance`\>

#### Defined in

[core-ethereum/src/index.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L98)

___

### cachedGetNativeBalance

• `Private` **cachedGetNativeBalance**: () => `Promise`<`NativeBalance`\>

#### Type declaration

▸ (): `Promise`<`NativeBalance`\>

##### Returns

`Promise`<`NativeBalance`\>

#### Defined in

[core-ethereum/src/index.ts:120](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L120)

___

### indexer

• **indexer**: [`Indexer`](indexer.md)

___

### privateKey

• `Private` **privateKey**: `Uint8Array`

#### Defined in

[core-ethereum/src/index.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L40)

___

### publicKey

• `Private` **publicKey**: `PublicKey`

#### Defined in

[core-ethereum/src/index.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L41)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [`captureRejectionSymbol`](default.md#capturerejectionsymbol)

#### Inherited from

EventEmitter.captureRejectionSymbol

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:43

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:49

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:50

___

### errorMonitor

▪ `Static` `Readonly` **errorMonitor**: typeof [`errorMonitor`](default.md#errormonitor)

This symbol shall be used to install a listener for only monitoring `'error'`
events. Listeners installed using this symbol are called before the regular
`'error'` listeners are called.

Installing a listener using this symbol does not change the behavior once an
`'error'` event is emitted, therefore the process will still crash if no
regular `'error'` listener is installed.

#### Inherited from

EventEmitter.errorMonitor

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:42

## Methods

### addListener

▸ **addListener**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:62

___

### announce

▸ **announce**(`multiaddr`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | `Multiaddr` |

#### Returns

`Promise`<`string`\>

#### Defined in

[core-ethereum/src/index.ts:65](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L65)

___

### emit

▸ **emit**(`event`, ...`args`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `...args` | `any`[] |

#### Returns

`boolean`

#### Inherited from

EventEmitter.emit

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:72

___

### eventNames

▸ **eventNames**(): (`string` \| `symbol`)[]

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:77

___

### getAccount

▸ **getAccount**(`addr`): `Promise`<`AccountEntry`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`AccountEntry`\>

#### Defined in

[core-ethereum/src/index.ts:85](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L85)

___

### getAddress

▸ **getAddress**(): `Address`

#### Returns

`Address`

#### Defined in

[core-ethereum/src/index.ts:107](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L107)

___

### getBalance

▸ **getBalance**(`useCache?`): `Promise`<`Balance`\>

Retrieves HOPR balance, optionally uses the cache.

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `useCache` | `boolean` | `false` |

#### Returns

`Promise`<`Balance`\>

HOPR balance

#### Defined in

[core-ethereum/src/index.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L103)

___

### getChannel

▸ **getChannel**(`src`, `counterparty`): [`Channel`](channel.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `src` | `PublicKey` |
| `counterparty` | `PublicKey` |

#### Returns

[`Channel`](channel.md)

#### Defined in

[core-ethereum/src/index.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L61)

___

### getChannelsFrom

▸ **getChannelsFrom**(`addr`): `Promise`<[`ChannelEntry`](channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)[]\>

#### Defined in

[core-ethereum/src/index.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L77)

___

### getChannelsTo

▸ **getChannelsTo**(`addr`): `Promise`<[`ChannelEntry`](channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)[]\>

#### Defined in

[core-ethereum/src/index.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L81)

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:69

___

### getNativeBalance

▸ **getNativeBalance**(`useCache?`): `Promise`<`NativeBalance`\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `useCache` | `boolean` | `false` |

#### Returns

`Promise`<`NativeBalance`\>

#### Defined in

[core-ethereum/src/index.ts:124](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L124)

___

### getOpenChannelsFrom

▸ **getOpenChannelsFrom**(`p`): `Promise`<[`ChannelEntry`](channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `p` | `PublicKey` |

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)[]\>

#### Defined in

[core-ethereum/src/index.ts:73](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L73)

___

### getPublicKey

▸ **getPublicKey**(): `PublicKey`

#### Returns

`PublicKey`

#### Defined in

[core-ethereum/src/index.ts:111](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L111)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`): `Promise`<`PublicKey`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`PublicKey`\>

#### Defined in

[core-ethereum/src/index.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L89)

___

### getRandomOpenChannel

▸ **getRandomOpenChannel**(): `Promise`<[`ChannelEntry`](channelentry.md)\>

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)\>

#### Defined in

[core-ethereum/src/index.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L93)

___

### listenerCount

▸ **listenerCount**(`event`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |

#### Returns

`number`

#### Inherited from

EventEmitter.listenerCount

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:73

___

### listeners

▸ **listeners**(`event`): `Function`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.listeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:70

___

### off

▸ **off**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.off

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:66

___

### on

▸ **on**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.on

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:63

___

### once

▸ **once**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:64

___

### prependListener

▸ **prependListener**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.prependOnceListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:76

___

### rawListeners

▸ **rawListeners**(`event`): `Function`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.rawListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:71

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:67

___

### removeListener

▸ **removeListener**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:65

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:68

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

[core-ethereum/src/index.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L128)

___

### stop

▸ **stop**(): `Promise`<`void`\>

Stops the connector.

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/index.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L56)

___

### uncachedGetBalance

▸ `Private` **uncachedGetBalance**(): `Promise`<`Balance`\>

#### Returns

`Promise`<`Balance`\>

#### Defined in

[core-ethereum/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L97)

___

### uncachedGetNativeBalance

▸ `Private` **uncachedGetNativeBalance**(): `Promise`<`NativeBalance`\>

Retrieves ETH balance, optionally uses the cache.

#### Returns

`Promise`<`NativeBalance`\>

ETH balance

#### Defined in

[core-ethereum/src/index.ts:119](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L119)

___

### waitForPublicNodes

▸ **waitForPublicNodes**(): `Promise`<`Multiaddr`[]\>

#### Returns

`Promise`<`Multiaddr`[]\>

#### Defined in

[core-ethereum/src/index.ts:137](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L137)

___

### withdraw

▸ **withdraw**(`currency`, `recipient`, `amount`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `currency` | ``"NATIVE"`` \| ``"HOPR"`` |
| `recipient` | `string` |
| `amount` | `string` |

#### Returns

`Promise`<`string`\>

#### Defined in

[core-ethereum/src/index.ts:69](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L69)

___

### create

▸ `Static` **create**(`db`, `privateKey`, `options?`): `Promise`<[`default`](default.md)\>

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

`Promise`<[`default`](default.md)\>

a promise resolved to the connector

#### Defined in

[core-ethereum/src/index.ts:149](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L149)

___

### listenerCount

▸ `Static` **listenerCount**(`emitter`, `event`): `number`

**`deprecated`** since v4.0.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `EventEmitter` |
| `event` | `string` \| `symbol` |

#### Returns

`number`

#### Inherited from

EventEmitter.listenerCount

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:31

___

### on

▸ `Static` **on**(`emitter`, `event`): `AsyncIterableIterator`<`any`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `EventEmitter` |
| `event` | `string` |

#### Returns

`AsyncIterableIterator`<`any`\>

#### Inherited from

EventEmitter.on

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:28

___

### once

▸ `Static` **once**(`emitter`, `event`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `NodeEventTarget` |
| `event` | `string` \| `symbol` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:26

▸ `Static` **once**(`emitter`, `event`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` |
| `event` | `string` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:27
