[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Indexer

# Class: Indexer

Indexes HoprChannels smart contract and stores to the DB,
all channels in the network.
Also keeps track of the latest block number.

## Hierarchy

- `EventEmitter`

  ↳ **`Indexer`**

## Table of contents

### Constructors

- [constructor](Indexer.md#constructor)

### Properties

- [chain](Indexer.md#chain)
- [genesisBlock](Indexer.md#genesisblock)
- [lastSnapshot](Indexer.md#lastsnapshot)
- [latestBlock](Indexer.md#latestblock)
- [status](Indexer.md#status)
- [unconfirmedEvents](Indexer.md#unconfirmedevents)
- [unsubscribeBlock](Indexer.md#unsubscribeblock)
- [unsubscribeErrors](Indexer.md#unsubscribeerrors)
- [captureRejectionSymbol](Indexer.md#capturerejectionsymbol)
- [captureRejections](Indexer.md#capturerejections)
- [defaultMaxListeners](Indexer.md#defaultmaxlisteners)
- [errorMonitor](Indexer.md#errormonitor)

### Methods

- [addListener](Indexer.md#addlistener)
- [emit](Indexer.md#emit)
- [eventNames](Indexer.md#eventnames)
- [getAccount](Indexer.md#getaccount)
- [getAnnouncedAddresses](Indexer.md#getannouncedaddresses)
- [getEvents](Indexer.md#getevents)
- [getMaxListeners](Indexer.md#getmaxlisteners)
- [getOpenChannelsFrom](Indexer.md#getopenchannelsfrom)
- [getPublicKeyOf](Indexer.md#getpublickeyof)
- [getPublicNodes](Indexer.md#getpublicnodes)
- [getRandomOpenChannel](Indexer.md#getrandomopenchannel)
- [indexEvent](Indexer.md#indexevent)
- [listenerCount](Indexer.md#listenercount)
- [listeners](Indexer.md#listeners)
- [off](Indexer.md#off)
- [on](Indexer.md#on)
- [onAnnouncement](Indexer.md#onannouncement)
- [onChannelClosed](Indexer.md#onchannelclosed)
- [onChannelUpdated](Indexer.md#onchannelupdated)
- [onNewBlock](Indexer.md#onnewblock)
- [onNewEvents](Indexer.md#onnewevents)
- [onProviderError](Indexer.md#onprovidererror)
- [onTicketRedeemed](Indexer.md#onticketredeemed)
- [onTransfer](Indexer.md#ontransfer)
- [once](Indexer.md#once)
- [prependListener](Indexer.md#prependlistener)
- [prependOnceListener](Indexer.md#prependoncelistener)
- [processPastEvents](Indexer.md#processpastevents)
- [processUnconfirmedEvents](Indexer.md#processunconfirmedevents)
- [rawListeners](Indexer.md#rawlisteners)
- [removeAllListeners](Indexer.md#removealllisteners)
- [removeListener](Indexer.md#removelistener)
- [resolvePendingTransaction](Indexer.md#resolvependingtransaction)
- [restart](Indexer.md#restart)
- [setMaxListeners](Indexer.md#setmaxlisteners)
- [start](Indexer.md#start)
- [stop](Indexer.md#stop)
- [getEventListeners](Indexer.md#geteventlisteners)
- [listenerCount](Indexer.md#listenercount)
- [on](Indexer.md#on)
- [once](Indexer.md#once)

## Constructors

### constructor

• **new Indexer**(`address`, `db`, `maxConfirmations`, `blockRange`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |
| `db` | `HoprDB` |
| `maxConfirmations` | `number` |
| `blockRange` | `number` |

#### Overrides

EventEmitter.constructor

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L55)

## Properties

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
| `getToken` | () => `HoprToken` |
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

[packages/core-ethereum/src/indexer/index.ts:48](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L48)

___

### genesisBlock

• `Private` **genesisBlock**: `number`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L49)

___

### lastSnapshot

• `Private` **lastSnapshot**: `IndexerSnapshot`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L50)

___

### latestBlock

• **latestBlock**: `number` = `0`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L46)

___

### status

• **status**: ``"started"`` \| ``"restarting"`` \| ``"stopped"`` = `'stopped'`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L45)

___

### unconfirmedEvents

• `Private` **unconfirmedEvents**: `FIFO`<`TypedEvent`<`any`, `any`\>\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L47)

___

### unsubscribeBlock

• `Private` **unsubscribeBlock**: () => `void`

#### Type declaration

▸ (): `void`

##### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:53](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L53)

___

### unsubscribeErrors

• `Private` **unsubscribeErrors**: () => `void`

#### Type declaration

▸ (): `void`

##### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L52)

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

▸ **addListener**(`eventName`, `listener`): [`Indexer`](Indexer.md)

Alias for `emitter.on(eventName, listener)`.

**`since`** v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

node_modules/@types/node/events.d.ts:299

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

### getAccount

▸ **getAccount**(`address`): `Promise`<`AccountEntry`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<`AccountEntry`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:706](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L706)

___

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(): `Promise`<`Multiaddr`[]\>

#### Returns

`Promise`<`Multiaddr`[]\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:718](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L718)

___

### getEvents

▸ `Private` **getEvents**(`fromBlock`, `toBlock`, `withTokenTransactions`): `Promise`<`TypedEvent`<`any`, `any`\>[]\>

Gets all interesting on-chain events, such as Transfer events and payment
channel events

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `fromBlock` | `number` | block to start from |
| `toBlock` | `number` | last block (inclusive) to consider |
| `withTokenTransactions` | `boolean` | if true, also query for token transfer towards or from the node towards someone else |

#### Returns

`Promise`<`TypedEvent`<`any`, `any`\>[]\>

all relevant events in the specified block range

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:195](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L195)

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

Returns the current max listener value for the `EventEmitter` which is either
set by `emitter.setMaxListeners(n)` or defaults to [defaultMaxListeners](Indexer.md#defaultmaxlisteners).

**`since`** v1.0.0

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:471

___

### getOpenChannelsFrom

▸ **getOpenChannelsFrom**(`source`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

Returns peer's open channels.
NOTE: channels with status 'PENDING_TO_CLOSE' are not included

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `source` | `PublicKey` | peer |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

peer's open channels

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:761](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L761)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`address`): `Promise`<`PublicKey`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<`PublicKey`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:710](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L710)

___

### getPublicNodes

▸ **getPublicNodes**(): `Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Returns

`Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:722](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L722)

___

### getRandomOpenChannel

▸ **getRandomOpenChannel**(): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

Returns a random open channel.
NOTE: channels with status 'PENDING_TO_CLOSE' are not included

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

an open channel

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:744](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L744)

___

### indexEvent

▸ `Private` **indexEvent**(`indexerEvent`, `txHash`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `indexerEvent` | `IndexerEvents` |
| `txHash` | `string`[] |

#### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:702](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L702)

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

▸ **off**(`eventName`, `listener`): [`Indexer`](Indexer.md)

Alias for `emitter.removeListener()`.

**`since`** v10.0.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.off

#### Defined in

node_modules/@types/node/events.d.ts:444

___

### on

▸ **on**(`eventName`, `listener`): [`Indexer`](Indexer.md)

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

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.on

#### Defined in

node_modules/@types/node/events.d.ts:330

___

### onAnnouncement

▸ `Private` **onAnnouncement**(`event`, `blockNumber`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `AnnouncementEvent` |
| `blockNumber` | `BN` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:588](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L588)

___

### onChannelClosed

▸ `Private` **onChannelClosed**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | [`ChannelEntry`](ChannelEntry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:683](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L683)

___

### onChannelUpdated

▸ `Private` **onChannelUpdated**(`event`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdatedEvent` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:615](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L615)

___

### onNewBlock

▸ `Private` **onNewBlock**(`blockNumber`, `fetchEvents?`): `Promise`<`void`\>

Called whenever a new block found.
This will update `this.latestBlock`,
and processes events which are within
confirmed blocks.

#### Parameters

| Name | Type | Default value | Description |
| :------ | :------ | :------ | :------ |
| `blockNumber` | `number` | `undefined` | latest on-chain block number |
| `fetchEvents` | `boolean` | `false` | if true, query provider for events in block |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:378](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L378)

___

### onNewEvents

▸ `Private` **onNewEvents**(`events`): `void`

Adds new events to the queue of unprocessed events

**`dev`** ignores events that have been processed before.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `events` | `TypedEvent`<`any`, `any`\>[] | new unprocessed events |

#### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:428](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L428)

___

### onProviderError

▸ `Private` **onProviderError**(`error`): `Promise`<`void`\>

Called whenever there was a provider error.
Will restart the indexer if needed.

#### Parameters

| Name | Type |
| :------ | :------ |
| `error` | `any` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:340](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L340)

___

### onTicketRedeemed

▸ `Private` **onTicketRedeemed**(`event`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `TicketRedeemedEvent` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:654](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L654)

___

### onTransfer

▸ `Private` **onTransfer**(`event`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `TransferEvent` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:688](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L688)

___

### once

▸ **once**(`eventName`, `listener`): [`Indexer`](Indexer.md)

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

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.once

#### Defined in

node_modules/@types/node/events.d.ts:359

___

### prependListener

▸ **prependListener**(`eventName`, `listener`): [`Indexer`](Indexer.md)

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

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

node_modules/@types/node/events.d.ts:579

___

### prependOnceListener

▸ **prependOnceListener**(`eventName`, `listener`): [`Indexer`](Indexer.md)

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

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.prependOnceListener

#### Defined in

node_modules/@types/node/events.d.ts:595

___

### processPastEvents

▸ `Private` **processPastEvents**(`fromBlock`, `maxToBlock`, `maxBlockRange`): `Promise`<`number`\>

Query past events, this will loop until it gets all blocks from `toBlock` to `fromBlock`.
If we exceed response pull limit, we switch into quering smaller chunks.
TODO: optimize DB and fetch requests

#### Parameters

| Name | Type |
| :------ | :------ |
| `fromBlock` | `number` |
| `maxToBlock` | `number` |
| `maxBlockRange` | `number` |

#### Returns

`Promise`<`number`\>

past events and last queried block

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:292](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L292)

___

### processUnconfirmedEvents

▸ **processUnconfirmedEvents**(`blockNumber`, `lastDatabaseSnapshot`): `Promise`<`void`\>

Process all stored but not yet processed events up to latest
confirmed block (latestBlock - confirmationTime)

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `blockNumber` | `number` | latest on-chain block number |
| `lastDatabaseSnapshot` | `Snapshot` | latest snapshot in database |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:484](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L484)

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

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`Indexer`](Indexer.md)

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

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

node_modules/@types/node/events.d.ts:455

___

### removeListener

▸ **removeListener**(`eventName`, `listener`): [`Indexer`](Indexer.md)

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

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

node_modules/@types/node/events.d.ts:439

___

### resolvePendingTransaction

▸ **resolvePendingTransaction**(`eventType`, `tx`): `DeferType`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventType` | `IndexerEvents` |
| `tx` | `string` |

#### Returns

`DeferType`<`string`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:767](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L767)

___

### restart

▸ `Protected` **restart**(): `Promise`<`void`\>

Restarts the indexer

#### Returns

`Promise`<`void`\>

a promise that resolves once the indexer
has been restarted

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:166](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L166)

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`Indexer`](Indexer.md)

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

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:465

___

### start

▸ **start**(`chain`, `genesisBlock`): `Promise`<`void`\>

Starts indexing.

#### Parameters

| Name | Type |
| :------ | :------ |
| `chain` | `Object` |
| `chain.announce` | (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.finalizeChannelClosure` | (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.getAllQueuingTransactionRequests` | () => `TransactionRequest`[] |
| `chain.getBalance` | (`address`: `Address`) => `Promise`<`Balance`\> |
| `chain.getChannels` | () => `HoprChannels` |
| `chain.getGenesisBlock` | () => `number` |
| `chain.getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } |
| `chain.getLatestBlockNumber` | `any` |
| `chain.getNativeBalance` | (`address`: `Address`) => `Promise`<`NativeBalance`\> |
| `chain.getNativeTokenTransactionInBlock` | (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getToken` | () => `HoprToken` |
| `chain.getWallet` | () => `Wallet` |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.subscribeBlock` | (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` |
| `chain.subscribeChannelEvents` | (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` |
| `chain.subscribeError` | (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` |
| `chain.subscribeTokenEvents` | (`cb`: (`event`: `TypedEvent`<`any`, `any`\>) => `void` \| `Promise`<`void`\>) => () => `void` |
| `chain.unsubscribe` | () => `void` |
| `chain.updateConfirmedTransaction` | (`hash`: `string`) => `void` |
| `chain.waitUntilReady` | () => `Promise`<`Network`\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `genesisBlock` | `number` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:69](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L69)

___

### stop

▸ **stop**(): `void`

Stops indexing.

#### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:146](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L146)

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
