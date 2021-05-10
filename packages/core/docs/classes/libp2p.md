[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / LibP2P

# Class: LibP2P

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

- _Libp2p_base_

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

### Accessors

- [connections](libp2p.md#connections)
- [multiaddrs](libp2p.md#multiaddrs)

### Methods

- [\_onStarting](libp2p.md#_onstarting)
- [addListener](libp2p.md#addlistener)
- [dial](libp2p.md#dial)
- [dialProtocol](libp2p.md#dialprotocol)
- [emit](libp2p.md#emit)
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
- [rawListeners](libp2p.md#rawlisteners)
- [removeAllListeners](libp2p.md#removealllisteners)
- [removeListener](libp2p.md#removelistener)
- [setMaxListeners](libp2p.md#setmaxlisteners)
- [start](libp2p.md#start)
- [stop](libp2p.md#stop)
- [unhandle](libp2p.md#unhandle)
- [create](libp2p.md#create)

## Constructors

### constructor

\+ **new LibP2P**(`_options`: [_Libp2pOptions_](../modules/libp2p.md#libp2poptions) & [_constructorOptions_](../modules/libp2p.md#constructoroptions)): [_LibP2P_](libp2p.md)

Libp2p node.

#### Parameters

| Name       | Type                                                                                                                    |
| :--------- | :---------------------------------------------------------------------------------------------------------------------- |
| `_options` | [_Libp2pOptions_](../modules/libp2p.md#libp2poptions) & [_constructorOptions_](../modules/libp2p.md#constructoroptions) |

**Returns:** [_LibP2P_](libp2p.md)

Overrides: Libp2p_base.constructor

Defined in: node_modules/libp2p/dist/src/index.d.ts:63

## Properties

### \_config

• **\_config**: _any_

Defined in: node_modules/libp2p/dist/src/index.d.ts:79

---

### \_dht

• **\_dht**: _any_

Defined in: node_modules/libp2p/dist/src/index.d.ts:103

---

### \_discovery

• **\_discovery**: _Map_<any, any\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:81

---

### \_isStarted

• **\_isStarted**: _boolean_

Defined in: node_modules/libp2p/dist/src/index.d.ts:129

---

### \_maybeConnect

• `Private` **\_maybeConnect**: _any_

Will dial to the given `peerId` if the current number of
connected peers is less than the configured `ConnectionManager`
minConnections.

**`param`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:220

---

### \_modules

• **\_modules**: _any_

Defined in: node_modules/libp2p/dist/src/index.d.ts:78

---

### \_onDidStart

• `Private` **\_onDidStart**: _any_

Called when libp2p has started and before it returns

Defined in: node_modules/libp2p/dist/src/index.d.ts:211

---

### \_onDiscoveryPeer

• `Private` **\_onDiscoveryPeer**: _any_

Called whenever peer discovery services emit `peer` events.
Known peers may be emitted.

**`param`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:115

---

### \_options

• **\_options**: _any_

Defined in: node_modules/libp2p/dist/src/index.d.ts:71

---

### \_setupPeerDiscovery

• `Private` **\_setupPeerDiscovery**: _any_

Initializes and starts peer discovery services

**`async`**

Defined in: node_modules/libp2p/dist/src/index.d.ts:227

---

### \_transport

• **\_transport**: _any_[]

Defined in: node_modules/libp2p/dist/src/index.d.ts:80

---

### addressManager

• **addressManager**: _AddressManager_

Defined in: node_modules/libp2p/dist/src/index.d.ts:77

---

### addresses

• **addresses**: _any_

Defined in: node_modules/libp2p/dist/src/index.d.ts:76

---

### connectionManager

• **connectionManager**: _ConnectionManager_

Defined in: node_modules/libp2p/dist/src/index.d.ts:82

---

### contentRouting

• **contentRouting**: _ContentRouting_

Defined in: node_modules/libp2p/dist/src/index.d.ts:107

---

### datastore

• **datastore**: _any_

Defined in: node_modules/libp2p/dist/src/index.d.ts:74

---

### dialer

• **dialer**: _Dialer_

Defined in: node_modules/libp2p/dist/src/index.d.ts:100

---

### identifyService

• **identifyService**: _IdentifyService_

Defined in: node_modules/libp2p/dist/src/index.d.ts:102

---

### keychain

• **keychain**: _Keychain_

Defined in: node_modules/libp2p/dist/src/index.d.ts:84

---

### metrics

• **metrics**: _Metrics_

Defined in: node_modules/libp2p/dist/src/index.d.ts:83

---

### natManager

• **natManager**: _NatManager_

Defined in: node_modules/libp2p/dist/src/index.d.ts:87

---

### peerId

• **peerId**: _PeerId_

Defined in: node_modules/libp2p/dist/src/index.d.ts:73

---

### peerRouting

• **peerRouting**: _PeerRouting_

Defined in: node_modules/libp2p/dist/src/index.d.ts:106

---

### peerStore

• **peerStore**: _PeerStore_

Defined in: node_modules/libp2p/dist/src/index.d.ts:75

---

### pubsub

• **pubsub**: _PubsubBaseProtocol_

Defined in: node_modules/libp2p/dist/src/index.d.ts:105

---

### registrar

• **registrar**: _Registrar_

Defined in: node_modules/libp2p/dist/src/index.d.ts:88

---

### relay

• **relay**: _Relay_

Defined in: node_modules/libp2p/dist/src/index.d.ts:101

---

### transportManager

• **transportManager**: _TransportManager_

Defined in: node_modules/libp2p/dist/src/index.d.ts:86

---

### upgrader

• **upgrader**: _Upgrader_

Defined in: node_modules/libp2p/dist/src/index.d.ts:85

## Accessors

### connections

• get **connections**(): _Map_<string, Connection[]\>

Gets a Map of the current connections. The keys are the stringified
`PeerId` of the peer. The value is an array of Connections to that peer.

**Returns:** _Map_<string, Connection[]\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:145

---

### multiaddrs

• get **multiaddrs**(): _Multiaddr_[]

Get a deduplicated list of peer advertising multiaddrs by concatenating
the listen addresses used by transports with any configured
announce addresses as well as observed addresses reported by peers.

If Announce addrs are specified, configured listen addresses will be
ignored though observed addresses will still be included.

**Returns:** _Multiaddr_[]

Defined in: node_modules/libp2p/dist/src/index.d.ts:183

## Methods

### \_onStarting

▸ **\_onStarting**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:205

---

### addListener

▸ **addListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): _any_

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** _any_

Inherited from: Libp2p_base.addListener

Defined in: node_modules/libp2p/dist/src/types.d.ts:74

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

Defined in: node_modules/libp2p/dist/src/index.d.ts:155

---

### dialProtocol

▸ **dialProtocol**(`peer`: _string_ \| _PeerId_ \| _Multiaddr_, `protocols`: _string_ \| _string_[], `options?`: { `signal?`: AbortSignal }): _Promise_<any\>

Dials to the provided peer and handshakes with the given protocol.
If successful, the known metadata of the peer will be added to the nodes `peerStore`,
and the `Connection` will be returned

**`async`**

#### Parameters

| Name              | Type                                | Description      |
| :---------------- | :---------------------------------- | :--------------- |
| `peer`            | _string_ \| _PeerId_ \| _Multiaddr_ | The peer to dial |
| `protocols`       | _string_ \| _string_[]              |                  |
| `options?`        | _object_                            | -                |
| `options.signal?` | AbortSignal                         | -                |

**Returns:** _Promise_<any\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:170

---

### emit

▸ **emit**(`event`: _string_ \| _symbol_, ...`args`: _any_[]): _boolean_

#### Parameters

| Name      | Type                 |
| :-------- | :------------------- |
| `event`   | _string_ \| _symbol_ |
| `...args` | _any_[]              |

**Returns:** _boolean_

Inherited from: Libp2p_base.emit

Defined in: node_modules/libp2p/dist/src/types.d.ts:84

---

### getMaxListeners

▸ **getMaxListeners**(): _number_

**Returns:** _number_

Inherited from: Libp2p_base.getMaxListeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:81

---

### handle

▸ **handle**(`protocols`: _string_ \| _string_[], `handler`: (`__namedParameters`: { `connection`: _any_ ; `protocol`: _any_ ; `stream`: _any_ }) => _void_): _void_

Registers the `handler` for each protocol

#### Parameters

| Name        | Type                                                                                           |
| :---------- | :--------------------------------------------------------------------------------------------- |
| `protocols` | _string_ \| _string_[]                                                                         |
| `handler`   | (`__namedParameters`: { `connection`: _any_ ; `protocol`: _any_ ; `stream`: _any_ }) => _void_ |

**Returns:** _void_

Defined in: node_modules/libp2p/dist/src/index.d.ts:95

---

### hangUp

▸ **hangUp**(`peer`: _string_ \| _PeerId_ \| _Multiaddr_): _Promise_<void\>

Disconnects all connections to the given `peer`

#### Parameters

| Name   | Type                                | Description                      |
| :----- | :---------------------------------- | :------------------------------- |
| `peer` | _string_ \| _PeerId_ \| _Multiaddr_ | the peer to close connections to |

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:190

---

### isStarted

▸ **isStarted**(): _boolean_

**Returns:** _boolean_

Defined in: node_modules/libp2p/dist/src/index.d.ts:138

---

### listenerCount

▸ **listenerCount**(`event`: _string_ \| _symbol_): _number_

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** _number_

Inherited from: Libp2p_base.listenerCount

Defined in: node_modules/libp2p/dist/src/types.d.ts:85

---

### listeners

▸ **listeners**(`event`: _string_ \| _symbol_): Function[]

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** Function[]

Inherited from: Libp2p_base.listeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:82

---

### loadKeychain

▸ **loadKeychain**(): _Promise_<void\>

Load keychain keys from the datastore.
Imports the private key as 'self', if needed.

**`async`**

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:137

---

### off

▸ **off**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): _any_

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** _any_

Inherited from: Libp2p_base.off

Defined in: node_modules/libp2p/dist/src/types.d.ts:78

---

### on

▸ **on**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): _any_

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** _any_

Inherited from: Libp2p_base.on

Defined in: node_modules/libp2p/dist/src/types.d.ts:75

---

### once

▸ **once**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): _any_

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** _any_

Inherited from: Libp2p_base.once

Defined in: node_modules/libp2p/dist/src/types.d.ts:76

---

### ping

▸ **ping**(`peer`: _string_ \| _PeerId_ \| _Multiaddr_): _Promise_<number\>

Pings the given peer in order to obtain the operation latency.

#### Parameters

| Name   | Type                                | Description      |
| :----- | :---------------------------------- | :--------------- |
| `peer` | _string_ \| _PeerId_ \| _Multiaddr_ | The peer to ping |

**Returns:** _Promise_<number\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:197

---

### rawListeners

▸ **rawListeners**(`event`: _string_ \| _symbol_): Function[]

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** Function[]

Inherited from: Libp2p_base.rawListeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:83

---

### removeAllListeners

▸ **removeAllListeners**(`event?`: _string_ \| _symbol_): _any_

#### Parameters

| Name     | Type                 |
| :------- | :------------------- |
| `event?` | _string_ \| _symbol_ |

**Returns:** _any_

Inherited from: Libp2p_base.removeAllListeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:79

---

### removeListener

▸ **removeListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): _any_

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** _any_

Inherited from: Libp2p_base.removeListener

Defined in: node_modules/libp2p/dist/src/types.d.ts:77

---

### setMaxListeners

▸ **setMaxListeners**(`n`: _number_): _any_

#### Parameters

| Name | Type     |
| :--- | :------- |
| `n`  | _number_ |

**Returns:** _any_

Inherited from: Libp2p_base.setMaxListeners

Defined in: node_modules/libp2p/dist/src/types.d.ts:80

---

### start

▸ **start**(): _Promise_<void\>

Starts the libp2p node and all its subsystems

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:121

---

### stop

▸ **stop**(): _Promise_<void\>

Stop the libp2p node by closing its listeners and open connections

**`async`**

**Returns:** _Promise_<void\>

Defined in: node_modules/libp2p/dist/src/index.d.ts:128

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

Defined in: node_modules/libp2p/dist/src/index.d.ts:204

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

Defined in: node_modules/libp2p/dist/src/index.d.ts:63
