[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [index](../modules/index.md) / LibP2P

# Class: LibP2P

[index](../modules/index.md).LibP2P

**`property`** {boolean} persistence

**`property`** {boolean} enabled

**`property`** {import('./circuit').RelayAdvertiseOptions} advertise

**`property`** {import('./circuit').HopOptions} hop

**`property`** {import('./circuit').AutoRelayOptions} autoRelay

**`property`** {Object} [dht] dht module options

**`property`** {Object} [peerDiscovery]

**`property`** {Pubsub} [pubsub] pubsub module options

**`property`** {RelayOptions} [relay]

**`property`** {Record<string, Object>} [transport] transport options indexed by transport key

**`property`** {TransportFactory[]} transport

**`property`** {MuxerFactory[]} streamMuxer

**`property`** {Crypto[]} connEncryption

**`property`** {Libp2pModules} modules libp2p modules to use

**`property`** {import('./address-manager').AddressManagerOptions} [addresses]

**`property`** {import('./connection-manager').ConnectionManagerOptions} [connectionManager]

**`property`** {import('./dialer').DialerOptions} [dialer]

**`property`** {import('./metrics').MetricsOptions} [metrics]

**`property`** {Object} [keychain]

**`property`** {import('./transport-manager').TransportManagerOptions} [transportManager]

**`property`** {PeerStoreOptions & import('./peer-store/persistent').PersistentPeerStoreOptions} [peerStore]

**`property`** {Libp2pConfig} [config]

**`property`** {PeerId} peerId

**`property`** {PeerId} [peerId]

**`fires`** Libp2p#error Emitted when an error occurs

**`fires`** Libp2p#peer:discovery Emitted when a peer is discovered

## Hierarchy

- *Libp2p\_base*

  ↳ **LibP2P**

## Table of contents

### Constructors

- [constructor](index.libp2p-1.md#constructor)

### Properties

- [\_config](index.libp2p-1.md#_config)
- [\_dht](index.libp2p-1.md#_dht)
- [\_discovery](index.libp2p-1.md#_discovery)
- [\_isStarted](index.libp2p-1.md#_isstarted)
- [\_maybeConnect](index.libp2p-1.md#_maybeconnect)
- [\_modules](index.libp2p-1.md#_modules)
- [\_onDidStart](index.libp2p-1.md#_ondidstart)
- [\_onDiscoveryPeer](index.libp2p-1.md#_ondiscoverypeer)
- [\_options](index.libp2p-1.md#_options)
- [\_setupPeerDiscovery](index.libp2p-1.md#_setuppeerdiscovery)
- [\_transport](index.libp2p-1.md#_transport)
- [addressManager](index.libp2p-1.md#addressmanager)
- [addresses](index.libp2p-1.md#addresses)
- [connectionManager](index.libp2p-1.md#connectionmanager)
- [contentRouting](index.libp2p-1.md#contentrouting)
- [datastore](index.libp2p-1.md#datastore)
- [dialer](index.libp2p-1.md#dialer)
- [identifyService](index.libp2p-1.md#identifyservice)
- [keychain](index.libp2p-1.md#keychain)
- [metrics](index.libp2p-1.md#metrics)
- [natManager](index.libp2p-1.md#natmanager)
- [peerId](index.libp2p-1.md#peerid)
- [peerRouting](index.libp2p-1.md#peerrouting)
- [peerStore](index.libp2p-1.md#peerstore)
- [pubsub](index.libp2p-1.md#pubsub)
- [registrar](index.libp2p-1.md#registrar)
- [relay](index.libp2p-1.md#relay)
- [transportManager](index.libp2p-1.md#transportmanager)
- [upgrader](index.libp2p-1.md#upgrader)

### Accessors

- [connections](index.libp2p-1.md#connections)
- [multiaddrs](index.libp2p-1.md#multiaddrs)

### Methods

- [\_onStarting](index.libp2p-1.md#_onstarting)
- [addListener](index.libp2p-1.md#addlistener)
- [dial](index.libp2p-1.md#dial)
- [dialProtocol](index.libp2p-1.md#dialprotocol)
- [emit](index.libp2p-1.md#emit)
- [getMaxListeners](index.libp2p-1.md#getmaxlisteners)
- [handle](index.libp2p-1.md#handle)
- [hangUp](index.libp2p-1.md#hangup)
- [isStarted](index.libp2p-1.md#isstarted)
- [listenerCount](index.libp2p-1.md#listenercount)
- [listeners](index.libp2p-1.md#listeners)
- [loadKeychain](index.libp2p-1.md#loadkeychain)
- [off](index.libp2p-1.md#off)
- [on](index.libp2p-1.md#on)
- [once](index.libp2p-1.md#once)
- [ping](index.libp2p-1.md#ping)
- [rawListeners](index.libp2p-1.md#rawlisteners)
- [removeAllListeners](index.libp2p-1.md#removealllisteners)
- [removeListener](index.libp2p-1.md#removelistener)
- [setMaxListeners](index.libp2p-1.md#setmaxlisteners)
- [start](index.libp2p-1.md#start)
- [stop](index.libp2p-1.md#stop)
- [unhandle](index.libp2p-1.md#unhandle)
- [create](index.libp2p-1.md#create)

## Constructors

### constructor

\+ **new LibP2P**(`_options`: [*Libp2pOptions*](../modules/index.libp2p.md#libp2poptions) & [*constructorOptions*](../modules/index.libp2p.md#constructoroptions)): [*LibP2P*](index.libp2p-1.md)

Libp2p node.

#### Parameters

| Name | Type |
| :------ | :------ |
| `_options` | [*Libp2pOptions*](../modules/index.libp2p.md#libp2poptions) & [*constructorOptions*](../modules/index.libp2p.md#constructoroptions) |

**Returns:** [*LibP2P*](index.libp2p-1.md)

Overrides: Libp2p\_base.constructor

Defined in: node_modules/libp2p/dist/src/index.d.ts:63

## Properties

### \_config

• **\_config**: *any*

Defined in: node_modules/libp2p/dist/src/index.d.ts:79

___

### \_dht

• **\_dht**: *any*

Defined in: node_modules/libp2p/dist/src/index.d.ts:103

___

### \_discovery

• **\_discovery**: *Map*<any, any\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:81

___

### \_isStarted

• **\_isStarted**: *boolean*

Defined in: node_modules/libp2p/dist/src/index.d.ts:129

___

### \_maybeConnect

• `Private` **\_maybeConnect**: *any*

Will dial to the given `peerId` if the current number of
connected peers is less than the configured `ConnectionManager`
minConnections.

**`param`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:220

___

### \_modules

• **\_modules**: *any*

Defined in: node_modules/libp2p/dist/src/index.d.ts:78

___

### \_onDidStart

• `Private` **\_onDidStart**: *any*

Called when libp2p has started and before it returns

Defined in: node_modules/libp2p/dist/src/index.d.ts:211

___

### \_onDiscoveryPeer

• `Private` **\_onDiscoveryPeer**: *any*

Called whenever peer discovery services emit `peer` events.
Known peers may be emitted.

**`param`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:115

___

### \_options

• **\_options**: *any*

Defined in: node_modules/libp2p/dist/src/index.d.ts:71

___

### \_setupPeerDiscovery

• `Private` **\_setupPeerDiscovery**: *any*

Initializes and starts peer discovery services

**`async`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:227

___

### \_transport

• **\_transport**: *any*[]

Defined in: node_modules/libp2p/dist/src/index.d.ts:80

___

### addressManager

• **addressManager**: *AddressManager*

Defined in: node_modules/libp2p/dist/src/index.d.ts:77

___

### addresses

• **addresses**: *any*

Defined in: node_modules/libp2p/dist/src/index.d.ts:76

___

### connectionManager

• **connectionManager**: *ConnectionManager*

Defined in: node_modules/libp2p/dist/src/index.d.ts:82

___

### contentRouting

• **contentRouting**: *ContentRouting*

Defined in: node_modules/libp2p/dist/src/index.d.ts:107

___

### datastore

• **datastore**: *any*

Defined in: node_modules/libp2p/dist/src/index.d.ts:74

___

### dialer

• **dialer**: *Dialer*

Defined in: node_modules/libp2p/dist/src/index.d.ts:100

___

### identifyService

• **identifyService**: *IdentifyService*

Defined in: node_modules/libp2p/dist/src/index.d.ts:102

___

### keychain

• **keychain**: *Keychain*

Defined in: node_modules/libp2p/dist/src/index.d.ts:84

___

### metrics

• **metrics**: *Metrics*

Defined in: node_modules/libp2p/dist/src/index.d.ts:83

___

### natManager

• **natManager**: *NatManager*

Defined in: node_modules/libp2p/dist/src/index.d.ts:87

___

### peerId

• **peerId**: *PeerId*

Defined in: node_modules/libp2p/dist/src/index.d.ts:73

___

### peerRouting

• **peerRouting**: *PeerRouting*

Defined in: node_modules/libp2p/dist/src/index.d.ts:106

___

### peerStore

• **peerStore**: *PeerStore*

Defined in: node_modules/libp2p/dist/src/index.d.ts:75

___

### pubsub

• **pubsub**: *PubsubBaseProtocol*

Defined in: node_modules/libp2p/dist/src/index.d.ts:105

___

### registrar

• **registrar**: *Registrar*

Defined in: node_modules/libp2p/dist/src/index.d.ts:88

___

### relay

• **relay**: *Relay*

Defined in: node_modules/libp2p/dist/src/index.d.ts:101

___

### transportManager

• **transportManager**: *TransportManager*

Defined in: node_modules/libp2p/dist/src/index.d.ts:86

___

### upgrader

• **upgrader**: *Upgrader*

Defined in: node_modules/libp2p/dist/src/index.d.ts:85

## Accessors

### connections

• get **connections**(): *Map*<string, Connection[]\>

Gets a Map of the current connections. The keys are the stringified
`PeerId` of the peer. The value is an array of Connections to that peer.

**Returns:** *Map*<string, Connection[]\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:145

___

### multiaddrs

• get **multiaddrs**(): *Multiaddr*[]

Get a deduplicated list of peer advertising multiaddrs by concatenating
the listen addresses used by transports with any configured
announce addresses as well as observed addresses reported by peers.

If Announce addrs are specified, configured listen addresses will be
ignored though observed addresses will still be included.

**Returns:** *Multiaddr*[]

Defined in: node_modules/libp2p/dist/src/index.d.ts:183

## Methods

### \_onStarting

▸ **_onStarting**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:205

___

### addListener

▸ **addListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** *any*

Inherited from: Libp2p\_base.addListener

Defined in: node_modules/libp2p/dist/src/types.d.ts:74

___

### dial

▸ **dial**(`peer`: *string* \| *PeerId* \| *Multiaddr*, `options?`: { `signal?`: AbortSignal  }): *Promise*<Connection\>

Dials to the provided peer. If successful, the known metadata of the
peer will be added to the nodes `peerStore`

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *string* \| *PeerId* \| *Multiaddr* | The peer to dial |
| `options?` | *object* | - |
| `options.signal?` | AbortSignal | - |

**Returns:** *Promise*<Connection\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:155

___

### dialProtocol

▸ **dialProtocol**(`peer`: *string* \| *PeerId* \| *Multiaddr*, `protocols`: *string* \| *string*[], `options?`: { `signal?`: AbortSignal  }): *Promise*<any\>

Dials to the provided peer and handshakes with the given protocol.
If successful, the known metadata of the peer will be added to the nodes `peerStore`,
and the `Connection` will be returned

**`async`**

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *string* \| *PeerId* \| *Multiaddr* | The peer to dial |
| `protocols` | *string* \| *string*[] |  |
| `options?` | *object* | - |
| `options.signal?` | AbortSignal | - |

**Returns:** *Promise*<any\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:170

___

### emit

▸ **emit**(`event`: *string* \| *symbol*, ...`args`: *any*[]): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `...args` | *any*[] |

**Returns:** *boolean*

Inherited from: Libp2p\_base.emit

Defined in: node_modules/libp2p/dist/src/types.d.ts:84

___

### getMaxListeners

▸ **getMaxListeners**(): *number*

**Returns:** *number*

Inherited from: Libp2p\_base.getMaxListeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:81

___

### handle

▸ **handle**(`protocols`: *string* \| *string*[], `handler`: (`__namedParameters`: { `connection`: *any* ; `protocol`: *any* ; `stream`: *any*  }) => *void*): *void*

Registers the `handler` for each protocol

#### Parameters

| Name | Type |
| :------ | :------ |
| `protocols` | *string* \| *string*[] |
| `handler` | (`__namedParameters`: { `connection`: *any* ; `protocol`: *any* ; `stream`: *any*  }) => *void* |

**Returns:** *void*

Defined in: node_modules/libp2p/dist/src/index.d.ts:95

___

### hangUp

▸ **hangUp**(`peer`: *string* \| *PeerId* \| *Multiaddr*): *Promise*<void\>

Disconnects all connections to the given `peer`

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *string* \| *PeerId* \| *Multiaddr* | the peer to close connections to |

**Returns:** *Promise*<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:190

___

### isStarted

▸ **isStarted**(): *boolean*

**Returns:** *boolean*

Defined in: node_modules/libp2p/dist/src/index.d.ts:138

___

### listenerCount

▸ **listenerCount**(`event`: *string* \| *symbol*): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** *number*

Inherited from: Libp2p\_base.listenerCount

Defined in: node_modules/libp2p/dist/src/types.d.ts:85

___

### listeners

▸ **listeners**(`event`: *string* \| *symbol*): Function[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** Function[]

Inherited from: Libp2p\_base.listeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:82

___

### loadKeychain

▸ **loadKeychain**(): *Promise*<void\>

Load keychain keys from the datastore.
Imports the private key as 'self', if needed.

**`async`**

**Returns:** *Promise*<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:137

___

### off

▸ **off**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** *any*

Inherited from: Libp2p\_base.off

Defined in: node_modules/libp2p/dist/src/types.d.ts:78

___

### on

▸ **on**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** *any*

Inherited from: Libp2p\_base.on

Defined in: node_modules/libp2p/dist/src/types.d.ts:75

___

### once

▸ **once**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** *any*

Inherited from: Libp2p\_base.once

Defined in: node_modules/libp2p/dist/src/types.d.ts:76

___

### ping

▸ **ping**(`peer`: *string* \| *PeerId* \| *Multiaddr*): *Promise*<number\>

Pings the given peer in order to obtain the operation latency.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *string* \| *PeerId* \| *Multiaddr* | The peer to ping |

**Returns:** *Promise*<number\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:197

___

### rawListeners

▸ **rawListeners**(`event`: *string* \| *symbol*): Function[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** Function[]

Inherited from: Libp2p\_base.rawListeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:83

___

### removeAllListeners

▸ **removeAllListeners**(`event?`: *string* \| *symbol*): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | *string* \| *symbol* |

**Returns:** *any*

Inherited from: Libp2p\_base.removeAllListeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:79

___

### removeListener

▸ **removeListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** *any*

Inherited from: Libp2p\_base.removeListener

Defined in: node_modules/libp2p/dist/src/types.d.ts:77

___

### setMaxListeners

▸ **setMaxListeners**(`n`: *number*): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | *number* |

**Returns:** *any*

Inherited from: Libp2p\_base.setMaxListeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:80

___

### start

▸ **start**(): *Promise*<void\>

Starts the libp2p node and all its subsystems

**Returns:** *Promise*<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:121

___

### stop

▸ **stop**(): *Promise*<void\>

Stop the libp2p node by closing its listeners and open connections

**`async`**

**Returns:** *Promise*<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:128

___

### unhandle

▸ **unhandle**(`protocols`: *string* \| *string*[]): *void*

Removes the handler for each protocol. The protocol
will no longer be supported on streams.

#### Parameters

| Name | Type |
| :------ | :------ |
| `protocols` | *string* \| *string*[] |

**Returns:** *void*

Defined in: node_modules/libp2p/dist/src/index.d.ts:204

___

### create

▸ `Static` **create**(`options`: [*Libp2pOptions*](../modules/index.libp2p.md#libp2poptions) & [*CreateOptions*](../modules/index.libp2p.md#createoptions)): *Promise*<[*LibP2P*](index.libp2p-1.md)\>

Like `new Libp2p(options)` except it will create a `PeerId`
instance if one is not provided in options.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [*Libp2pOptions*](../modules/index.libp2p.md#libp2poptions) & [*CreateOptions*](../modules/index.libp2p.md#createoptions) | Libp2p configuration options |

**Returns:** *Promise*<[*LibP2P*](index.libp2p-1.md)\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:63
