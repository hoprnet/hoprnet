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
- [getEventListener](LibP2P.md#geteventlistener)
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

packages/core/node_modules/@types/node/events.d.ts:46

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:52

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:53

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

packages/core/node_modules/@types/node/events.d.ts:45

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

▸ **addListener**(`event`, `listener`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:72

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

packages/core/node_modules/@types/node/events.d.ts:82

___

### eventNames

▸ **eventNames**(): (`string` \| `symbol`)[]

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:87

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:79

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

packages/core/node_modules/@types/node/events.d.ts:83

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

packages/core/node_modules/@types/node/events.d.ts:80

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

▸ **off**(`event`, `listener`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.off

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:76

___

### on

▸ **on**(`event`, `listener`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.on

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:73

___

### once

▸ **once**(`event`, `listener`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.once

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:74

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

▸ **prependListener**(`event`, `listener`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:85

___

### prependOnceListener

▸ **prependOnceListener**(`event`, `listener`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.prependOnceListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:86

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

packages/core/node_modules/@types/node/events.d.ts:81

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:77

___

### removeListener

▸ **removeListener**(`event`, `listener`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:75

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`LibP2P`](LibP2P.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[`LibP2P`](LibP2P.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:78

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

packages/core/node_modules/@types/node/events.d.ts:34

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

packages/core/node_modules/@types/node/events.d.ts:30

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

packages/core/node_modules/@types/node/events.d.ts:27

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

packages/core/node_modules/@types/node/events.d.ts:25

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

packages/core/node_modules/@types/node/events.d.ts:26
