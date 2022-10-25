[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / default

# Class: default

## Hierarchy

- `EventEmitter`

  ↳ **`default`**

## Table of contents

### Constructors

- [constructor](default.md#constructor)

### Properties

- [acknowledgements](default.md#acknowledgements)
- [environment](default.md#environment)
- [forward](default.md#forward)
- [heartbeat](default.md#heartbeat)
- [indexer](default.md#indexer)
- [knownPublicNodesCache](default.md#knownpublicnodescache)
- [libp2pComponents](default.md#libp2pcomponents)
- [networkPeers](default.md#networkpeers)
- [pubKey](default.md#pubkey)
- [status](default.md#status)
- [stopLibp2p](default.md#stoplibp2p)
- [stopPeriodicCheck](default.md#stopperiodiccheck)
- [strategy](default.md#strategy)
- [captureRejectionSymbol](default.md#capturerejectionsymbol)
- [captureRejections](default.md#capturerejections)
- [defaultMaxListeners](default.md#defaultmaxlisteners)
- [errorMonitor](default.md#errormonitor)

### Methods

- [addListener](default.md#addlistener)
- [announce](default.md#announce)
- [closeChannel](default.md#closechannel)
- [closeConnectionsTo](default.md#closeconnectionsto)
- [connectionReport](default.md#connectionreport)
- [emit](default.md#emit)
- [emitOnConnector](default.md#emitonconnector)
- [eventNames](default.md#eventnames)
- [fundChannel](default.md#fundchannel)
- [getAddressesAnnouncedOnChain](default.md#getaddressesannouncedonchain)
- [getAddressesAnnouncedToDHT](default.md#getaddressesannouncedtodht)
- [getAllChannels](default.md#getallchannels)
- [getAllTickets](default.md#getalltickets)
- [getBalance](default.md#getbalance)
- [getChannel](default.md#getchannel)
- [getChannelStrategy](default.md#getchannelstrategy)
- [getChannelsFrom](default.md#getchannelsfrom)
- [getChannelsTo](default.md#getchannelsto)
- [getConnectedPeers](default.md#getconnectedpeers)
- [getConnectionInfo](default.md#getconnectioninfo)
- [getConnectivityHealth](default.md#getconnectivityhealth)
- [getEntryNodes](default.md#getentrynodes)
- [getEthereumAddress](default.md#getethereumaddress)
- [getId](default.md#getid)
- [getIntermediateNodes](default.md#getintermediatenodes)
- [getListeningAddresses](default.md#getlisteningaddresses)
- [getMaxListeners](default.md#getmaxlisteners)
- [getNativeBalance](default.md#getnativebalance)
- [getObservedAddresses](default.md#getobservedaddresses)
- [getPublicKeyOf](default.md#getpublickeyof)
- [getTicketStatistics](default.md#getticketstatistics)
- [getTickets](default.md#gettickets)
- [getVersion](default.md#getversion)
- [isAllowedAccessToNetwork](default.md#isallowedaccesstonetwork)
- [listenerCount](default.md#listenercount)
- [listeners](default.md#listeners)
- [maybeEmitFundsEmptyEvent](default.md#maybeemitfundsemptyevent)
- [maybeLogProfilingToGCloud](default.md#maybelogprofilingtogcloud)
- [off](default.md#off)
- [on](default.md#on)
- [onChannelWaitingForCommitment](default.md#onchannelwaitingforcommitment)
- [onOwnChannelUpdated](default.md#onownchannelupdated)
- [onPeerAnnouncement](default.md#onpeerannouncement)
- [once](default.md#once)
- [openChannel](default.md#openchannel)
- [ping](default.md#ping)
- [prependListener](default.md#prependlistener)
- [prependOnceListener](default.md#prependoncelistener)
- [rawListeners](default.md#rawlisteners)
- [redeemAllTickets](default.md#redeemalltickets)
- [redeemTicketsInChannel](default.md#redeemticketsinchannel)
- [removeAllListeners](default.md#removealllisteners)
- [removeListener](default.md#removelistener)
- [sendMessage](default.md#sendmessage)
- [setChannelStrategy](default.md#setchannelstrategy)
- [setMaxListeners](default.md#setmaxlisteners)
- [signMessage](default.md#signmessage)
- [smartContractInfo](default.md#smartcontractinfo)
- [start](default.md#start)
- [startPeriodicStrategyCheck](default.md#startperiodicstrategycheck)
- [stop](default.md#stop)
- [subscribeOnConnector](default.md#subscribeonconnector)
- [tickChannelStrategy](default.md#tickchannelstrategy)
- [validateIntermediatePath](default.md#validateintermediatepath)
- [waitForFunds](default.md#waitforfunds)
- [waitForRunning](default.md#waitforrunning)
- [withdraw](default.md#withdraw)
- [getEventListeners](default.md#geteventlisteners)
- [listenerCount](default.md#listenercount-1)
- [on](default.md#on-1)
- [once](default.md#once-1)

## Constructors

### constructor

• **new default**(`id`, `db`, `connector`, `options`, `publicNodesEmitter?`)

Create an uninitialized Hopr Node

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `id` | `PeerId` | PeerId to use, determines node address |
| `db` | `HoprDB` | used to persist protocol state |
| `connector` | `default` | an instance of the blockchain wrapper |
| `options` | [`HoprOptions`](../modules.md#hoproptions) |  |
| `publicNodesEmitter` | `PublicNodesEmitter` | used to pass information about newly announced nodes to transport module |

#### Overrides

EventEmitter.constructor

## Properties

### acknowledgements

• `Private` **acknowledgements**: `AcknowledgementInteraction`

#### Defined in

[packages/core/src/index.ts:195](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L195)

___

### environment

• **environment**: [`ResolvedEnvironment`](../modules.md#resolvedenvironment)

#### Defined in

[packages/core/src/index.ts:201](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L201)

___

### forward

• `Private` **forward**: `PacketForwardInteraction`

#### Defined in

[packages/core/src/index.ts:194](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L194)

___

### heartbeat

• `Private` **heartbeat**: `default`

#### Defined in

[packages/core/src/index.ts:193](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L193)

___

### indexer

• **indexer**: `Indexer`

#### Defined in

[packages/core/src/index.ts:203](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L203)

___

### knownPublicNodesCache

• `Private` **knownPublicNodesCache**: `Set`<`unknown`\>

#### Defined in

[packages/core/src/index.ts:199](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L199)

___

### libp2pComponents

• `Private` **libp2pComponents**: `Components`

#### Defined in

[packages/core/src/index.ts:196](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L196)

___

### networkPeers

• `Private` **networkPeers**: `NetworkPeers`

#### Defined in

[packages/core/src/index.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L192)

___

### pubKey

• `Private` **pubKey**: `PublicKey`

#### Defined in

[packages/core/src/index.ts:198](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L198)

___

### status

• **status**: [`NodeStatus`](../modules.md#nodestatus) = `'UNINITIALIZED'`

#### Defined in

[packages/core/src/index.ts:188](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L188)

___

### stopLibp2p

• `Private` **stopLibp2p**: () => `void` \| `Promise`<`void`\>

#### Type declaration

▸ (): `void` \| `Promise`<`void`\>

This method will be invoked to stop the component.

It should not assume any other components are running when it is called.

##### Returns

`void` \| `Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:197](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L197)

___

### stopPeriodicCheck

• `Private` **stopPeriodicCheck**: () => `void`

#### Type declaration

▸ (): `void`

##### Returns

`void`

#### Defined in

[packages/core/src/index.ts:190](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L190)

___

### strategy

• `Private` **strategy**: [`ChannelStrategyInterface`](../interfaces/ChannelStrategyInterface.md)

#### Defined in

[packages/core/src/index.ts:191](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L191)

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

___

### announce

▸ `Private` **announce**(`announceRoutableAddress?`): `Promise`<`void`\>

Announces address of node on-chain to be reachable by other nodes.

**`dev`** Promise resolves before own announcement appears in the indexer

#### Parameters

| Name | Type | Default value | Description |
| :------ | :------ | :------ | :------ |
| `announceRoutableAddress` | `boolean` | `false` | publish routable address if true |

#### Returns

`Promise`<`void`\>

a Promise that resolves once announce transaction has been published

___

### closeChannel

▸ **closeChannel**(`counterparty`, `direction`): `Promise`<{ `receipt`: `string` ; `status`: `ChannelStatus`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `PeerId` |
| `direction` | ``"incoming"`` \| ``"outgoing"`` |

#### Returns

`Promise`<{ `receipt`: `string` ; `status`: `ChannelStatus`  }\>

___

### closeConnectionsTo

▸ `Private` **closeConnectionsTo**(`peer`): `void`

Closes all open connections to a peer. Used to temporarily or permanently
disconnect from a peer.
Similar to `libp2p.hangUp` but catching all errors.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `PeerId` | PeerId of the peer from whom we want to disconnect |

#### Returns

`void`

___

### connectionReport

▸ **connectionReport**(): `Promise`<`string`\>

**`deprecated`** Used by API v1

#### Returns

`Promise`<`string`\>

a string describing the connection status between
us and various nodes

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

___

### emitOnConnector

▸ **emitOnConnector**(`event`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` |

#### Returns

`void`

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

___

### fundChannel

▸ **fundChannel**(`counterparty`, `myFund`, `counterpartyFund`): `Promise`<`void`\>

Fund a payment channel

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `counterparty` | `PeerId` | the counter party's peerId |
| `myFund` | `BN` | the amount to fund the channel in my favor HOPR(wei) |
| `counterpartyFund` | `BN` | the amount to fund the channel in counterparty's favor HOPR(wei) |

#### Returns

`Promise`<`void`\>

___

### getAddressesAnnouncedOnChain

▸ **getAddressesAnnouncedOnChain**(): `AsyncGenerator`<`Multiaddr`, `void`, `unknown`\>

Takes a look into the indexer.

#### Returns

`AsyncGenerator`<`Multiaddr`, `void`, `unknown`\>

a list of announced multi addresses

___

### getAddressesAnnouncedToDHT

▸ **getAddressesAnnouncedToDHT**(`peer?`, `_timeout?`): `Promise`<`Multiaddr`[]\>

List of addresses that is announced to other nodes

**`dev`** returned list can change at runtime

#### Parameters

| Name | Type | Default value | Description |
| :------ | :------ | :------ | :------ |
| `peer` | `PeerId` | `undefined` | peer to query for, default self |
| `_timeout` | `number` | `5e3` | [optional] custom timeout for DHT query |

#### Returns

`Promise`<`Multiaddr`[]\>

___

### getAllChannels

▸ `Private` **getAllChannels**(): `AsyncIterable`<`ChannelEntry`\>

#### Returns

`AsyncIterable`<`ChannelEntry`\>

___

### getAllTickets

▸ **getAllTickets**(): `Promise`<`Ticket`[]\>

#### Returns

`Promise`<`Ticket`[]\>

___

### getBalance

▸ **getBalance**(): `Promise`<`Balance`\>

#### Returns

`Promise`<`Balance`\>

___

### getChannel

▸ **getChannel**(`src`, `dest`): `Promise`<`ChannelEntry`\>

Get the channel entry between source and destination node.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `src` | `PeerId` | PeerId |
| `dest` | `PeerId` | PeerId |

#### Returns

`Promise`<`ChannelEntry`\>

the channel entry of those two nodes

___

### getChannelStrategy

▸ **getChannelStrategy**(): `string`

#### Returns

`string`

___

### getChannelsFrom

▸ **getChannelsFrom**(`addr`): `Promise`<`ChannelEntry`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`ChannelEntry`[]\>

___

### getChannelsTo

▸ **getChannelsTo**(`addr`): `Promise`<`ChannelEntry`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`ChannelEntry`[]\>

___

### getConnectedPeers

▸ **getConnectedPeers**(): `Iterable`<`PeerId`\>

#### Returns

`Iterable`<`PeerId`\>

a list connected peerIds

___

### getConnectionInfo

▸ **getConnectionInfo**(`peerId`): `Entry`

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peerId` | `PeerId` | of the node we want to get the connection info for |

#### Returns

`Entry`

various information about the connection

___

### getConnectivityHealth

▸ **getConnectivityHealth**(): [`NetworkHealthIndicator`](../enums/NetworkHealthIndicator.md)

Recalculates and retrieves the current connectivity health indicator.

#### Returns

[`NetworkHealthIndicator`](../enums/NetworkHealthIndicator.md)

___

### getEntryNodes

▸ **getEntryNodes**(): `Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Returns

`Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

___

### getEthereumAddress

▸ **getEthereumAddress**(): `Address`

#### Returns

`Address`

___

### getId

▸ **getId**(): `PeerId`

Gets the peer ID of this HOPR node.

#### Returns

`PeerId`

___

### getIntermediateNodes

▸ `Private` **getIntermediateNodes**(`destination`): `Promise`<`PublicKey`[]\>

Takes a destination and samples randomly intermediate nodes
that will relay that message before it reaches its destination.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `destination` | `PublicKey` | instance of peerInfo that contains the peerId of the destination |

#### Returns

`Promise`<`PublicKey`[]\>

___

### getListeningAddresses

▸ **getListeningAddresses**(): `Multiaddr`[]

List the addresses on which the node is listening

#### Returns

`Multiaddr`[]

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

___

### getNativeBalance

▸ **getNativeBalance**(): `Promise`<`NativeBalance`\>

#### Returns

`Promise`<`NativeBalance`\>

___

### getObservedAddresses

▸ **getObservedAddresses**(`peer`): `Promise`<`Multiaddr`[]\>

Gets the observed addresses of a given peer.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `PeerId` | peer to query for |

#### Returns

`Promise`<`Multiaddr`[]\>

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`): `Promise`<`PublicKey`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`PublicKey`\>

___

### getTicketStatistics

▸ **getTicketStatistics**(): `Promise`<{ `losing`: `number` ; `neglected`: `number` ; `pending`: `number` ; `redeemed`: `number` ; `redeemedValue`: `Balance` ; `rejected`: `number` ; `rejectedValue`: `Balance` ; `unredeemed`: `number` = ack.length; `unredeemedValue`: `Balance` ; `winProportion`: `number`  }\>

#### Returns

`Promise`<{ `losing`: `number` ; `neglected`: `number` ; `pending`: `number` ; `redeemed`: `number` ; `redeemedValue`: `Balance` ; `rejected`: `number` ; `rejectedValue`: `Balance` ; `unredeemed`: `number` = ack.length; `unredeemedValue`: `Balance` ; `winProportion`: `number`  }\>

___

### getTickets

▸ **getTickets**(`peerId`): `Promise`<`Ticket`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

`Promise`<`Ticket`[]\>

___

### getVersion

▸ **getVersion**(): `any`

Returns the version of hopr-core.

#### Returns

`any`

___

### isAllowedAccessToNetwork

▸ **isAllowedAccessToNetwork**(`id`): `Promise`<`boolean`\>

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `id` | `PeerId` | the peer id of the account we want to check if it's allowed access to the network |

#### Returns

`Promise`<`boolean`\>

true if allowed access

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

___

### maybeEmitFundsEmptyEvent

▸ `Private` **maybeEmitFundsEmptyEvent**(`error`): `void`

If error provided is considered an out of funds error
- it will emit that the node is out of funds

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `error` | `any` | error thrown by an ethereum transaction |

#### Returns

`void`

___

### maybeLogProfilingToGCloud

▸ `Private` **maybeLogProfilingToGCloud**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

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

___

### onChannelWaitingForCommitment

▸ `Private` **onChannelWaitingForCommitment**(`c`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `ChannelEntry` |

#### Returns

`Promise`<`void`\>

___

### onOwnChannelUpdated

▸ `Private` **onOwnChannelUpdated**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | `ChannelEntry` |

#### Returns

`Promise`<`void`\>

___

### onPeerAnnouncement

▸ `Private` **onPeerAnnouncement**(`peer`): `Promise`<`void`\>

Called whenever a peer is announced

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `Object` | newly announced peer |
| `peer.id` | `PeerId` | - |
| `peer.multiaddrs` | `Multiaddr`[] | - |

#### Returns

`Promise`<`void`\>

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

___

### openChannel

▸ **openChannel**(`counterparty`, `amountToFund`): `Promise`<{ `channelId`: `Hash` ; `receipt`: `string`  }\>

Open a payment channel

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `counterparty` | `PeerId` | the counterparty's peerId |
| `amountToFund` | `BN` | the amount to fund in HOPR(wei) |

#### Returns

`Promise`<{ `channelId`: `Hash` ; `receipt`: `string`  }\>

___

### ping

▸ **ping**(`destination`): `Promise`<{ `info?`: `string` ; `latency`: `number`  }\>

Ping a node.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `destination` | `PeerId` | PeerId of the node |

#### Returns

`Promise`<{ `info?`: `string` ; `latency`: `number`  }\>

latency

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

___

### redeemAllTickets

▸ **redeemAllTickets**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

___

### redeemTicketsInChannel

▸ **redeemTicketsInChannel**(`peerId`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

`Promise`<`void`\>

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

___

### sendMessage

▸ **sendMessage**(`msg`, `destination`, `intermediatePath?`): `Promise`<`string`\>

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `msg` | `Uint8Array` | message to send |
| `destination` | `PeerId` | PeerId of the destination |
| `intermediatePath?` | `PublicKey`[] | optional set path manually |

#### Returns

`Promise`<`string`\>

___

### setChannelStrategy

▸ **setChannelStrategy**(`strategy`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `strategy` | [`ChannelStrategyInterface`](../interfaces/ChannelStrategyInterface.md) |

#### Returns

`void`

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

___

### signMessage

▸ **signMessage**(`message`): `Uint8Array`

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | `Uint8Array` |

#### Returns

`Uint8Array`

___

### smartContractInfo

▸ **smartContractInfo**(): `Object`

#### Returns

`Object`

| Name | Type |
| :------ | :------ |
| `channelClosureSecs` | `number` |
| `hoprChannelsAddress` | `string` |
| `hoprNetworkRegistryAddress` | `string` |
| `hoprTokenAddress` | `string` |
| `network` | `string` |

___

### start

▸ **start**(`__testingLibp2p?`): `Promise`<`void`\>

Start node

The node has a fairly complex lifecycle. This method should do all setup
required for a node to be functioning.

If the node is not funded, it will throw.

- Create a link to the ethereum blockchain
  - Finish indexing previous blocks [SLOW]
  - Find publicly accessible relays

- Start LibP2P and work out our network configuration.
  - Pass the list of relays from the indexer

- Wait for wallet to be funded with ETH [requires user interaction]

- Announce address, pubkey, and multiaddr on chain.

- Start heartbeat, automatic strategies, etc..

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `__testingLibp2p?` | `Libp2p` | use simulated libp2p instance for testing |

#### Returns

`Promise`<`void`\>

___

### startPeriodicStrategyCheck

▸ **startPeriodicStrategyCheck**(): `void`

#### Returns

`void`

___

### stop

▸ **stop**(): `Promise`<`void`\>

Shuts down the node and saves keys and peerBook in the database

#### Returns

`Promise`<`void`\>

___

### subscribeOnConnector

▸ **subscribeOnConnector**(`event`, `callback`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` |
| `callback` | () => `void` |

#### Returns

`void`

___

### tickChannelStrategy

▸ `Private` **tickChannelStrategy**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

___

### validateIntermediatePath

▸ `Private` **validateIntermediatePath**(`intermediatePath`): `Promise`<`void`\>

Validates the manual intermediate path by checking if it does not contain
channels that are not opened.
Throws an error if some channel is not opened.

#### Parameters

| Name | Type |
| :------ | :------ |
| `intermediatePath` | `PublicKey`[] |

#### Returns

`Promise`<`void`\>

___

### waitForFunds

▸ **waitForFunds**(): `Promise`<`void`\>

This is a utility method to wait until the node is funded.
A backoff is implemented, if node has not been funded and
MAX_DELAY is reached, this function will reject.

#### Returns

`Promise`<`void`\>

___

### waitForRunning

▸ **waitForRunning**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

___

### withdraw

▸ **withdraw**(`currency`, `recipient`, `amount`): `Promise`<`string`\>

Withdraw on-chain assets to a given address

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `currency` | ``"NATIVE"`` \| ``"HOPR"`` | either native currency or HOPR tokens |
| `recipient` | `string` | the account where the assets should be transferred to |
| `amount` | `string` | how many tokens to be transferred |

#### Returns

`Promise`<`string`\>

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
| `emitter` | `EventEmitter` \| `DOMEventTarget` |
| `name` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.getEventListeners

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
