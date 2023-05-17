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
- [db](default.md#db)
- [environment](default.md#environment)
- [forward](default.md#forward)
- [heartbeat](default.md#heartbeat)
- [id](default.md#id)
- [indexer](default.md#indexer)
- [knownPublicNodesCache](default.md#knownpublicnodescache)
- [libp2pComponents](default.md#libp2pcomponents)
- [networkPeers](default.md#networkpeers)
- [options](default.md#options)
- [pubKey](default.md#pubkey)
- [publicNodesEmitter](default.md#publicnodesemitter)
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
- [maybeEnableDhtServerMode](default.md#maybeenabledhtservermode)
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
- [printAvailableAddresses](default.md#printavailableaddresses)
- [printAvailableProtocols](default.md#printavailableprotocols)
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
- [strategyCloseChannel](default.md#strategyclosechannel)
- [strategyOpenChannel](default.md#strategyopenchannel)
- [subscribeOnConnector](default.md#subscribeonconnector)
- [tickChannelStrategy](default.md#tickchannelstrategy)
- [updateChannelMetrics](default.md#updatechannelmetrics)
- [validateIntermediatePath](default.md#validateintermediatepath)
- [waitForFunds](default.md#waitforfunds)
- [waitForRunning](default.md#waitforrunning)
- [withdraw](default.md#withdraw)
- [getEventListeners](default.md#geteventlisteners)
- [listenerCount](default.md#listenercount-1)
- [on](default.md#on-1)
- [once](default.md#once-1)
- [setMaxListeners](default.md#setmaxlisteners-1)

## Constructors

### constructor

• **new default**(`id`, `db`, `options`, `publicNodesEmitter?`)

Create an uninitialized Hopr Node

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `id` | `PeerId` | PeerId to use, determines node address |
| `db` | `HoprDB` | used to persist protocol state |
| `options` | [`HoprOptions`](../modules.md#hoproptions) |  |
| `publicNodesEmitter` | `PublicNodesEmitter` | used to pass information about newly announced nodes to transport module |

#### Overrides

EventEmitter.constructor

#### Defined in

[packages/core/src/index.ts:265](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L265)

## Properties

### acknowledgements

• `Private` **acknowledgements**: `AcknowledgementInteraction`

#### Defined in

[packages/core/src/index.ts:244](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L244)

___

### db

• **db**: `HoprDB`

used to persist protocol state

#### Defined in

[packages/core/src/index.ts:267](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L267)

___

### environment

• **environment**: [`ResolvedEnvironment`](ResolvedEnvironment.md)

#### Defined in

[packages/core/src/index.ts:250](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L250)

___

### forward

• `Private` **forward**: `PacketForwardInteraction`

#### Defined in

[packages/core/src/index.ts:243](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L243)

___

### heartbeat

• `Private` **heartbeat**: `default`

#### Defined in

[packages/core/src/index.ts:242](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L242)

___

### id

• `Private` **id**: `PeerId`

PeerId to use, determines node address

#### Defined in

[packages/core/src/index.ts:266](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L266)

___

### indexer

• **indexer**: `Indexer`

#### Defined in

[packages/core/src/index.ts:252](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L252)

___

### knownPublicNodesCache

• `Private` **knownPublicNodesCache**: `Set`<`unknown`\>

#### Defined in

[packages/core/src/index.ts:248](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L248)

___

### libp2pComponents

• `Private` **libp2pComponents**: `Components`

#### Defined in

[packages/core/src/index.ts:245](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L245)

___

### networkPeers

• `Private` **networkPeers**: `Network`

#### Defined in

[packages/core/src/index.ts:241](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L241)

___

### options

• `Private` **options**: [`HoprOptions`](../modules.md#hoproptions)

#### Defined in

[packages/core/src/index.ts:268](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L268)

___

### pubKey

• `Private` **pubKey**: `PublicKey`

#### Defined in

[packages/core/src/index.ts:247](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L247)

___

### publicNodesEmitter

• `Private` **publicNodesEmitter**: `PublicNodesEmitter`

used to pass information about newly announced nodes to transport module

#### Defined in

[packages/core/src/index.ts:269](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L269)

___

### status

• **status**: [`NodeStatus`](../modules.md#nodestatus) = `'UNINITIALIZED'`

#### Defined in

[packages/core/src/index.ts:237](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L237)

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

[packages/core/src/index.ts:246](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L246)

___

### stopPeriodicCheck

• `Private` **stopPeriodicCheck**: () => `void`

#### Type declaration

▸ (): `void`

##### Returns

`void`

#### Defined in

[packages/core/src/index.ts:239](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L239)

___

### strategy

• `Private` **strategy**: [`ChannelStrategyInterface`](../interfaces/ChannelStrategyInterface.md)

#### Defined in

[packages/core/src/index.ts:240](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L240)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [`captureRejectionSymbol`](default.md#capturerejectionsymbol)

#### Inherited from

EventEmitter.captureRejectionSymbol

#### Defined in

node_modules/@types/node/events.d.ts:291

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

node_modules/@types/node/events.d.ts:296

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:297

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

node_modules/@types/node/events.d.ts:290

## Methods

### addListener

▸ **addListener**(`eventName`, `listener`): [`default`](default.md)

Alias for `emitter.on(eventName, listener)`.

**`Since`**

v0.1.26

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

node_modules/@types/node/events.d.ts:317

___

### announce

▸ `Private` **announce**(`announceRoutableAddress?`): `Promise`<`void`\>

Announces address of node on-chain to be reachable by other nodes.

**`Dev`**

Promise resolves before own announcement appears in the indexer

#### Parameters

| Name | Type | Default value | Description |
| :------ | :------ | :------ | :------ |
| `announceRoutableAddress` | `boolean` | `false` | publish routable address if true |

#### Returns

`Promise`<`void`\>

a Promise that resolves once announce transaction has been published

#### Defined in

[packages/core/src/index.ts:1183](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1183)

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

#### Defined in

[packages/core/src/index.ts:1355](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1355)

___

### closeConnectionsTo

▸ `Private` **closeConnectionsTo**(`peer`): `Promise`<`void`\>

Closes all open connections to a peer. Used to temporarily or permanently
disconnect from a peer.
Similar to `libp2p.hangUp` but catching all errors.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `PeerId` | PeerId of the peer from whom we want to disconnect |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:1113](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1113)

___

### connectionReport

▸ **connectionReport**(): `Promise`<`string`\>

**`Deprecated`**

Used by API v1

#### Returns

`Promise`<`string`\>

a string describing the connection status between
us and various nodes

#### Defined in

[packages/core/src/index.ts:1128](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1128)

___

### emit

▸ **emit**(`eventName`, `...args`): `boolean`

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

**`Since`**

v0.1.26

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

node_modules/@types/node/events.d.ts:573

___

### emitOnConnector

▸ **emitOnConnector**(`event`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` |

#### Returns

`void`

#### Defined in

[packages/core/src/index.ts:1147](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1147)

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

**`Since`**

v6.0.0

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

node_modules/@types/node/events.d.ts:632

___

### fundChannel

▸ **fundChannel**(`counterparty`, `myFund`, `counterpartyFund`): `Promise`<`string`\>

Fund a payment channel

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `counterparty` | `PeerId` | the counter party's peerId |
| `myFund` | `BN` | the amount to fund the channel in my favor HOPR(wei) |
| `counterpartyFund` | `BN` | the amount to fund the channel in counterparty's favor HOPR(wei) |

#### Returns

`Promise`<`string`\>

#### Defined in

[packages/core/src/index.ts:1330](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1330)

___

### getAddressesAnnouncedOnChain

▸ **getAddressesAnnouncedOnChain**(): `AsyncGenerator`<`Multiaddr`, `void`, `unknown`\>

Takes a look into the indexer.

#### Returns

`AsyncGenerator`<`Multiaddr`, `void`, `unknown`\>

a list of announced multi addresses

#### Defined in

[packages/core/src/index.ts:1095](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1095)

___

### getAddressesAnnouncedToDHT

▸ **getAddressesAnnouncedToDHT**(`peer?`, `_timeout?`): `Promise`<`Multiaddr`[]\>

List of addresses that is announced to other nodes

**`Dev`**

returned list can change at runtime

#### Parameters

| Name | Type | Default value | Description |
| :------ | :------ | :------ | :------ |
| `peer` | `PeerId` | `undefined` | peer to query for, default self |
| `_timeout` | `number` | `5e3` | [optional] custom timeout for DHT query |

#### Returns

`Promise`<`Multiaddr`[]\>

#### Defined in

[packages/core/src/index.ts:892](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L892)

___

### getAllChannels

▸ **getAllChannels**(): `Promise`<`ChannelEntry`[]\>

#### Returns

`Promise`<`ChannelEntry`[]\>

#### Defined in

[packages/core/src/index.ts:1472](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1472)

___

### getAllTickets

▸ **getAllTickets**(): `Promise`<`Ticket`[]\>

#### Returns

`Promise`<`Ticket`[]\>

#### Defined in

[packages/core/src/index.ts:1415](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1415)

___

### getBalance

▸ **getBalance**(): `Promise`<`Balance`\>

#### Returns

`Promise`<`Balance`\>

#### Defined in

[packages/core/src/index.ts:1265](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1265)

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

#### Defined in

[packages/core/src/index.ts:1468](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1468)

___

### getChannelStrategy

▸ **getChannelStrategy**(): [`ChannelStrategyInterface`](../interfaces/ChannelStrategyInterface.md)

#### Returns

[`ChannelStrategyInterface`](../interfaces/ChannelStrategyInterface.md)

#### Defined in

[packages/core/src/index.ts:1261](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1261)

___

### getChannelsFrom

▸ **getChannelsFrom**(`addr`): `Promise`<`ChannelEntry`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`ChannelEntry`[]\>

#### Defined in

[packages/core/src/index.ts:1476](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1476)

___

### getChannelsTo

▸ **getChannelsTo**(`addr`): `Promise`<`ChannelEntry`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`ChannelEntry`[]\>

#### Defined in

[packages/core/src/index.ts:1480](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1480)

___

### getConnectedPeers

▸ **getConnectedPeers**(): `Iterable`<`PeerId`\>

#### Returns

`Iterable`<`PeerId`\>

a list connected peerIds

#### Defined in

[packages/core/src/index.ts:1078](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1078)

___

### getConnectionInfo

▸ **getConnectionInfo**(`peerId`): [`PeerStatus`](PeerStatus.md)

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peerId` | `PeerId` | of the node we want to get the connection info for |

#### Returns

[`PeerStatus`](PeerStatus.md)

various information about the connection

#### Defined in

[packages/core/src/index.ts:1103](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1103)

___

### getConnectivityHealth

▸ **getConnectivityHealth**(): `number`

Recalculates and retrieves the current connectivity health indicator.

#### Returns

`number`

#### Defined in

[packages/core/src/index.ts:845](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L845)

___

### getEntryNodes

▸ **getEntryNodes**(): `Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Returns

`Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Defined in

[packages/core/src/index.ts:1488](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1488)

___

### getEthereumAddress

▸ **getEthereumAddress**(): `Address`

#### Returns

`Address`

#### Defined in

[packages/core/src/index.ts:1507](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1507)

___

### getId

▸ **getId**(): `PeerId`

Gets the peer ID of this HOPR node.

#### Returns

`PeerId`

#### Defined in

[packages/core/src/index.ts:882](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L882)

___

### getIntermediateNodes

▸ `Private` **getIntermediateNodes**(`destination`, `hops?`): `Promise`<`PublicKey`[]\>

Takes a destination, and optionally the desired number of hops,
and samples randomly intermediate nodes
that will relay that message before it reaches its destination.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `destination` | `PublicKey` | instance of peerInfo that contains the peerId of the destination |
| `hops?` | `number` | optional number of required intermediate nodes (must be an integer 1,2,...MAX_HOPS inclusive) |

#### Returns

`Promise`<`PublicKey`[]\>

#### Defined in

[packages/core/src/index.ts:1546](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1546)

___

### getListeningAddresses

▸ **getListeningAddresses**(): `Multiaddr`[]

List the addresses on which the node is listening

#### Returns

`Multiaddr`[]

#### Defined in

[packages/core/src/index.ts:921](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L921)

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

Returns the current max listener value for the `EventEmitter` which is either
set by `emitter.setMaxListeners(n)` or defaults to [defaultMaxListeners](default.md#defaultmaxlisteners).

**`Since`**

v1.0.0

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:489

___

### getNativeBalance

▸ **getNativeBalance**(): `Promise`<`NativeBalance`\>

#### Returns

`Promise`<`NativeBalance`\>

#### Defined in

[packages/core/src/index.ts:1269](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1269)

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

#### Defined in

[packages/core/src/index.ts:935](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L935)

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

[packages/core/src/index.ts:1484](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1484)

___

### getTicketStatistics

▸ **getTicketStatistics**(): `Promise`<{ `losing`: `number` ; `neglected`: `number` ; `pending`: `number` ; `redeemed`: `number` ; `redeemedValue`: `Balance` ; `rejected`: `number` ; `rejectedValue`: `Balance` ; `unredeemed`: `number` = ack.length; `unredeemedValue`: `Balance` ; `winProportion`: `number`  }\>

#### Returns

`Promise`<{ `losing`: `number` ; `neglected`: `number` ; `pending`: `number` ; `redeemed`: `number` ; `redeemedValue`: `Balance` ; `rejected`: `number` ; `rejectedValue`: `Balance` ; `unredeemed`: `number` = ack.length; `unredeemedValue`: `Balance` ; `winProportion`: `number`  }\>

#### Defined in

[packages/core/src/index.ts:1430](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1430)

___

### getTickets

▸ **getTickets**(`peerId`): `Promise`<`Ticket`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

`Promise`<`Ticket`[]\>

#### Defined in

[packages/core/src/index.ts:1419](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1419)

___

### getVersion

▸ **getVersion**(): `string`

Returns the version of hopr-core.

#### Returns

`string`

#### Defined in

[packages/core/src/index.ts:838](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L838)

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

#### Defined in

[packages/core/src/index.ts:1534](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1534)

___

### listenerCount

▸ **listenerCount**(`eventName`): `number`

Returns the number of listeners listening to the event named `eventName`.

**`Since`**

v3.2.0

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `eventName` | `string` \| `symbol` | The name of the event being listened for |

#### Returns

`number`

#### Inherited from

EventEmitter.listenerCount

#### Defined in

node_modules/@types/node/events.d.ts:579

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

**`Since`**

v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.listeners

#### Defined in

node_modules/@types/node/events.d.ts:502

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

#### Defined in

[packages/core/src/index.ts:654](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L654)

___

### maybeEnableDhtServerMode

▸ `Private` **maybeEnableDhtServerMode**(): `Promise`<`void`\>

Checks if we are announcing public addresses to the DHT.
If so, switch DHT to `server`-mode such that the node will
reply to DHT queries of other nodes

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:579](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L579)

___

### maybeLogProfilingToGCloud

▸ `Private` **maybeLogProfilingToGCloud**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:603](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L603)

___

### off

▸ **off**(`eventName`, `listener`): [`default`](default.md)

Alias for `emitter.removeListener()`.

**`Since`**

v10.0.0

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

node_modules/@types/node/events.d.ts:462

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

**`Since`**

v0.1.101

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

node_modules/@types/node/events.d.ts:348

___

### onChannelWaitingForCommitment

▸ `Private` **onChannelWaitingForCommitment**(`c`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `ChannelEntry` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:622](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L622)

___

### onOwnChannelUpdated

▸ `Private` **onOwnChannelUpdated**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | `ChannelEntry` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:643](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L643)

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

#### Defined in

[packages/core/src/index.ts:672](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L672)

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

**`Since`**

v0.3.0

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

node_modules/@types/node/events.d.ts:377

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

#### Defined in

[packages/core/src/index.ts:1290](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1290)

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

#### Defined in

[packages/core/src/index.ts:1053](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1053)

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

**`Since`**

v6.0.0

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

node_modules/@types/node/events.d.ts:597

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

**`Since`**

v6.0.0

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

node_modules/@types/node/events.d.ts:613

___

### printAvailableAddresses

▸ `Private` **printAvailableAddresses**(): `string`

Pretty-print available addresses

#### Returns

`string`

#### Defined in

[packages/core/src/index.ts:541](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L541)

___

### printAvailableProtocols

▸ `Private` **printAvailableProtocols**(): `string`

Pretty-print available addresses

#### Returns

`string`

#### Defined in

[packages/core/src/index.ts:559](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L559)

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

**`Since`**

v9.4.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.rawListeners

#### Defined in

node_modules/@types/node/events.d.ts:532

___

### redeemAllTickets

▸ **redeemAllTickets**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:1451](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1451)

___

### redeemTicketsInChannel

▸ **redeemTicketsInChannel**(`peerId`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:1455](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1455)

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`default`](default.md)

Removes all listeners, or those of the specified `eventName`.

It is bad practice to remove listeners added elsewhere in the code,
particularly when the `EventEmitter` instance was created by some other
component or module (e.g. sockets or file streams).

Returns a reference to the `EventEmitter`, so that calls can be chained.

**`Since`**

v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

node_modules/@types/node/events.d.ts:473

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

**`Since`**

v0.1.26

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

node_modules/@types/node/events.d.ts:457

___

### sendMessage

▸ **sendMessage**(`msg`, `destination`, `intermediatePath?`, `hops?`): `Promise`<`string`\>

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `msg` | `Uint8Array` | message to send |
| `destination` | `PeerId` | PeerId of the destination |
| `intermediatePath?` | `PublicKey`[] | optional set path manually |
| `hops?` | `number` | optional number of required intermediate nodes |

#### Returns

`Promise`<`string`\>

hex representation of ack challenge

#### Defined in

[packages/core/src/index.ts:984](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L984)

___

### setChannelStrategy

▸ **setChannelStrategy**(`strategy`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `strategy` | [`ChannelStrategyInterface`](../interfaces/ChannelStrategyInterface.md) |

#### Returns

`void`

#### Defined in

[packages/core/src/index.ts:1248](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1248)

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`default`](default.md)

By default `EventEmitter`s will print a warning if more than `10` listeners are
added for a particular event. This is a useful default that helps finding
memory leaks. The `emitter.setMaxListeners()` method allows the limit to be
modified for this specific `EventEmitter` instance. The value can be set to`Infinity` (or `0`) to indicate an unlimited number of listeners.

Returns a reference to the `EventEmitter`, so that calls can be chained.

**`Since`**

v0.3.5

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:483

___

### signMessage

▸ **signMessage**(`message`): `Uint8Array`

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | `Uint8Array` |

#### Returns

`Uint8Array`

#### Defined in

[packages/core/src/index.ts:1496](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1496)

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

#### Defined in

[packages/core/src/index.ts:1274](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1274)

___

### start

▸ **start**(`__initialNodes?`, `__testingLibp2p?`): `Promise`<`void`\>

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
| `__initialNodes?` | { `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[] | - |
| `__testingLibp2p?` | `Libp2p` | use simulated libp2p instance for testing |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:305](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L305)

___

### startPeriodicStrategyCheck

▸ **startPeriodicStrategyCheck**(): `void`

#### Returns

`void`

#### Defined in

[packages/core/src/index.ts:1151](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1151)

___

### stop

▸ **stop**(): `Promise`<`void`\>

Shuts down the node and saves keys and peerBook in the database

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:853](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L853)

___

### strategyCloseChannel

▸ `Private` **strategyCloseChannel**(`destination`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `destination` | `string` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:731](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L731)

___

### strategyOpenChannel

▸ `Private` **strategyOpenChannel**(`status`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `status` | `OutgoingChannelStatus` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:712](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L712)

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

#### Defined in

[packages/core/src/index.ts:1144](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1144)

___

### tickChannelStrategy

▸ `Private` **tickChannelStrategy**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:776](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L776)

___

### updateChannelMetrics

▸ `Private` **updateChannelMetrics**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:741](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L741)

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

#### Defined in

[packages/core/src/index.ts:946](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L946)

___

### waitForFunds

▸ **waitForFunds**(): `Promise`<`void`\>

This is a utility method to wait until the node is funded.
A backoff is implemented, if node has not been funded and
MAX_DELAY is reached, this function will reject.

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:1566](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1566)

___

### waitForRunning

▸ **waitForRunning**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:1611](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1611)

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

#### Defined in

[packages/core/src/index.ts:1518](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L1518)

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

**`Since`**

v15.2.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `EventEmitter` \| `DOMEventTarget` |
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

**`Since`**

v0.9.12

**`Deprecated`**

Since v3.2.0 - Use `listenerCount` instead.

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

**`Since`**

v13.6.0, v12.16.0

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

**`Since`**

v11.13.0, v10.16.0

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

___

### setMaxListeners

▸ `Static` **setMaxListeners**(`n?`, `...eventTargets`): `void`

```js
const {
  setMaxListeners,
  EventEmitter
} = require('events');

const target = new EventTarget();
const emitter = new EventEmitter();

setMaxListeners(5, target, emitter);
```

**`Since`**

v15.4.0

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `n?` | `number` | A non-negative number. The maximum number of listeners per `EventTarget` event. |
| `...eventTargets` | (`EventEmitter` \| `DOMEventTarget`)[] | - |

#### Returns

`void`

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:280
