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

- [constructor](libp2p.md#constructor)

### Properties

- [\_config](libp2p.md#_config)
- [\_dht](libp2p.md#_dht)
- [\_discovery](libp2p.md#_discovery)
- [\_isStarted](libp2p.md#_isstarted)
- [\_maybeConnect](libp2p.md#_maybeconnect)
- [\_modules](libp2p.md#_modules)
- [\_onDidStart](libp2p.md#_ondidstart)
- [\_onDiscoveryPeer](libp2p.md#_ondiscoverypeer)
- [\_options](libp2p.md#_options)
- [\_setupPeerDiscovery](libp2p.md#_setuppeerdiscovery)
- [\_transport](libp2p.md#_transport)
- [addressManager](libp2p.md#addressmanager)
- [addresses](libp2p.md#addresses)
- [connectionManager](libp2p.md#connectionmanager)
- [contentRouting](libp2p.md#contentrouting)
- [datastore](libp2p.md#datastore)
- [dialer](libp2p.md#dialer)
- [identifyService](libp2p.md#identifyservice)
- [keychain](libp2p.md#keychain)
- [metrics](libp2p.md#metrics)
- [natManager](libp2p.md#natmanager)
- [peerId](libp2p.md#peerid)
- [peerRouting](libp2p.md#peerrouting)
- [peerStore](libp2p.md#peerstore)
- [pubsub](libp2p.md#pubsub)
- [registrar](libp2p.md#registrar)
- [relay](libp2p.md#relay)
- [transportManager](libp2p.md#transportmanager)
- [upgrader](libp2p.md#upgrader)
- [captureRejectionSymbol](libp2p.md#capturerejectionsymbol)
- [captureRejections](libp2p.md#capturerejections)
- [defaultMaxListeners](libp2p.md#defaultmaxlisteners)
- [errorMonitor](libp2p.md#errormonitor)

### Accessors

- [connections](libp2p.md#connections)
- [multiaddrs](libp2p.md#multiaddrs)

### Methods

- [\_dial](libp2p.md#_dial)
- [\_onStarting](libp2p.md#_onstarting)
- [addListener](libp2p.md#addlistener)
- [dial](libp2p.md#dial)
- [dialProtocol](libp2p.md#dialprotocol)
- [emit](libp2p.md#emit)
- [eventNames](libp2p.md#eventnames)
- [getMaxListeners](libp2p.md#getmaxlisteners)
- [handle](libp2p.md#handle)
- [hangUp](libp2p.md#hangup)
- [isStarted](libp2p.md#isstarted)
- [listenerCount](libp2p.md#listenercount)
- [listeners](libp2p.md#listeners)
- [loadKeychain](libp2p.md#loadkeychain)
- [off](libp2p.md#off)
- [on](libp2p.md#on)
- [once](libp2p.md#once)
- [ping](libp2p.md#ping)
- [prependListener](libp2p.md#prependlistener)
- [prependOnceListener](libp2p.md#prependoncelistener)
- [rawListeners](libp2p.md#rawlisteners)
- [removeAllListeners](libp2p.md#removealllisteners)
- [removeListener](libp2p.md#removelistener)
- [setMaxListeners](libp2p.md#setmaxlisteners)
- [start](libp2p.md#start)
- [stop](libp2p.md#stop)
- [unhandle](libp2p.md#unhandle)
- [create](libp2p.md#create)
- [listenerCount](libp2p.md#listenercount)
- [on](libp2p.md#on)
- [once](libp2p.md#once)

## Constructors

### constructor

• **new LibP2P**(`_options`)

Libp2p node.

#### Parameters

| Name | Type |
| :------ | :------ |
| `_options` | [`Libp2pOptions`](../modules/libp2p.md#libp2poptions) & [`constructorOptions`](../modules/libp2p.md#constructoroptions) |

#### Overrides

EventEmitter.constructor

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:105

## Properties

### \_config

• **\_config**: { `dht`: { `enabled`: `boolean` ; `kBucketSize`: `number` ; `randomWalk`: { `enabled`: `boolean` ; `interval`: `number` ; `queriesPerPeriod`: `number` ; `timeout`: `number`  }  } ; `nat`: { `enabled`: `boolean` ; `externalIp`: ``null`` ; `gateway`: ``null`` ; `keepAlive`: `boolean` ; `pmp`: { `enabled`: `boolean`  } ; `ttl`: `number`  } ; `peerDiscovery`: { `autoDial`: `boolean`  } ; `pubsub`: { `enabled`: `boolean`  } ; `relay`: { `advertise`: { `bootDelay`: `number` ; `enabled`: `boolean` ; `ttl`: `number`  } ; `autoRelay`: { `enabled`: `boolean` ; `maxListeners`: `number`  } ; `enabled`: `boolean` ; `hop`: { `active`: `boolean` ; `enabled`: `boolean`  }  } ; `transport`: {}  } & [`Libp2pConfig`](../modules/libp2p.md#libp2pconfig)

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:210

___

### \_dht

• **\_dht**: `any`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:274

___

### \_discovery

• **\_discovery**: `Map`<`any`, `any`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:256

___

### \_isStarted

• **\_isStarted**: `boolean`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:300

___

### \_maybeConnect

• `Private` **\_maybeConnect**: `any`

Will dial to the given `peerId` if the current number of
connected peers is less than the configured `ConnectionManager`
minConnections.

**`param`**

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:400

___

### \_modules

• **\_modules**: [`Libp2pModules`](../modules/libp2p.md#libp2pmodules)

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:209

___

### \_onDidStart

• `Private` **\_onDidStart**: `any`

Called when libp2p has started and before it returns

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:391

___

### \_onDiscoveryPeer

• `Private` **\_onDiscoveryPeer**: `any`

Called whenever peer discovery services emit `peer` events.
Known peers may be emitted.

**`param`**

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:286

___

### \_options

• **\_options**: { `addresses`: { `announce`: `never`[] ; `announceFilter`: (`multiaddrs`: `Multiaddr`[]) => `Multiaddr`[] ; `listen`: `never`[] ; `noAnnounce`: `never`[]  } ; `config`: { `dht`: { `enabled`: `boolean` ; `kBucketSize`: `number` ; `randomWalk`: { `enabled`: `boolean` ; `interval`: `number` ; `queriesPerPeriod`: `number` ; `timeout`: `number`  }  } ; `nat`: { `enabled`: `boolean` ; `externalIp`: ``null`` ; `gateway`: ``null`` ; `keepAlive`: `boolean` ; `pmp`: { `enabled`: `boolean`  } ; `ttl`: `number`  } ; `peerDiscovery`: { `autoDial`: `boolean`  } ; `pubsub`: { `enabled`: `boolean`  } ; `relay`: { `advertise`: { `bootDelay`: `number` ; `enabled`: `boolean` ; `ttl`: `number`  } ; `autoRelay`: { `enabled`: `boolean` ; `maxListeners`: `number`  } ; `enabled`: `boolean` ; `hop`: { `active`: `boolean` ; `enabled`: `boolean`  }  } ; `transport`: {}  } ; `connectionManager`: { `minConnections`: `number`  } ; `dialer`: { `addressSorter`: (`addresses`: `Address`[]) => `Address`[] ; `dialTimeout`: `number` ; `maxDialsPerPeer`: `number` ; `maxParallelDials`: `number` ; `resolvers`: { `dnsaddr`: `any`  }  } ; `host`: { `agentVersion`: `string`  } ; `metrics`: { `enabled`: `boolean`  } ; `peerRouting`: { `refreshManager`: { `bootDelay`: `number` ; `enabled`: `boolean` ; `interval`: `number`  }  } ; `peerStore`: { `persistence`: `boolean` ; `threshold`: `number`  } ; `transportManager`: { `faultTolerance`: `number`  }  } & [`Libp2pOptions`](../modules/libp2p.md#libp2poptions) & [`constructorOptions`](../modules/libp2p.md#constructoroptions)

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:113

___

### \_setupPeerDiscovery

• `Private` **\_setupPeerDiscovery**: `any`

Initializes and starts peer discovery services

**`async`**

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:407

___

### \_transport

• **\_transport**: `any`[]

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:255

___

### addressManager

• **addressManager**: `AddressManager`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:208

___

### addresses

• **addresses**: { `announce`: `never`[] ; `announceFilter`: (`multiaddrs`: `Multiaddr`[]) => `Multiaddr`[] ; `listen`: `never`[] ; `noAnnounce`: `never`[]  } & `AddressManagerOptions`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:202

___

### connectionManager

• **connectionManager**: `ConnectionManager`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:257

___

### contentRouting

• **contentRouting**: `ContentRouting`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:278

___

### datastore

• **datastore**: `Datastore`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:200

___

### dialer

• **dialer**: `Dialer`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:271

___

### identifyService

• **identifyService**: `IdentifyService`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:273

___

### keychain

• **keychain**: `Keychain`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:259

___

### metrics

• **metrics**: `Metrics`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:258

___

### natManager

• **natManager**: `NatManager`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:262

___

### peerId

• **peerId**: `PeerId`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:199

___

### peerRouting

• **peerRouting**: `PeerRouting`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:277

___

### peerStore

• **peerStore**: `PeerStore`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:201

___

### pubsub

• **pubsub**: `PubsubBaseProtocol`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:276

___

### registrar

• **registrar**: `Registrar`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:263

___

### relay

• **relay**: `Relay`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:272

___

### transportManager

• **transportManager**: `TransportManager`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:261

___

### upgrader

• **upgrader**: `Upgrader`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:260

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [`captureRejectionSymbol`](default.md#capturerejectionsymbol)

#### Inherited from

EventEmitter.captureRejectionSymbol

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:43

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:49

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:50

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

packages/core/node_modules/@types/node/events.d.ts:42

## Accessors

### connections

• `get` **connections**(): `Map`<`string`, `Connection`[]\>

Gets a Map of the current connections. The keys are the stringified
`PeerId` of the peer. The value is an array of Connections to that peer.

#### Returns

`Map`<`string`, `Connection`[]\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:316

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

node_modules/libp2p/dist/src/index.d.ts:363

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

node_modules/libp2p/dist/src/index.d.ts:352

___

### \_onStarting

▸ **_onStarting**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:385

___

### addListener

▸ **addListener**(`event`, `listener`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:62

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

node_modules/libp2p/dist/src/index.d.ts:326

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

node_modules/libp2p/dist/src/index.d.ts:340

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

packages/core/node_modules/@types/node/events.d.ts:72

___

### eventNames

▸ **eventNames**(): (`string` \| `symbol`)[]

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:77

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:69

___

### handle

▸ **handle**(`protocols`, `handler`): `void`

Registers the `handler` for each protocol

#### Parameters

| Name | Type |
| :------ | :------ |
| `protocols` | `string` \| `string`[] |
| `handler` | (`props`: [`HandlerProps`](../modules/libp2p.md#handlerprops)) => `void` |

#### Returns

`void`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:270

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

node_modules/libp2p/dist/src/index.d.ts:370

___

### isStarted

▸ **isStarted**(): `boolean`

#### Returns

`boolean`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:309

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

packages/core/node_modules/@types/node/events.d.ts:73

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

packages/core/node_modules/@types/node/events.d.ts:70

___

### loadKeychain

▸ **loadKeychain**(): `Promise`<`void`\>

Load keychain keys from the datastore.
Imports the private key as 'self', if needed.

**`async`**

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:308

___

### off

▸ **off**(`event`, `listener`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.off

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:66

___

### on

▸ **on**(`event`, `listener`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.on

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:63

___

### once

▸ **once**(`event`, `listener`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.once

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:64

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

node_modules/libp2p/dist/src/index.d.ts:377

___

### prependListener

▸ **prependListener**(`event`, `listener`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`, `listener`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.prependOnceListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:76

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

packages/core/node_modules/@types/node/events.d.ts:71

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:67

___

### removeListener

▸ **removeListener**(`event`, `listener`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:65

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`LibP2P`](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[`LibP2P`](libp2p.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:68

___

### start

▸ **start**(): `Promise`<`void`\>

Starts the libp2p node and all its subsystems

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:292

___

### stop

▸ **stop**(): `Promise`<`void`\>

Stop the libp2p node by closing its listeners and open connections

**`async`**

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:299

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

node_modules/libp2p/dist/src/index.d.ts:384

___

### create

▸ `Static` **create**(`options`): `Promise`<[`LibP2P`](libp2p.md)\>

Like `new Libp2p(options)` except it will create a `PeerId`
instance if one is not provided in options.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [`Libp2pOptions`](../modules/libp2p.md#libp2poptions) & [`CreateOptions`](../modules/libp2p.md#createoptions) | Libp2p configuration options |

#### Returns

`Promise`<[`LibP2P`](libp2p.md)\>

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:105

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

packages/core/node_modules/@types/node/events.d.ts:31

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

packages/core/node_modules/@types/node/events.d.ts:28

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

packages/core/node_modules/@types/node/events.d.ts:26

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

packages/core/node_modules/@types/node/events.d.ts:27
