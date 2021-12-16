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
- [automaticChainCreation](default.md#automaticchaincreation)
- [cachedGetBalance](default.md#cachedgetbalance)
- [cachedGetNativeBalance](default.md#cachedgetnativebalance)
- [chain](default.md#chain)
- [indexer](default.md#indexer)
- [redeemingAll](default.md#redeemingall)
- [started](default.md#started)
- [captureRejectionSymbol](default.md#capturerejectionsymbol)
- [captureRejections](default.md#capturerejections)
- [defaultMaxListeners](default.md#defaultmaxlisteners)
- [errorMonitor](default.md#errormonitor)

### Methods

- [addListener](default.md#addlistener)
- [announce](default.md#announce)
- [commitToChannel](default.md#committochannel)
- [createChain](default.md#createchain)
- [emit](default.md#emit)
- [eventNames](default.md#eventnames)
- [finalizeClosure](default.md#finalizeclosure)
- [fundChannel](default.md#fundchannel)
- [getAccount](default.md#getaccount)
- [getBalance](default.md#getbalance)
- [getChain](default.md#getchain)
- [getMaxListeners](default.md#getmaxlisteners)
- [getNativeBalance](default.md#getnativebalance)
- [getOpenChannelsFrom](default.md#getopenchannelsfrom)
- [getPublicKey](default.md#getpublickey)
- [getPublicKeyOf](default.md#getpublickeyof)
- [getRandomOpenChannel](default.md#getrandomopenchannel)
- [initializeClosure](default.md#initializeclosure)
- [listenerCount](default.md#listenercount)
- [listeners](default.md#listeners)
- [off](default.md#off)
- [on](default.md#on)
- [once](default.md#once)
- [openChannel](default.md#openchannel)
- [prependListener](default.md#prependlistener)
- [prependOnceListener](default.md#prependoncelistener)
- [rawListeners](default.md#rawlisteners)
- [redeemAllTickets](default.md#redeemalltickets)
- [redeemTicket](default.md#redeemticket)
- [redeemTicketsInChannel](default.md#redeemticketsinchannel)
- [removeAllListeners](default.md#removealllisteners)
- [removeListener](default.md#removelistener)
- [setMaxListeners](default.md#setmaxlisteners)
- [setTxHandler](default.md#settxhandler)
- [smartContractInfo](default.md#smartcontractinfo)
- [start](default.md#start)
- [stop](default.md#stop)
- [uncachedGetBalance](default.md#uncachedgetbalance)
- [uncachedGetNativeBalance](default.md#uncachedgetnativebalance)
- [waitForPublicNodes](default.md#waitforpublicnodes)
- [withdraw](default.md#withdraw)
- [getEventListeners](default.md#geteventlisteners)
- [listenerCount](default.md#listenercount)
- [on](default.md#on)
- [once](default.md#once)

## Constructors

### constructor

• **new default**(`db`, `publicKey`, `privateKey`, `options`, `automaticChainCreation?`)

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `db` | `HoprDB` | `undefined` |
| `publicKey` | `PublicKey` | `undefined` |
| `privateKey` | `Uint8Array` | `undefined` |
| `options` | [`ChainOptions`](../modules.md#chainoptions) | `undefined` |
| `automaticChainCreation` | `boolean` | `true` |

#### Overrides

EventEmitter.constructor

#### Defined in

[packages/core-ethereum/src/index.ts:63](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L63)

## Properties

### CHAIN\_NAME

• `Readonly` **CHAIN\_NAME**: ``"HOPR on Ethereum"``

#### Defined in

[packages/core-ethereum/src/index.ts:121](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L121)

___

### automaticChainCreation

• `Protected` **automaticChainCreation**: `boolean` = `true`

___

### cachedGetBalance

• `Private` **cachedGetBalance**: () => `Promise`<`Balance`\>

#### Type declaration

▸ (): `Promise`<`Balance`\>

##### Returns

`Promise`<`Balance`\>

#### Defined in

[packages/core-ethereum/src/index.ts:163](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L163)

___

### cachedGetNativeBalance

• `Private` **cachedGetNativeBalance**: () => `Promise`<`NativeBalance`\>

#### Type declaration

▸ (): `Promise`<`NativeBalance`\>

##### Returns

`Promise`<`NativeBalance`\>

#### Defined in

[packages/core-ethereum/src/index.ts:184](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L184)

___

### chain

• `Private` **chain**: `Object`

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

[packages/core-ethereum/src/index.ts:59](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L59)

___

### indexer

• **indexer**: [`Indexer`](Indexer.md)

#### Defined in

[packages/core-ethereum/src/index.ts:58](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L58)

___

### redeemingAll

• `Private` **redeemingAll**: `Promise`<`void`\> = `undefined`

#### Defined in

[packages/core-ethereum/src/index.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L61)

___

### started

• `Private` **started**: `Promise`<[`default`](default.md)\>

#### Defined in

[packages/core-ethereum/src/index.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L60)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [`captureRejectionSymbol`](default.md#capturerejectionsymbol)

#### Inherited from

EventEmitter.captureRejectionSymbol

#### Defined in

node_modules/@types/node/events.d.ts:273

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

node_modules/@types/node/events.d.ts:278

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:279

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

node_modules/@types/node/events.d.ts:272

## Methods

### addListener

▸ **addListener**(`eventName`, `listener`): [`default`](default.md)

Alias for `emitter.on(eventName, listener)`.

**`since`** v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

node_modules/@types/node/events.d.ts:299

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

[packages/core-ethereum/src/index.ts:131](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L131)

___

### commitToChannel

▸ **commitToChannel**(`c`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | [`ChannelEntry`](ChannelEntry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/index.ts:205](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L205)

___

### createChain

▸ `Private` **createChain**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/index.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L89)

___

### emit

▸ **emit**(`eventName`, ...`args`): `boolean`

Synchronously calls each of the listeners registered for the event named`eventName`, in the order they were registered, passing the supplied arguments
to each.

Returns `true` if the event had listeners, `false` otherwise.

```js
const EventEmitter = require('events');
const myEmitter = new EventEmitter();

// First listener
myEmitter.on('event', function firstListener() {
  console.log('Helloooo! first listener');
});
// Second listener
myEmitter.on('event', function secondListener(arg1, arg2) {
  console.log(`event with parameters ${arg1}, ${arg2} in second listener`);
});
// Third listener
myEmitter.on('event', function thirdListener(...args) {
  const parameters = args.join(', ');
  console.log(`event with parameters ${parameters} in third listener`);
});

console.log(myEmitter.listeners('event'));

myEmitter.emit('event', 1, 2, 3, 4, 5);

// Prints:
// [
//   [Function: firstListener],
//   [Function: secondListener],
//   [Function: thirdListener]
// ]
// Helloooo! first listener
// event with parameters 1, 2 in second listener
// event with parameters 1, 2, 3, 4, 5 in third listener
```

**`since`** v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |
| `...args` | `any`[] |

#### Returns

`boolean`

#### Inherited from

EventEmitter.emit

#### Defined in

node_modules/@types/node/events.d.ts:555

___

### eventNames

▸ **eventNames**(): (`string` \| `symbol`)[]

Returns an array listing the events for which the emitter has registered
listeners. The values in the array are strings or `Symbol`s.

```js
const EventEmitter = require('events');
const myEE = new EventEmitter();
myEE.on('foo', () => {});
myEE.on('bar', () => {});

const sym = Symbol('symbol');
myEE.on(sym, () => {});

console.log(myEE.eventNames());
// Prints: [ 'foo', 'bar', Symbol(symbol) ]
```

**`since`** v6.0.0

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

node_modules/@types/node/events.d.ts:614

___

### finalizeClosure

▸ **finalizeClosure**(`dest`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `dest` | `PublicKey` |

#### Returns

`Promise`<`string`\>

#### Defined in

[packages/core-ethereum/src/index.ts:330](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L330)

___

### fundChannel

▸ **fundChannel**(`dest`, `myFund`, `counterpartyFund`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `dest` | `PublicKey` |
| `myFund` | `Balance` |
| `counterpartyFund` | `Balance` |

#### Returns

`Promise`<`string`\>

#### Defined in

[packages/core-ethereum/src/index.ts:360](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L360)

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

[packages/core-ethereum/src/index.ts:150](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L150)

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

[packages/core-ethereum/src/index.ts:168](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L168)

___

### getChain

▸ **getChain**(): `Object`

#### Returns

`Object`

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

[packages/core-ethereum/src/index.ts:117](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L117)

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

Returns the current max listener value for the `EventEmitter` which is either
set by `emitter.setMaxListeners(n)` or defaults to [defaultMaxListeners](default.md#defaultmaxlisteners).

**`since`** v1.0.0

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:471

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

[packages/core-ethereum/src/index.ts:188](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L188)

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

[packages/core-ethereum/src/index.ts:146](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L146)

___

### getPublicKey

▸ **getPublicKey**(): `PublicKey`

#### Returns

`PublicKey`

#### Defined in

[packages/core-ethereum/src/index.ts:172](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L172)

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

[packages/core-ethereum/src/index.ts:154](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L154)

___

### getRandomOpenChannel

▸ **getRandomOpenChannel**(): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[packages/core-ethereum/src/index.ts:158](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L158)

___

### initializeClosure

▸ **initializeClosure**(`dest`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `dest` | `PublicKey` |

#### Returns

`Promise`<`string`\>

#### Defined in

[packages/core-ethereum/src/index.ts:322](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L322)

___

### listenerCount

▸ **listenerCount**(`eventName`): `number`

Returns the number of listeners listening to the event named `eventName`.

**`since`** v3.2.0

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `eventName` | `string` \| `symbol` | The name of the event being listened for |

#### Returns

`number`

#### Inherited from

EventEmitter.listenerCount

#### Defined in

node_modules/@types/node/events.d.ts:561

___

### listeners

▸ **listeners**(`eventName`): `Function`[]

Returns a copy of the array of listeners for the event named `eventName`.

```js
server.on('connection', (stream) => {
  console.log('someone connected!');
});
console.log(util.inspect(server.listeners('connection')));
// Prints: [ [Function] ]
```

**`since`** v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.listeners

#### Defined in

node_modules/@types/node/events.d.ts:484

___

### off

▸ **off**(`eventName`, `listener`): [`default`](default.md)

Alias for `emitter.removeListener()`.

**`since`** v10.0.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.off

#### Defined in

node_modules/@types/node/events.d.ts:444

___

### on

▸ **on**(`eventName`, `listener`): [`default`](default.md)

Adds the `listener` function to the end of the listeners array for the
event named `eventName`. No checks are made to see if the `listener` has
already been added. Multiple calls passing the same combination of `eventName`and `listener` will result in the `listener` being added, and called, multiple
times.

```js
server.on('connection', (stream) => {
  console.log('someone connected!');
});
```

Returns a reference to the `EventEmitter`, so that calls can be chained.

By default, event listeners are invoked in the order they are added. The`emitter.prependListener()` method can be used as an alternative to add the
event listener to the beginning of the listeners array.

```js
const myEE = new EventEmitter();
myEE.on('foo', () => console.log('a'));
myEE.prependListener('foo', () => console.log('b'));
myEE.emit('foo');
// Prints:
//   b
//   a
```

**`since`** v0.1.101

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `eventName` | `string` \| `symbol` | The name of the event. |
| `listener` | (...`args`: `any`[]) => `void` | The callback function |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.on

#### Defined in

node_modules/@types/node/events.d.ts:330

___

### once

▸ **once**(`eventName`, `listener`): [`default`](default.md)

Adds a **one-time**`listener` function for the event named `eventName`. The
next time `eventName` is triggered, this listener is removed and then invoked.

```js
server.once('connection', (stream) => {
  console.log('Ah, we have our first user!');
});
```

Returns a reference to the `EventEmitter`, so that calls can be chained.

By default, event listeners are invoked in the order they are added. The`emitter.prependOnceListener()` method can be used as an alternative to add the
event listener to the beginning of the listeners array.

```js
const myEE = new EventEmitter();
myEE.once('foo', () => console.log('a'));
myEE.prependOnceListener('foo', () => console.log('b'));
myEE.emit('foo');
// Prints:
//   b
//   a
```

**`since`** v0.3.0

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `eventName` | `string` \| `symbol` | The name of the event. |
| `listener` | (...`args`: `any`[]) => `void` | The callback function |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.once

#### Defined in

node_modules/@types/node/events.d.ts:359

___

### openChannel

▸ **openChannel**(`dest`, `amount`): `Promise`<`Hash`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `dest` | `PublicKey` |
| `amount` | `Balance` |

#### Returns

`Promise`<`Hash`\>

#### Defined in

[packages/core-ethereum/src/index.ts:340](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L340)

___

### prependListener

▸ **prependListener**(`eventName`, `listener`): [`default`](default.md)

Adds the `listener` function to the _beginning_ of the listeners array for the
event named `eventName`. No checks are made to see if the `listener` has
already been added. Multiple calls passing the same combination of `eventName`and `listener` will result in the `listener` being added, and called, multiple
times.

```js
server.prependListener('connection', (stream) => {
  console.log('someone connected!');
});
```

Returns a reference to the `EventEmitter`, so that calls can be chained.

**`since`** v6.0.0

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `eventName` | `string` \| `symbol` | The name of the event. |
| `listener` | (...`args`: `any`[]) => `void` | The callback function |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

node_modules/@types/node/events.d.ts:579

___

### prependOnceListener

▸ **prependOnceListener**(`eventName`, `listener`): [`default`](default.md)

Adds a **one-time**`listener` function for the event named `eventName` to the_beginning_ of the listeners array. The next time `eventName` is triggered, this
listener is removed, and then invoked.

```js
server.prependOnceListener('connection', (stream) => {
  console.log('Ah, we have our first user!');
});
```

Returns a reference to the `EventEmitter`, so that calls can be chained.

**`since`** v6.0.0

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `eventName` | `string` \| `symbol` | The name of the event. |
| `listener` | (...`args`: `any`[]) => `void` | The callback function |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.prependOnceListener

#### Defined in

node_modules/@types/node/events.d.ts:595

___

### rawListeners

▸ **rawListeners**(`eventName`): `Function`[]

Returns a copy of the array of listeners for the event named `eventName`,
including any wrappers (such as those created by `.once()`).

```js
const emitter = new EventEmitter();
emitter.once('log', () => console.log('log once'));

// Returns a new Array with a function `onceWrapper` which has a property
// `listener` which contains the original listener bound above
const listeners = emitter.rawListeners('log');
const logFnWrapper = listeners[0];

// Logs "log once" to the console and does not unbind the `once` event
logFnWrapper.listener();

// Logs "log once" to the console and removes the listener
logFnWrapper();

emitter.on('log', () => console.log('log persistently'));
// Will return a new Array with a single function bound by `.on()` above
const newListeners = emitter.rawListeners('log');

// Logs "log persistently" twice
newListeners[0]();
emitter.emit('log');
```

**`since`** v9.4.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.rawListeners

#### Defined in

node_modules/@types/node/events.d.ts:514

___

### redeemAllTickets

▸ **redeemAllTickets**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/index.ts:225](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L225)

___

### redeemTicket

▸ `Private` **redeemTicket**(`counterparty`, `ackTicket`): `Promise`<[`RedeemTicketResponse`](../modules.md#redeemticketresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `PublicKey` |
| `ackTicket` | `AcknowledgedTicket` |

#### Returns

`Promise`<[`RedeemTicketResponse`](../modules.md#redeemticketresponse)\>

#### Defined in

[packages/core-ethereum/src/index.ts:268](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L268)

___

### redeemTicketsInChannel

▸ `Private` **redeemTicketsInChannel**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | [`ChannelEntry`](ChannelEntry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/index.ts:239](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L239)

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`default`](default.md)

Removes all listeners, or those of the specified `eventName`.

It is bad practice to remove listeners added elsewhere in the code,
particularly when the `EventEmitter` instance was created by some other
component or module (e.g. sockets or file streams).

Returns a reference to the `EventEmitter`, so that calls can be chained.

**`since`** v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

node_modules/@types/node/events.d.ts:455

___

### removeListener

▸ **removeListener**(`eventName`, `listener`): [`default`](default.md)

Removes the specified `listener` from the listener array for the event named`eventName`.

```js
const callback = (stream) => {
  console.log('someone connected!');
};
server.on('connection', callback);
// ...
server.removeListener('connection', callback);
```

`removeListener()` will remove, at most, one instance of a listener from the
listener array. If any single listener has been added multiple times to the
listener array for the specified `eventName`, then `removeListener()` must be
called multiple times to remove each instance.

Once an event is emitted, all listeners attached to it at the
time of emitting are called in order. This implies that any`removeListener()` or `removeAllListeners()` calls _after_ emitting and_before_ the last listener finishes execution will
not remove them from`emit()` in progress. Subsequent events behave as expected.

```js
const myEmitter = new MyEmitter();

const callbackA = () => {
  console.log('A');
  myEmitter.removeListener('event', callbackB);
};

const callbackB = () => {
  console.log('B');
};

myEmitter.on('event', callbackA);

myEmitter.on('event', callbackB);

// callbackA removes listener callbackB but it will still be called.
// Internal listener array at time of emit [callbackA, callbackB]
myEmitter.emit('event');
// Prints:
//   A
//   B

// callbackB is now removed.
// Internal listener array [callbackA]
myEmitter.emit('event');
// Prints:
//   A
```

Because listeners are managed using an internal array, calling this will
change the position indices of any listener registered _after_ the listener
being removed. This will not impact the order in which listeners are called,
but it means that any copies of the listener array as returned by
the `emitter.listeners()` method will need to be recreated.

When a single function has been added as a handler multiple times for a single
event (as in the example below), `removeListener()` will remove the most
recently added instance. In the example the `once('ping')`listener is removed:

```js
const ee = new EventEmitter();

function pong() {
  console.log('pong');
}

ee.on('ping', pong);
ee.once('ping', pong);
ee.removeListener('ping', pong);

ee.emit('ping');
ee.emit('ping');
```

Returns a reference to the `EventEmitter`, so that calls can be chained.

**`since`** v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

node_modules/@types/node/events.d.ts:439

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`default`](default.md)

By default `EventEmitter`s will print a warning if more than `10` listeners are
added for a particular event. This is a useful default that helps finding
memory leaks. The `emitter.setMaxListeners()` method allows the limit to be
modified for this specific `EventEmitter` instance. The value can be set to`Infinity` (or `0`) to indicate an unlimited number of listeners.

Returns a reference to the `EventEmitter`, so that calls can be chained.

**`since`** v0.3.5

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:465

___

### setTxHandler

▸ **setTxHandler**(`evt`, `tx`): `DeferType`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `evt` | `IndexerEvents` |
| `tx` | `string` |

#### Returns

`DeferType`<`string`\>

#### Defined in

[packages/core-ethereum/src/index.ts:142](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L142)

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

[packages/core-ethereum/src/index.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L192)

___

### start

▸ **start**(): `Promise`<[`default`](default.md)\>

#### Returns

`Promise`<[`default`](default.md)\>

#### Defined in

[packages/core-ethereum/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L95)

___

### stop

▸ **stop**(): `Promise`<`void`\>

Stops the connector.

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/index.ts:126](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L126)

___

### uncachedGetBalance

▸ `Private` **uncachedGetBalance**(): `Promise`<`Balance`\>

#### Returns

`Promise`<`Balance`\>

#### Defined in

[packages/core-ethereum/src/index.ts:162](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L162)

___

### uncachedGetNativeBalance

▸ `Private` **uncachedGetNativeBalance**(): `Promise`<`NativeBalance`\>

Retrieves ETH balance, optionally uses the cache.

#### Returns

`Promise`<`NativeBalance`\>

ETH balance

#### Defined in

[packages/core-ethereum/src/index.ts:180](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L180)

___

### waitForPublicNodes

▸ **waitForPublicNodes**(): `Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Returns

`Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Defined in

[packages/core-ethereum/src/index.ts:201](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L201)

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

[packages/core-ethereum/src/index.ts:135](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L135)

___

### getEventListeners

▸ `Static` **getEventListeners**(`emitter`, `name`): `Function`[]

Returns a copy of the array of listeners for the event named `eventName`.

For `EventEmitter`s this behaves exactly the same as calling `.listeners` on
the emitter.

For `EventTarget`s this is the only way to get the event listeners for the
event target. This is useful for debugging and diagnostic purposes.

```js
const { getEventListeners, EventEmitter } = require('events');

{
  const ee = new EventEmitter();
  const listener = () => console.log('Events are fun');
  ee.on('foo', listener);
  getEventListeners(ee, 'foo'); // [listener]
}
{
  const et = new EventTarget();
  const listener = () => console.log('Events are fun');
  et.addEventListener('foo', listener);
  getEventListeners(et, 'foo'); // [listener]
}
```

**`since`** v15.2.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` \| `EventEmitter` |
| `name` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.getEventListeners

#### Defined in

node_modules/@types/node/events.d.ts:262

___

### listenerCount

▸ `Static` **listenerCount**(`emitter`, `eventName`): `number`

A class method that returns the number of listeners for the given `eventName`registered on the given `emitter`.

```js
const { EventEmitter, listenerCount } = require('events');
const myEmitter = new EventEmitter();
myEmitter.on('event', () => {});
myEmitter.on('event', () => {});
console.log(listenerCount(myEmitter, 'event'));
// Prints: 2
```

**`since`** v0.9.12

**`deprecated`** Since v3.2.0 - Use `listenerCount` instead.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `emitter` | `EventEmitter` | The emitter to query |
| `eventName` | `string` \| `symbol` | The event name |

#### Returns

`number`

#### Inherited from

EventEmitter.listenerCount

#### Defined in

node_modules/@types/node/events.d.ts:234

___

### on

▸ `Static` **on**(`emitter`, `eventName`, `options?`): `AsyncIterableIterator`<`any`\>

```js
const { on, EventEmitter } = require('events');

(async () => {
  const ee = new EventEmitter();

  // Emit later on
  process.nextTick(() => {
    ee.emit('foo', 'bar');
    ee.emit('foo', 42);
  });

  for await (const event of on(ee, 'foo')) {
    // The execution of this inner block is synchronous and it
    // processes one event at a time (even with await). Do not use
    // if concurrent execution is required.
    console.log(event); // prints ['bar'] [42]
  }
  // Unreachable here
})();
```

Returns an `AsyncIterator` that iterates `eventName` events. It will throw
if the `EventEmitter` emits `'error'`. It removes all listeners when
exiting the loop. The `value` returned by each iteration is an array
composed of the emitted event arguments.

An `AbortSignal` can be used to cancel waiting on events:

```js
const { on, EventEmitter } = require('events');
const ac = new AbortController();

(async () => {
  const ee = new EventEmitter();

  // Emit later on
  process.nextTick(() => {
    ee.emit('foo', 'bar');
    ee.emit('foo', 42);
  });

  for await (const event of on(ee, 'foo', { signal: ac.signal })) {
    // The execution of this inner block is synchronous and it
    // processes one event at a time (even with await). Do not use
    // if concurrent execution is required.
    console.log(event); // prints ['bar'] [42]
  }
  // Unreachable here
})();

process.nextTick(() => ac.abort());
```

**`since`** v13.6.0, v12.16.0

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `emitter` | `EventEmitter` | - |
| `eventName` | `string` | The name of the event being listened for |
| `options?` | `StaticEventEmitterOptions` | - |

#### Returns

`AsyncIterableIterator`<`any`\>

that iterates `eventName` events emitted by the `emitter`

#### Inherited from

EventEmitter.on

#### Defined in

node_modules/@types/node/events.d.ts:217

___

### once

▸ `Static` **once**(`emitter`, `eventName`, `options?`): `Promise`<`any`[]\>

Creates a `Promise` that is fulfilled when the `EventEmitter` emits the given
event or that is rejected if the `EventEmitter` emits `'error'` while waiting.
The `Promise` will resolve with an array of all the arguments emitted to the
given event.

This method is intentionally generic and works with the web platform [EventTarget](https://dom.spec.whatwg.org/#interface-eventtarget) interface, which has no special`'error'` event
semantics and does not listen to the `'error'` event.

```js
const { once, EventEmitter } = require('events');

async function run() {
  const ee = new EventEmitter();

  process.nextTick(() => {
    ee.emit('myevent', 42);
  });

  const [value] = await once(ee, 'myevent');
  console.log(value);

  const err = new Error('kaboom');
  process.nextTick(() => {
    ee.emit('error', err);
  });

  try {
    await once(ee, 'myevent');
  } catch (err) {
    console.log('error happened', err);
  }
}

run();
```

The special handling of the `'error'` event is only used when `events.once()`is used to wait for another event. If `events.once()` is used to wait for the
'`error'` event itself, then it is treated as any other kind of event without
special handling:

```js
const { EventEmitter, once } = require('events');

const ee = new EventEmitter();

once(ee, 'error')
  .then(([err]) => console.log('ok', err.message))
  .catch((err) => console.log('error', err.message));

ee.emit('error', new Error('boom'));

// Prints: ok boom
```

An `AbortSignal` can be used to cancel waiting for the event:

```js
const { EventEmitter, once } = require('events');

const ee = new EventEmitter();
const ac = new AbortController();

async function foo(emitter, event, signal) {
  try {
    await once(emitter, event, { signal });
    console.log('event emitted!');
  } catch (error) {
    if (error.name === 'AbortError') {
      console.error('Waiting for the event was canceled!');
    } else {
      console.error('There was an error', error.message);
    }
  }
}

foo(ee, 'foo', ac.signal);
ac.abort(); // Abort waiting for the event
ee.emit('foo'); // Prints: Waiting for the event was canceled!
```

**`since`** v11.13.0, v10.16.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `NodeEventTarget` |
| `eventName` | `string` \| `symbol` |
| `options?` | `StaticEventEmitterOptions` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

node_modules/@types/node/events.d.ts:157

▸ `Static` **once**(`emitter`, `eventName`, `options?`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` |
| `eventName` | `string` |
| `options?` | `StaticEventEmitterOptions` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

node_modules/@types/node/events.d.ts:158
