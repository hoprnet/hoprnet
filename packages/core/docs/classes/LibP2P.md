[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / LibP2P

# Class: LibP2P

**`property`** {Connection} connection

**`property`** {MuxedStream} stream

**`property`** {string} protocol

**`property`** {boolean} [enabled = false]

**`property`** {number} [queriesPerPeriod = 1]

**`property`** {number} [interval = 300e3]

**`property`** {number} [timeout = 10e3]

**`property`** {boolean} [enabled = false]

**`property`** {number} [kBucketSize = 20]

**`property`** {RandomWalkOptions} [randomWalk]

**`property`** {boolean} [clientMode]

**`property`** {import('libp2p-interfaces/src/types').DhtSelectors} [selectors]

**`property`** {import('libp2p-interfaces/src/types').DhtValidators} [validators]

**`property`** {Datastore} [datastore]

**`property`** {boolean} persistence

**`property`** {boolean} enabled

**`property`** {boolean} enabled

**`property`** {boolean} [enabled = true]

**`property`** {import('./circuit').RelayAdvertiseOptions} [advertise]

**`property`** {import('./circuit').HopOptions} [hop]

**`property`** {import('./circuit').AutoRelayOptions} [autoRelay]

**`property`** {DhtOptions} [dht] dht module options

**`property`** {import('./nat-manager').NatManagerOptions} [nat]

**`property`** {Record<string, Object|boolean>} [peerDiscovery]

**`property`** {PubsubLocalOptions & PubsubOptions} [pubsub] pubsub module options

**`property`** {RelayOptions} [relay]

**`property`** {Record<string, Object>} [transport] transport options indexed by transport key

**`property`** {TransportFactory[]} transport

**`property`** {MuxerFactory[]} streamMuxer

**`property`** {Crypto[]} connEncryption

**`property`** {PeerDiscoveryFactory[]} [peerDiscovery]

**`property`** {PeerRoutingModule[]} [peerRouting]

**`property`** {ContentRoutingModule[]} [contentRouting]

**`property`** {Object} [dht]

**`property`** {{new(...args: any[]): Pubsub}} [pubsub]

**`property`** {Protector} [connProtector]

**`property`** {Libp2pModules} modules libp2p modules to use

**`property`** {import('./address-manager').AddressManagerOptions} [addresses]

**`property`** {import('./connection-manager').ConnectionManagerOptions} [connectionManager]

**`property`** {Datastore} [datastore]

**`property`** {import('./dialer').DialerOptions} [dialer]

**`property`** {import('./identify/index').HostProperties} [host] libp2p host

**`property`** {KeychainOptions & import('./keychain/index').KeychainOptions} [keychain]

**`property`** {MetricsOptions & import('./metrics').MetricsOptions} [metrics]

**`property`** {import('./peer-routing').PeerRoutingOptions} [peerRouting]

**`property`** {PeerStoreOptions & import('./peer-store/persistent').PersistentPeerStoreOptions} [peerStore]

**`property`** {import('./transport-manager').TransportManagerOptions} [transportManager]

**`property`** {Libp2pConfig} [config]

**`property`** {PeerId} peerId

**`property`** {PeerId} [peerId]

**`fires`** Libp2p#error Emitted when an error occurs

**`fires`** Libp2p#peer:discovery Emitted when a peer is discovered

## Hierarchy

- `EventEmitter`

  ↳ **`LibP2P`**

## Table of contents

### Constructors

- [constructor](LibP2P.md#constructor)

### Properties

- [\_config](LibP2P.md#_config)
- [\_dht](LibP2P.md#_dht)
- [\_discovery](LibP2P.md#_discovery)
- [\_isStarted](LibP2P.md#_isstarted)
- [\_maybeConnect](LibP2P.md#_maybeconnect)
- [\_modules](LibP2P.md#_modules)
- [\_onDidStart](LibP2P.md#_ondidstart)
- [\_onDiscoveryPeer](LibP2P.md#_ondiscoverypeer)
- [\_options](LibP2P.md#_options)
- [\_setupPeerDiscovery](LibP2P.md#_setuppeerdiscovery)
- [\_transport](LibP2P.md#_transport)
- [addressManager](LibP2P.md#addressmanager)
- [addresses](LibP2P.md#addresses)
- [connectionManager](LibP2P.md#connectionmanager)
- [contentRouting](LibP2P.md#contentrouting)
- [datastore](LibP2P.md#datastore)
- [dialer](LibP2P.md#dialer)
- [identifyService](LibP2P.md#identifyservice)
- [keychain](LibP2P.md#keychain)
- [metrics](LibP2P.md#metrics)
- [natManager](LibP2P.md#natmanager)
- [peerId](LibP2P.md#peerid)
- [peerRouting](LibP2P.md#peerrouting)
- [peerStore](LibP2P.md#peerstore)
- [pubsub](LibP2P.md#pubsub)
- [registrar](LibP2P.md#registrar)
- [relay](LibP2P.md#relay)
- [transportManager](LibP2P.md#transportmanager)
- [upgrader](LibP2P.md#upgrader)
- [captureRejectionSymbol](LibP2P.md#capturerejectionsymbol)
- [captureRejections](LibP2P.md#capturerejections)
- [defaultMaxListeners](LibP2P.md#defaultmaxlisteners)
- [errorMonitor](LibP2P.md#errormonitor)

### Accessors

- [connections](LibP2P.md#connections)
- [multiaddrs](LibP2P.md#multiaddrs)

### Methods

- [\_dial](LibP2P.md#_dial)
- [\_onStarting](LibP2P.md#_onstarting)
- [addListener](LibP2P.md#addlistener)
- [dial](LibP2P.md#dial)
- [dialProtocol](LibP2P.md#dialprotocol)
- [emit](LibP2P.md#emit)
- [eventNames](LibP2P.md#eventnames)
- [getMaxListeners](LibP2P.md#getmaxlisteners)
- [handle](LibP2P.md#handle)
- [hangUp](LibP2P.md#hangup)
- [isStarted](LibP2P.md#isstarted)
- [listenerCount](LibP2P.md#listenercount)
- [listeners](LibP2P.md#listeners)
- [loadKeychain](LibP2P.md#loadkeychain)
- [off](LibP2P.md#off)
- [on](LibP2P.md#on)
- [once](LibP2P.md#once)
- [ping](LibP2P.md#ping)
- [prependListener](LibP2P.md#prependlistener)
- [prependOnceListener](LibP2P.md#prependoncelistener)
- [rawListeners](LibP2P.md#rawlisteners)
- [removeAllListeners](LibP2P.md#removealllisteners)
- [removeListener](LibP2P.md#removelistener)
- [setMaxListeners](LibP2P.md#setmaxlisteners)
- [start](LibP2P.md#start)
- [stop](LibP2P.md#stop)
- [unhandle](LibP2P.md#unhandle)
- [create](LibP2P.md#create)
- [getEventListeners](LibP2P.md#geteventlisteners)
- [listenerCount](LibP2P.md#listenercount)
- [on](LibP2P.md#on)
- [once](LibP2P.md#once)

## Constructors

### constructor

• **new LibP2P**(`_options`)

Libp2p node.

#### Parameters

| Name | Type |
| :------ | :------ |
| `_options` | [`Libp2pOptions`](../modules/LibP2P.md#libp2poptions) & [`constructorOptions`](../modules/LibP2P.md#constructoroptions) |

#### Overrides

EventEmitter.constructor

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:112

## Properties

### \_config

• **\_config**: { `dht`: { `enabled`: `boolean` ; `kBucketSize`: `number` ; `randomWalk`: { `enabled`: `boolean` ; `interval`: `number` ; `queriesPerPeriod`: `number` ; `timeout`: `number`  }  } ; `nat`: { `enabled`: `boolean` ; `externalIp`: ``null`` ; `gateway`: ``null`` ; `keepAlive`: `boolean` ; `pmp`: { `enabled`: `boolean`  } ; `ttl`: `number`  } ; `peerDiscovery`: { `autoDial`: `boolean`  } ; `protocolPrefix`: `string` ; `pubsub`: { `enabled`: `boolean`  } ; `relay`: { `advertise`: { `bootDelay`: `number` ; `enabled`: `boolean` ; `ttl`: `number`  } ; `autoRelay`: { `enabled`: `boolean` ; `maxListeners`: `number`  } ; `enabled`: `boolean` ; `hop`: { `active`: `boolean` ; `enabled`: `boolean`  }  } ; `transport`: {}  } & [`Libp2pConfig`](../modules/LibP2P.md#libp2pconfig)

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:211

___

### \_dht

• **\_dht**: `any`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:276

___

### \_discovery

• **\_discovery**: `Map`<`any`, `any`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:258

___

### \_isStarted

• **\_isStarted**: `boolean`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:302

___

### \_maybeConnect

• `Private` **\_maybeConnect**: `any`

Will dial to the given `peerId` if the current number of
connected peers is less than the configured `ConnectionManager`
minConnections.

**`param`**

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:402

___

### \_modules

• **\_modules**: [`Libp2pModules`](../modules/LibP2P.md#libp2pmodules)

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:210

___

### \_onDidStart

• `Private` **\_onDidStart**: `any`

Called when libp2p has started and before it returns

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:393

___

### \_onDiscoveryPeer

• `Private` **\_onDiscoveryPeer**: `any`

Called whenever peer discovery services emit `peer` events.
Known peers may be emitted.

**`param`**

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:288

___

### \_options

• **\_options**: { `addresses`: { `announce`: `never`[] ; `listen`: `never`[] ; `noAnnounce`: `never`[] ; `announceFilter`: (`multiaddrs`: `Multiaddr`[]) => `Multiaddr`[]  } ; `config`: { `dht`: { `enabled`: `boolean` ; `kBucketSize`: `number` ; `randomWalk`: { `enabled`: `boolean` ; `interval`: `number` ; `queriesPerPeriod`: `number` ; `timeout`: `number`  }  } ; `nat`: { `enabled`: `boolean` ; `externalIp`: ``null`` ; `gateway`: ``null`` ; `keepAlive`: `boolean` ; `pmp`: { `enabled`: `boolean`  } ; `ttl`: `number`  } ; `peerDiscovery`: { `autoDial`: `boolean`  } ; `protocolPrefix`: `string` ; `pubsub`: { `enabled`: `boolean`  } ; `relay`: { `advertise`: { `bootDelay`: `number` ; `enabled`: `boolean` ; `ttl`: `number`  } ; `autoRelay`: { `enabled`: `boolean` ; `maxListeners`: `number`  } ; `enabled`: `boolean` ; `hop`: { `active`: `boolean` ; `enabled`: `boolean`  }  } ; `transport`: {}  } ; `connectionManager`: { `minConnections`: `number`  } ; `dialer`: { `addressSorter`: (`addresses`: `Address`[]) => `Address`[] ; `dialTimeout`: `number` ; `maxDialsPerPeer`: `number` ; `maxParallelDials`: `number` ; `resolvers`: { `dnsaddr`: `any`  }  } ; `host`: { `agentVersion`: `string`  } ; `metrics`: { `enabled`: `boolean`  } ; `peerRouting`: { `refreshManager`: { `bootDelay`: `number` ; `enabled`: `boolean` ; `interval`: `number`  }  } ; `peerStore`: { `persistence`: `boolean` ; `threshold`: `number`  } ; `transportManager`: { `faultTolerance`: `number`  }  } & [`Libp2pOptions`](../modules/LibP2P.md#libp2poptions) & [`constructorOptions`](../modules/LibP2P.md#constructoroptions)

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:113

___

### \_setupPeerDiscovery

• `Private` **\_setupPeerDiscovery**: `any`

Initializes and starts peer discovery services

**`async`**

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:409

___

### \_transport

• **\_transport**: `any`[]

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:257

___

### addressManager

• **addressManager**: `AddressManager`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:209

___

### addresses

• **addresses**: { `announce`: `never`[] ; `listen`: `never`[] ; `noAnnounce`: `never`[] ; `announceFilter`: (`multiaddrs`: `Multiaddr`[]) => `Multiaddr`[]  } & `AddressManagerOptions`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:203

___

### connectionManager

• **connectionManager**: `ConnectionManager`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:259

___

### contentRouting

• **contentRouting**: `ContentRouting`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:280

___

### datastore

• **datastore**: `Datastore`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:201

___

### dialer

• **dialer**: `Dialer`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:273

___

### identifyService

• **identifyService**: `IdentifyService`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:275

___

### keychain

• **keychain**: `Keychain`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:261

___

### metrics

• **metrics**: `Metrics`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:260

___

### natManager

• **natManager**: `NatManager`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:264

___

### peerId

• **peerId**: `PeerId`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:200

___

### peerRouting

• **peerRouting**: `PeerRouting`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:279

___

### peerStore

• **peerStore**: `PeerStore`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:202

___

### pubsub

• **pubsub**: `PubsubBaseProtocol`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:278

___

### registrar

• **registrar**: `Registrar`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:265

___

### relay

• **relay**: `Relay`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:274

___

### transportManager

• **transportManager**: `TransportManager`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:263

___

### upgrader

• **upgrader**: `Upgrader`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:262

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

## Accessors

### connections

• `get` **connections**(): `Map`<`string`, `Connection`[]\>

Gets a Map of the current connections. The keys are the stringified
`PeerId` of the peer. The value is an array of Connections to that peer.

#### Returns

`Map`<`string`, `Connection`[]\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:318

___

### multiaddrs

• `get` **multiaddrs**(): `Multiaddr`[]

Get a deduplicated list of peer advertising multiaddrs by concatenating
the listen addresses used by transports with any configured
announce addresses as well as observed addresses reported by peers.

If Announce addrs are specified, configured listen addresses will be
ignored though observed addresses will still be included.

#### Returns

`Multiaddr`[]

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:365

## Methods

### \_dial

▸ **_dial**(`peer`, `options?`): `Promise`<`Connection`\>

**`async`**

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `string` \| `PeerId` \| `Multiaddr` | The peer to dial |
| `options?` | `object` | - |

#### Returns

`Promise`<`Connection`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:354

___

### \_onStarting

▸ **_onStarting**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:387

___

### addListener

▸ **addListener**(`eventName`, `listener`): [`LibP2P`](LibP2P.md)

Alias for `emitter.on(eventName, listener)`.

**`since`** v0.1.26

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

node_modules/@types/node/events.d.ts:299

___

### dial

▸ **dial**(`peer`, `options?`): `Promise`<`Connection`\>

Dials to the provided peer. If successful, the known metadata of the
peer will be added to the nodes `peerStore`

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `string` \| `PeerId` \| `Multiaddr` | The peer to dial |
| `options?` | `Object` | - |
| `options.signal?` | `AbortSignal` | - |

#### Returns

`Promise`<`Connection`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:328

___

### dialProtocol

▸ **dialProtocol**(`peer`, `protocols`, `options?`): `Promise`<`Object`\>

Dials to the provided peer and tries to handshake with the given protocols in order.
If successful, the known metadata of the peer will be added to the nodes `peerStore`,
and the `MuxedStream` will be returned together with the successful negotiated protocol.

**`async`**

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `string` \| `PeerId` \| `Multiaddr` | The peer to dial |
| `protocols` | `string` \| `string`[] |  |
| `options?` | `Object` | - |
| `options.signal?` | `AbortSignal` | - |

#### Returns

`Promise`<`Object`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:342

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

### handle

▸ **handle**(`protocols`, `handler`): `void`

Registers the `handler` for each protocol

#### Parameters

| Name | Type |
| :------ | :------ |
| `protocols` | `string` \| `string`[] |
| `handler` | (`props`: [`HandlerProps`](../modules/LibP2P.md#handlerprops)) => `void` |

#### Returns

`void`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:272

___

### hangUp

▸ **hangUp**(`peer`): `Promise`<`void`\>

Disconnects all connections to the given `peer`

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `string` \| `PeerId` \| `Multiaddr` | the peer to close connections to |

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:372

___

### isStarted

▸ **isStarted**(): `boolean`

#### Returns

`boolean`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:311

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

### loadKeychain

▸ **loadKeychain**(): `Promise`<`void`\>

Load keychain keys from the datastore.
Imports the private key as 'self', if needed.

**`async`**

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:310

___

### off

▸ **off**(`eventName`, `listener`): [`LibP2P`](LibP2P.md)

Alias for `emitter.removeListener()`.

**`since`** v10.0.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.off

#### Defined in

node_modules/@types/node/events.d.ts:444

___

### on

▸ **on**(`eventName`, `listener`): [`LibP2P`](LibP2P.md)

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

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.on

#### Defined in

node_modules/@types/node/events.d.ts:330

___

### once

▸ **once**(`eventName`, `listener`): [`LibP2P`](LibP2P.md)

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

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.once

#### Defined in

node_modules/@types/node/events.d.ts:359

___

### ping

▸ **ping**(`peer`): `Promise`<`number`\>

Pings the given peer in order to obtain the operation latency.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `string` \| `PeerId` \| `Multiaddr` | The peer to ping |

#### Returns

`Promise`<`number`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:379

___

### prependListener

▸ **prependListener**(`eventName`, `listener`): [`LibP2P`](LibP2P.md)

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

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

node_modules/@types/node/events.d.ts:579

___

### prependOnceListener

▸ **prependOnceListener**(`eventName`, `listener`): [`LibP2P`](LibP2P.md)

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

[`LibP2P`](LibP2P.md)

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

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`LibP2P`](LibP2P.md)

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

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

node_modules/@types/node/events.d.ts:455

___

### removeListener

▸ **removeListener**(`eventName`, `listener`): [`LibP2P`](LibP2P.md)

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

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

node_modules/@types/node/events.d.ts:439

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`LibP2P`](LibP2P.md)

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

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

node_modules/@types/node/events.d.ts:465

___

### start

▸ **start**(): `Promise`<`void`\>

Starts the libp2p node and all its subsystems

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:294

___

### stop

▸ **stop**(): `Promise`<`void`\>

Stop the libp2p node by closing its listeners and open connections

**`async`**

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:301

___

### unhandle

▸ **unhandle**(`protocols`): `void`

Removes the handler for each protocol. The protocol
will no longer be supported on streams.

#### Parameters

| Name | Type |
| :------ | :------ |
| `protocols` | `string` \| `string`[] |

#### Returns

`void`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:386

___

### create

▸ `Static` **create**(`options`): `Promise`<[`LibP2P`](LibP2P.md)\>

Like `new Libp2p(options)` except it will create a `PeerId`
instance if one is not provided in options.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [`Libp2pOptions`](../modules/LibP2P.md#libp2poptions) & [`CreateOptions`](../modules/LibP2P.md#createoptions) | Libp2p configuration options |

#### Returns

`Promise`<[`LibP2P`](LibP2P.md)\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:105

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
