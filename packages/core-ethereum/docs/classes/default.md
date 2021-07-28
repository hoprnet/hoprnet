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
- [cachedGetBalance](default.md#cachedgetbalance)
- [cachedGetNativeBalance](default.md#cachedgetnativebalance)
- [chain](default.md#chain)
- [indexer](default.md#indexer)
- [started](default.md#started)
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
- [start](default.md#start)
- [stop](default.md#stop)
- [uncachedGetBalance](default.md#uncachedgetbalance)
- [uncachedGetNativeBalance](default.md#uncachedgetnativebalance)
- [waitForPublicNodes](default.md#waitforpublicnodes)
- [withdraw](default.md#withdraw)
- [getEventListener](default.md#geteventlistener)
- [listenerCount](default.md#listenercount)
- [on](default.md#on)
- [once](default.md#once)

## Constructors

### constructor

• **new default**(`db`, `publicKey`, `privateKey`, `options?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | `HoprDB` |
| `publicKey` | `PublicKey` |
| `privateKey` | `Uint8Array` |
| `options?` | `Object` |
| `options.maxConfirmations?` | `number` |
| `options.provider?` | `string` |

#### Overrides

EventEmitter.constructor

#### Defined in

[core-ethereum/src/index.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L44)

## Properties

### CHAIN\_NAME

• `Readonly` **CHAIN\_NAME**: ``"HOPR on Ethereum"``

#### Defined in

[core-ethereum/src/index.ts:78](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L78)

___

### cachedGetBalance

• `Private` **cachedGetBalance**: () => `Promise`<`Balance`\>

#### Type declaration

▸ (): `Promise`<`Balance`\>

##### Returns

`Promise`<`Balance`\>

#### Defined in

[core-ethereum/src/index.ts:125](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L125)

___

### cachedGetNativeBalance

• `Private` **cachedGetNativeBalance**: () => `Promise`<`NativeBalance`\>

#### Type declaration

▸ (): `Promise`<`NativeBalance`\>

##### Returns

`Promise`<`NativeBalance`\>

#### Defined in

[core-ethereum/src/index.ts:143](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L143)

___

### chain

• `Private` **chain**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `announce` | (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> |
| `finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> |
| `getBalance` | (`address`: `Address`) => `Promise`<`Balance`\> |
| `getChannels` | () => `HoprChannels` |
| `getGenesisBlock` | () => `number` |
| `getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` ; `hoprTokenAddress`: `string` ; `network`: `Networks`  } |
| `getLatestBlockNumber` | () => `Promise`<`number`\> |
| `getNativeBalance` | (`address`: `Address`) => `Promise`<`NativeBalance`\> |
| `getPrivateKey` | () => `Uint8Array` |
| `getPublicKey` | () => `PublicKey` |
| `getWallet` | () => `Wallet` |
| `initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> |
| `redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> |
| `setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> |
| `subscribeBlock` | (`cb`: `any`) => `JsonRpcProvider` \| `WebSocketProvider` |
| `subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `subscribeError` | (`cb`: `any`) => `void` |
| `unsubscribe` | () => `void` |
| `waitUntilReady` | () => `Promise`<`Network`\> |
| `withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\> |

#### Defined in

[core-ethereum/src/index.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L41)

___

### indexer

• **indexer**: [`Indexer`](Indexer.md)

#### Defined in

[core-ethereum/src/index.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L40)

___

### started

• `Private` **started**: `Promise`<[`default`](default.md)\>

#### Defined in

[core-ethereum/src/index.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L42)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [`captureRejectionSymbol`](default.md#capturerejectionsymbol)

#### Inherited from

EventEmitter.captureRejectionSymbol

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:46

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:52

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:53

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

core-ethereum/node_modules/@types/node/events.d.ts:45

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

core-ethereum/node_modules/@types/node/events.d.ts:72

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

[core-ethereum/src/index.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L92)

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

core-ethereum/node_modules/@types/node/events.d.ts:82

___

### eventNames

▸ **eventNames**(): (`string` \| `symbol`)[]

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:87

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

[core-ethereum/src/index.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L112)

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

[core-ethereum/src/index.ts:130](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L130)

___

### getChannel

▸ **getChannel**(`src`, `counterparty`): [`Channel`](Channel.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `src` | `PublicKey` |
| `counterparty` | `PublicKey` |

#### Returns

[`Channel`](Channel.md)

#### Defined in

[core-ethereum/src/index.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L88)

___

### getChannelsFrom

▸ **getChannelsFrom**(`addr`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[core-ethereum/src/index.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L104)

___

### getChannelsTo

▸ **getChannelsTo**(`addr`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[core-ethereum/src/index.ts:108](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L108)

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:79

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

[core-ethereum/src/index.ts:147](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L147)

___

### getOpenChannelsFrom

▸ **getOpenChannelsFrom**(`p`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `p` | `PublicKey` |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[core-ethereum/src/index.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L100)

___

### getPublicKey

▸ **getPublicKey**(): `PublicKey`

#### Returns

`PublicKey`

#### Defined in

[core-ethereum/src/index.ts:134](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L134)

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

[core-ethereum/src/index.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L116)

___

### getRandomOpenChannel

▸ **getRandomOpenChannel**(): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[core-ethereum/src/index.ts:120](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L120)

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

core-ethereum/node_modules/@types/node/events.d.ts:83

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

core-ethereum/node_modules/@types/node/events.d.ts:80

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

core-ethereum/node_modules/@types/node/events.d.ts:76

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

core-ethereum/node_modules/@types/node/events.d.ts:73

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

core-ethereum/node_modules/@types/node/events.d.ts:74

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

core-ethereum/node_modules/@types/node/events.d.ts:85

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

core-ethereum/node_modules/@types/node/events.d.ts:86

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

core-ethereum/node_modules/@types/node/events.d.ts:81

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

core-ethereum/node_modules/@types/node/events.d.ts:77

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

core-ethereum/node_modules/@types/node/events.d.ts:75

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

core-ethereum/node_modules/@types/node/events.d.ts:78

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

[core-ethereum/src/index.ts:151](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L151)

___

### start

▸ **start**(): `Promise`<[`default`](default.md)\>

#### Returns

`Promise`<[`default`](default.md)\>

#### Defined in

[core-ethereum/src/index.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L60)

___

### stop

▸ **stop**(): `Promise`<`void`\>

Stops the connector.

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/index.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L83)

___

### uncachedGetBalance

▸ `Private` **uncachedGetBalance**(): `Promise`<`Balance`\>

#### Returns

`Promise`<`Balance`\>

#### Defined in

[core-ethereum/src/index.ts:124](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L124)

___

### uncachedGetNativeBalance

▸ `Private` **uncachedGetNativeBalance**(): `Promise`<`NativeBalance`\>

Retrieves ETH balance, optionally uses the cache.

#### Returns

`Promise`<`NativeBalance`\>

ETH balance

#### Defined in

[core-ethereum/src/index.ts:142](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L142)

___

### waitForPublicNodes

▸ **waitForPublicNodes**(): `Promise`<`Multiaddr`[]\>

#### Returns

`Promise`<`Multiaddr`[]\>

#### Defined in

[core-ethereum/src/index.ts:160](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L160)

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

[core-ethereum/src/index.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L96)

___

### getEventListener

▸ `Static` **getEventListener**(`emitter`, `name`): `Function`[]

Returns a list listener for a specific emitter event name.

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` \| `EventEmitter` |
| `name` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.getEventListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:34

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

core-ethereum/node_modules/@types/node/events.d.ts:30

___

### on

▸ `Static` **on**(`emitter`, `event`, `options?`): `AsyncIterableIterator`<`any`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `EventEmitter` |
| `event` | `string` |
| `options?` | `StaticEventEmitterOptions` |

#### Returns

`AsyncIterableIterator`<`any`\>

#### Inherited from

EventEmitter.on

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:27

___

### once

▸ `Static` **once**(`emitter`, `event`, `options?`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `NodeEventTarget` |
| `event` | `string` \| `symbol` |
| `options?` | `StaticEventEmitterOptions` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:25

▸ `Static` **once**(`emitter`, `event`, `options?`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` |
| `event` | `string` |
| `options?` | `StaticEventEmitterOptions` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:26
