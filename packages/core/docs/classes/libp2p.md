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

- *EventEmitter*

  ↳ **LibP2P**

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

\+ **new LibP2P**(`_options`: [_Libp2pOptions_](../modules/libp2p.md#libp2poptions) & [_constructorOptions_](../modules/libp2p.md#constructoroptions)): [_LibP2P_](libp2p.md)

Libp2p node.

#### Parameters

| Name       | Type                                                                                                                    |
| :--------- | :---------------------------------------------------------------------------------------------------------------------- |
| `_options` | [_Libp2pOptions_](../modules/libp2p.md#libp2poptions) & [_constructorOptions_](../modules/libp2p.md#constructoroptions) |

**Returns:** [_LibP2P_](libp2p.md)

Overrides: EventEmitter.constructor

Defined in: node_modules/libp2p/dist/src/index.d.ts:105

## Properties

### \_config

• **\_config**: { `dht`: { `enabled`: *boolean* ; `kBucketSize`: *number* ; `randomWalk`: { `enabled`: *boolean* ; `interval`: *number* ; `queriesPerPeriod`: *number* ; `timeout`: *number*  }  } ; `nat`: { `enabled`: *boolean* ; `externalIp`: ``null`` ; `gateway`: ``null`` ; `keepAlive`: *boolean* ; `pmp`: { `enabled`: *boolean*  } ; `ttl`: *number*  } ; `peerDiscovery`: { `autoDial`: *boolean*  } ; `pubsub`: { `enabled`: *boolean*  } ; `relay`: { `advertise`: { `bootDelay`: *number* ; `enabled`: *boolean* ; `ttl`: *number*  } ; `autoRelay`: { `enabled`: *boolean* ; `maxListeners`: *number*  } ; `enabled`: *boolean* ; `hop`: { `active`: *boolean* ; `enabled`: *boolean*  }  } ; `transport`: {}  } & [*Libp2pConfig*](../modules/libp2p.md#libp2pconfig)

Defined in: node_modules/libp2p/dist/src/index.d.ts:210

---

### \_dht

• **\_dht**: _any_

Defined in: node_modules/libp2p/dist/src/index.d.ts:274

---

### \_discovery

• **\_discovery**: _Map_<any, any\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:256

---

### \_isStarted

• **\_isStarted**: _boolean_

Defined in: node_modules/libp2p/dist/src/index.d.ts:300

---

### \_maybeConnect

• `Private` **\_maybeConnect**: _any_

Will dial to the given `peerId` if the current number of
connected peers is less than the configured `ConnectionManager`
minConnections.

**`param`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:400

---

### \_modules

• **\_modules**: [*Libp2pModules*](../modules/libp2p.md#libp2pmodules)

Defined in: node_modules/libp2p/dist/src/index.d.ts:209

---

### \_onDidStart

• `Private` **\_onDidStart**: _any_

Called when libp2p has started and before it returns

Defined in: node_modules/libp2p/dist/src/index.d.ts:391

---

### \_onDiscoveryPeer

• `Private` **\_onDiscoveryPeer**: _any_

Called whenever peer discovery services emit `peer` events.
Known peers may be emitted.

**`param`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:286

---

### \_options

• **\_options**: { `addresses`: { `announce`: *never*[] ; `announceFilter`: (`multiaddrs`: *Multiaddr*[]) => *Multiaddr*[] ; `listen`: *never*[] ; `noAnnounce`: *never*[]  } ; `config`: { `dht`: { `enabled`: *boolean* ; `kBucketSize`: *number* ; `randomWalk`: { `enabled`: *boolean* ; `interval`: *number* ; `queriesPerPeriod`: *number* ; `timeout`: *number*  }  } ; `nat`: { `enabled`: *boolean* ; `externalIp`: ``null`` ; `gateway`: ``null`` ; `keepAlive`: *boolean* ; `pmp`: { `enabled`: *boolean*  } ; `ttl`: *number*  } ; `peerDiscovery`: { `autoDial`: *boolean*  } ; `pubsub`: { `enabled`: *boolean*  } ; `relay`: { `advertise`: { `bootDelay`: *number* ; `enabled`: *boolean* ; `ttl`: *number*  } ; `autoRelay`: { `enabled`: *boolean* ; `maxListeners`: *number*  } ; `enabled`: *boolean* ; `hop`: { `active`: *boolean* ; `enabled`: *boolean*  }  } ; `transport`: {}  } ; `connectionManager`: { `minConnections`: *number*  } ; `dialer`: { `addressSorter`: (`addresses`: Address[]) => Address[] ; `dialTimeout`: *number* ; `maxDialsPerPeer`: *number* ; `maxParallelDials`: *number* ; `resolvers`: { `dnsaddr`: *any*  }  } ; `host`: { `agentVersion`: *string*  } ; `metrics`: { `enabled`: *boolean*  } ; `peerRouting`: { `refreshManager`: { `bootDelay`: *number* ; `enabled`: *boolean* ; `interval`: *number*  }  } ; `peerStore`: { `persistence`: *boolean* ; `threshold`: *number*  } ; `transportManager`: { `faultTolerance`: *number*  }  } & [*Libp2pOptions*](../modules/libp2p.md#libp2poptions) & [*constructorOptions*](../modules/libp2p.md#constructoroptions)

Defined in: node_modules/libp2p/dist/src/index.d.ts:113

---

### \_setupPeerDiscovery

• `Private` **\_setupPeerDiscovery**: _any_

Initializes and starts peer discovery services

**`async`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:407

---

### \_transport

• **\_transport**: _any_[]

Defined in: node_modules/libp2p/dist/src/index.d.ts:255

---

### addressManager

• **addressManager**: _AddressManager_

Defined in: node_modules/libp2p/dist/src/index.d.ts:208

---

### addresses

• **addresses**: { `announce`: *never*[] ; `announceFilter`: (`multiaddrs`: *Multiaddr*[]) => *Multiaddr*[] ; `listen`: *never*[] ; `noAnnounce`: *never*[]  } & AddressManagerOptions

Defined in: node_modules/libp2p/dist/src/index.d.ts:202

---

### connectionManager

• **connectionManager**: _ConnectionManager_

Defined in: node_modules/libp2p/dist/src/index.d.ts:257

---

### contentRouting

• **contentRouting**: _ContentRouting_

Defined in: node_modules/libp2p/dist/src/index.d.ts:278

---

### datastore

• **datastore**: Datastore

Defined in: node_modules/libp2p/dist/src/index.d.ts:200

---

### dialer

• **dialer**: _Dialer_

Defined in: node_modules/libp2p/dist/src/index.d.ts:271

---

### identifyService

• **identifyService**: _IdentifyService_

Defined in: node_modules/libp2p/dist/src/index.d.ts:273

---

### keychain

• **keychain**: _Keychain_

Defined in: node_modules/libp2p/dist/src/index.d.ts:259

---

### metrics

• **metrics**: _Metrics_

Defined in: node_modules/libp2p/dist/src/index.d.ts:258

---

### natManager

• **natManager**: _NatManager_

Defined in: node_modules/libp2p/dist/src/index.d.ts:262

---

### peerId

• **peerId**: _PeerId_

Defined in: node_modules/libp2p/dist/src/index.d.ts:199

---

### peerRouting

• **peerRouting**: _PeerRouting_

Defined in: node_modules/libp2p/dist/src/index.d.ts:277

---

### peerStore

• **peerStore**: _PeerStore_

Defined in: node_modules/libp2p/dist/src/index.d.ts:201

---

### pubsub

• **pubsub**: _PubsubBaseProtocol_

Defined in: node_modules/libp2p/dist/src/index.d.ts:276

---

### registrar

• **registrar**: _Registrar_

Defined in: node_modules/libp2p/dist/src/index.d.ts:263

---

### relay

• **relay**: _Relay_

Defined in: node_modules/libp2p/dist/src/index.d.ts:272

---

### transportManager

• **transportManager**: _TransportManager_

Defined in: node_modules/libp2p/dist/src/index.d.ts:261

---

### upgrader

• **upgrader**: _Upgrader_

Defined in: node_modules/libp2p/dist/src/index.d.ts:260

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: *typeof* [*captureRejectionSymbol*](default.md#capturerejectionsymbol)

Inherited from: EventEmitter.captureRejectionSymbol

Defined in: packages/core/node_modules/@types/node/events.d.ts:43

___

### captureRejections

▪ `Static` **captureRejections**: *boolean*

Sets or gets the default captureRejection value for all emitters.

Inherited from: EventEmitter.captureRejections

Defined in: packages/core/node_modules/@types/node/events.d.ts:49

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: *number*

Inherited from: EventEmitter.defaultMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:50

___

### errorMonitor

▪ `Static` `Readonly` **errorMonitor**: *typeof* [*errorMonitor*](default.md#errormonitor)

This symbol shall be used to install a listener for only monitoring `'error'`
events. Listeners installed using this symbol are called before the regular
`'error'` listeners are called.

Installing a listener using this symbol does not change the behavior once an
`'error'` event is emitted, therefore the process will still crash if no
regular `'error'` listener is installed.

Inherited from: EventEmitter.errorMonitor

Defined in: packages/core/node_modules/@types/node/events.d.ts:42

## Accessors

### connections

• get **connections**(): _Map_<string, Connection[]\>

Gets a Map of the current connections. The keys are the stringified
`PeerId` of the peer. The value is an array of Connections to that peer.

**Returns:** _Map_<string, Connection[]\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:316

---

### multiaddrs

• get **multiaddrs**(): _Multiaddr_[]

Get a deduplicated list of peer advertising multiaddrs by concatenating
the listen addresses used by transports with any configured
announce addresses as well as observed addresses reported by peers.

If Announce addrs are specified, configured listen addresses will be
ignored though observed addresses will still be included.

**Returns:** _Multiaddr_[]

Defined in: node_modules/libp2p/dist/src/index.d.ts:363

## Methods

### \_dial

▸ **_dial**(`peer`: *string* \| *PeerId* \| *Multiaddr*, `options?`: *object*): *Promise*<Connection\>

**`async`**

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *string* \| *PeerId* \| *Multiaddr* | The peer to dial |
| `options?` | *object* | - |

**Returns:** *Promise*<Connection\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:352

___

### \_onStarting

▸ **\_onStarting**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:385

---

### addListener

▸ **addListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*LibP2P*](libp2p.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.addListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:62

---

### dial

▸ **dial**(`peer`: _string_ \| _PeerId_ \| _Multiaddr_, `options?`: { `signal?`: AbortSignal }): _Promise_<Connection\>

Dials to the provided peer. If successful, the known metadata of the
peer will be added to the nodes `peerStore`

#### Parameters

| Name              | Type                                | Description      |
| :---------------- | :---------------------------------- | :--------------- |
| `peer`            | _string_ \| _PeerId_ \| _Multiaddr_ | The peer to dial |
| `options?`        | _object_                            | -                |
| `options.signal?` | AbortSignal                         | -                |

**Returns:** _Promise_<Connection\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:326

---

### dialProtocol

▸ **dialProtocol**(`peer`: *string* \| *PeerId* \| *Multiaddr*, `protocols`: *string* \| *string*[], `options?`: { `signal?`: AbortSignal  }): *Promise*<{ `protocol`: *string* ; `stream`: *MuxedStream*  }\>

Dials to the provided peer and tries to handshake with the given protocols in order.
If successful, the known metadata of the peer will be added to the nodes `peerStore`,
and the `MuxedStream` will be returned together with the successful negotiated protocol.

**`async`**

#### Parameters

| Name              | Type                                | Description      |
| :---------------- | :---------------------------------- | :--------------- |
| `peer`            | _string_ \| _PeerId_ \| _Multiaddr_ | The peer to dial |
| `protocols`       | _string_ \| _string_[]              |                  |
| `options?`        | _object_                            | -                |
| `options.signal?` | AbortSignal                         | -                |

**Returns:** *Promise*<{ `protocol`: *string* ; `stream`: *MuxedStream*  }\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:340

---

### emit

▸ **emit**(`event`: _string_ \| _symbol_, ...`args`: _any_[]): _boolean_

#### Parameters

| Name      | Type                 |
| :-------- | :------------------- |
| `event`   | _string_ \| _symbol_ |
| `...args` | _any_[]              |

**Returns:** _boolean_

Inherited from: EventEmitter.emit

Defined in: packages/core/node_modules/@types/node/events.d.ts:72

___

### eventNames

▸ **eventNames**(): (*string* \| *symbol*)[]

**Returns:** (*string* \| *symbol*)[]

Inherited from: EventEmitter.eventNames

Defined in: packages/core/node_modules/@types/node/events.d.ts:77

---

### getMaxListeners

▸ **getMaxListeners**(): _number_

**Returns:** _number_

Inherited from: EventEmitter.getMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:69

---

### handle

▸ **handle**(`protocols`: *string* \| *string*[], `handler`: (`props`: [*HandlerProps*](../modules/libp2p.md#handlerprops)) => *void*): *void*

Registers the `handler` for each protocol

#### Parameters

| Name | Type |
| :------ | :------ |
| `protocols` | *string* \| *string*[] |
| `handler` | (`props`: [*HandlerProps*](../modules/libp2p.md#handlerprops)) => *void* |

**Returns:** _void_

Defined in: node_modules/libp2p/dist/src/index.d.ts:270

---

### hangUp

▸ **hangUp**(`peer`: _string_ \| _PeerId_ \| _Multiaddr_): _Promise_<void\>

Disconnects all connections to the given `peer`

#### Parameters

| Name   | Type                                | Description                      |
| :----- | :---------------------------------- | :------------------------------- |
| `peer` | _string_ \| _PeerId_ \| _Multiaddr_ | the peer to close connections to |

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:370

---

### isStarted

▸ **isStarted**(): _boolean_

**Returns:** _boolean_

Defined in: node_modules/libp2p/dist/src/index.d.ts:309

---

### listenerCount

▸ **listenerCount**(`event`: _string_ \| _symbol_): _number_

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** _number_

Inherited from: EventEmitter.listenerCount

Defined in: packages/core/node_modules/@types/node/events.d.ts:73

---

### listeners

▸ **listeners**(`event`: _string_ \| _symbol_): Function[]

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** Function[]

Inherited from: EventEmitter.listeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:70

---

### loadKeychain

▸ **loadKeychain**(): _Promise_<void\>

Load keychain keys from the datastore.
Imports the private key as 'self', if needed.

**`async`**

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:308

---

### off

▸ **off**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*LibP2P*](libp2p.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.off

Defined in: packages/core/node_modules/@types/node/events.d.ts:66

---

### on

▸ **on**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*LibP2P*](libp2p.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.on

Defined in: packages/core/node_modules/@types/node/events.d.ts:63

---

### once

▸ **once**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*LibP2P*](libp2p.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.once

Defined in: packages/core/node_modules/@types/node/events.d.ts:64

---

### ping

▸ **ping**(`peer`: _string_ \| _PeerId_ \| _Multiaddr_): _Promise_<number\>

Pings the given peer in order to obtain the operation latency.

#### Parameters

| Name   | Type                                | Description      |
| :----- | :---------------------------------- | :--------------- |
| `peer` | _string_ \| _PeerId_ \| _Multiaddr_ | The peer to ping |

**Returns:** _Promise_<number\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:377

___

### prependListener

▸ **prependListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*LibP2P*](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.prependListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*LibP2P*](libp2p.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.prependOnceListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:76

---

### rawListeners

▸ **rawListeners**(`event`: _string_ \| _symbol_): Function[]

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** Function[]

Inherited from: EventEmitter.rawListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:71

---

### removeAllListeners

▸ **removeAllListeners**(`event?`: *string* \| *symbol*): [*LibP2P*](libp2p.md)

#### Parameters

| Name     | Type                 |
| :------- | :------------------- |
| `event?` | _string_ \| _symbol_ |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.removeAllListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:67

---

### removeListener

▸ **removeListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*LibP2P*](libp2p.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.removeListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:65

---

### setMaxListeners

▸ **setMaxListeners**(`n`: *number*): [*LibP2P*](libp2p.md)

#### Parameters

| Name | Type     |
| :--- | :------- |
| `n`  | _number_ |

**Returns:** [*LibP2P*](libp2p.md)

Inherited from: EventEmitter.setMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:68

---

### start

▸ **start**(): _Promise_<void\>

Starts the libp2p node and all its subsystems

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:292

---

### stop

▸ **stop**(): _Promise_<void\>

Stop the libp2p node by closing its listeners and open connections

**`async`**

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:299

---

### unhandle

▸ **unhandle**(`protocols`: _string_ \| _string_[]): _void_

Removes the handler for each protocol. The protocol
will no longer be supported on streams.

#### Parameters

| Name        | Type                   |
| :---------- | :--------------------- |
| `protocols` | _string_ \| _string_[] |

**Returns:** _void_

Defined in: node_modules/libp2p/dist/src/index.d.ts:384

---

### create

▸ `Static` **create**(`options`: [_Libp2pOptions_](../modules/libp2p.md#libp2poptions) & [_CreateOptions_](../modules/libp2p.md#createoptions)): _Promise_<[_LibP2P_](libp2p.md)\>

Like `new Libp2p(options)` except it will create a `PeerId`
instance if one is not provided in options.

#### Parameters

| Name      | Type                                                                                                          | Description                  |
| :-------- | :------------------------------------------------------------------------------------------------------------ | :--------------------------- |
| `options` | [_Libp2pOptions_](../modules/libp2p.md#libp2poptions) & [_CreateOptions_](../modules/libp2p.md#createoptions) | Libp2p configuration options |

**Returns:** _Promise_<[_LibP2P_](libp2p.md)\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:105

___

### listenerCount

▸ `Static` **listenerCount**(`emitter`: *EventEmitter*, `event`: *string* \| *symbol*): *number*

**`deprecated`** since v4.0.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | *EventEmitter* |
| `event` | *string* \| *symbol* |

**Returns:** *number*

Inherited from: EventEmitter.listenerCount

Defined in: packages/core/node_modules/@types/node/events.d.ts:31

___

### on

▸ `Static` **on**(`emitter`: *EventEmitter*, `event`: *string*): *AsyncIterableIterator*<any\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | *EventEmitter* |
| `event` | *string* |

**Returns:** *AsyncIterableIterator*<any\>

Inherited from: EventEmitter.on

Defined in: packages/core/node_modules/@types/node/events.d.ts:28

___

### once

▸ `Static` **once**(`emitter`: *NodeEventTarget*, `event`: *string* \| *symbol*): *Promise*<any[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | *NodeEventTarget* |
| `event` | *string* \| *symbol* |

**Returns:** *Promise*<any[]\>

Inherited from: EventEmitter.once

Defined in: packages/core/node_modules/@types/node/events.d.ts:26

▸ `Static` **once**(`emitter`: DOMEventTarget, `event`: *string*): *Promise*<any[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | DOMEventTarget |
| `event` | *string* |

**Returns:** *Promise*<any[]\>

Inherited from: EventEmitter.once

Defined in: packages/core/node_modules/@types/node/events.d.ts:27
