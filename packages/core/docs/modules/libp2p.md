[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / LibP2P

# Namespace: LibP2P

## Table of contents

### Type aliases

- [Connection](libp2p.md#connection)
- [ContentRoutingModule](libp2p.md#contentroutingmodule)
- [CreateOptions](libp2p.md#createoptions)
- [Crypto](libp2p.md#crypto)
- [Datastore](libp2p.md#datastore)
- [DhtOptions](libp2p.md#dhtoptions)
- [HandlerProps](libp2p.md#handlerprops)
- [KeychainOptions](libp2p.md#keychainoptions)
- [Libp2pConfig](libp2p.md#libp2pconfig)
- [Libp2pModules](libp2p.md#libp2pmodules)
- [Libp2pOptions](libp2p.md#libp2poptions)
- [MetricsOptions](libp2p.md#metricsoptions)
- [MuxedStream](libp2p.md#muxedstream)
- [MuxerFactory](libp2p.md#muxerfactory)
- [PeerDiscoveryFactory](libp2p.md#peerdiscoveryfactory)
- [PeerRoutingModule](libp2p.md#peerroutingmodule)
- [PeerStoreOptions](libp2p.md#peerstoreoptions)
- [Protector](libp2p.md#protector)
- [Pubsub](libp2p.md#pubsub)
- [PubsubLocalOptions](libp2p.md#pubsublocaloptions)
- [PubsubOptions](libp2p.md#pubsuboptions)
- [RandomWalkOptions](libp2p.md#randomwalkoptions)
- [RelayOptions](libp2p.md#relayoptions)
- [TransportFactory](libp2p.md#transportfactory)
- [constructorOptions](libp2p.md#constructoroptions)

## Type aliases

### Connection

Ƭ **Connection**: \_\_module

Defined in: node_modules/libp2p/dist/src/index.d.ts:485

___

### ContentRoutingModule

Ƭ **ContentRoutingModule**: ContentRouting

Defined in: node_modules/libp2p/dist/src/index.d.ts:492

---

### CreateOptions

Ƭ **CreateOptions**: _object_

#### Type declaration

| Name | Type |
| :------ | :------ |
| `peerId?` | PeerId |

Defined in: node_modules/libp2p/dist/src/index.d.ts:486

---

### Crypto

Ƭ **Crypto**: Crypto

Defined in: node_modules/libp2p/dist/src/index.d.ts:495

___

### Datastore

Ƭ **Datastore**: Datastore

Defined in: node_modules/libp2p/dist/src/index.d.ts:498

___

### DhtOptions

Ƭ **DhtOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `clientMode?` | *boolean* |
| `enabled?` | *boolean* |
| `kBucketSize?` | *number* |
| `randomWalk?` | [*RandomWalkOptions*](libp2p.md#randomwalkoptions) |
| `selectors?` | DhtSelectors |
| `validators?` | DhtValidators |

Defined in: node_modules/libp2p/dist/src/index.d.ts:506

___

### HandlerProps

Ƭ **HandlerProps**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `connection` | [*Connection*](libp2p.md#connection) |
| `protocol` | *string* |
| `stream` | [*MuxedStream*](libp2p.md#muxedstream) |

Defined in: node_modules/libp2p/dist/src/index.d.ts:475

---

### KeychainOptions

Ƭ **KeychainOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `datastore?` | Datastore |

Defined in: node_modules/libp2p/dist/src/index.d.ts:514

---

### Libp2pConfig

Ƭ **Libp2pConfig**: _object_

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `dht?` | [*DhtOptions*](libp2p.md#dhtoptions) | dht module options |
| `nat?` | NatManager.NatManagerOptions | - |
| `peerDiscovery?` | *Record*<string, boolean \| Object\> | - |
| `pubsub?` | [*PubsubLocalOptions*](libp2p.md#pubsublocaloptions) & PubsubOptions | pubsub module options |
| `relay?` | [*RelayOptions*](libp2p.md#relayoptions) | - |
| `transport?` | *Record*<string, Object\> | transport options indexed by transport key |

Defined in: node_modules/libp2p/dist/src/index.d.ts:451

---

### Libp2pModules

Ƭ **Libp2pModules**: _object_

#### Type declaration

| Name | Type |
| :------ | :------ |
| `connEncryption` | [*Crypto*](libp2p.md#crypto)[] |
| `connProtector?` | \_\_module |
| `contentRouting?` | ContentRouting[] |
| `dht?` | Object |
| `peerDiscovery?` | PeerDiscoveryFactory[] |
| `peerRouting?` | PeerRouting[] |
| `pubsub?` | (...`args`: *any*[]) => [*Pubsub*](libp2p.md#pubsub) |
| `streamMuxer` | [*MuxerFactory*](libp2p.md#muxerfactory)[] |
| `transport` | TransportFactory[] |

Defined in: node_modules/libp2p/dist/src/index.d.ts:440

---

### Libp2pOptions

Ƭ **Libp2pOptions**: _object_

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `addresses?` | AddressManager.AddressManagerOptions | - |
| `config?` | [*Libp2pConfig*](libp2p.md#libp2pconfig) | - |
| `connectionManager?` | ConnectionManager.ConnectionManagerOptions | - |
| `datastore?` | Datastore | - |
| `dialer?` | Dialer.DialerOptions | - |
| `host?` | IdentifyService.HostProperties | libp2p host |
| `keychain?` | [*KeychainOptions*](libp2p.md#keychainoptions) & Keychain.KeychainOptions | - |
| `metrics?` | [*MetricsOptions*](libp2p.md#metricsoptions) & Metrics.MetricsOptions | - |
| `modules` | [*Libp2pModules*](libp2p.md#libp2pmodules) | libp2p modules to use |
| `peerRouting?` | PeerRouting.PeerRoutingOptions | - |
| `peerStore?` | [*PeerStoreOptions*](libp2p.md#peerstoreoptions) & PersistentPeerStore.PersistentPeerStoreOptions | - |
| `transportManager?` | TransportManager.TransportManagerOptions | - |

Defined in: node_modules/libp2p/dist/src/index.d.ts:414

---

### MetricsOptions

Ƭ **MetricsOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `enabled` | *boolean* |

Defined in: node_modules/libp2p/dist/src/index.d.ts:523

---

### MuxedStream

Ƭ **MuxedStream**: MuxedStream

Defined in: node_modules/libp2p/dist/src/index.d.ts:489

---

### MuxerFactory

Ƭ **MuxerFactory**: MuxerFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:491

___

### PeerDiscoveryFactory

Ƭ **PeerDiscoveryFactory**: PeerDiscoveryFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:493

___

### PeerRoutingModule

Ƭ **PeerRoutingModule**: PeerRouting

Defined in: node_modules/libp2p/dist/src/index.d.ts:494

---

### PeerStoreOptions

Ƭ **PeerStoreOptions**: _object_

#### Type declaration

| Name          | Type      |
| :------------ | :-------- |
| `persistence` | _boolean_ |

Defined in: node_modules/libp2p/dist/src/index.d.ts:517

___

### Protector

Ƭ **Protector**: \_\_module

Defined in: node_modules/libp2p/dist/src/index.d.ts:499

---

### Pubsub

Ƭ **Pubsub**: \_\_module

Defined in: node_modules/libp2p/dist/src/index.d.ts:496

___

### PubsubLocalOptions

Ƭ **PubsubLocalOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `enabled` | *boolean* |

Defined in: node_modules/libp2p/dist/src/index.d.ts:520

___

### PubsubOptions

Ƭ **PubsubOptions**: PubsubOptions

Defined in: node_modules/libp2p/dist/src/index.d.ts:497

___

### RandomWalkOptions

Ƭ **RandomWalkOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `enabled?` | *boolean* |
| `interval?` | *number* |
| `queriesPerPeriod?` | *number* |
| `timeout?` | *number* |

Defined in: node_modules/libp2p/dist/src/index.d.ts:500

---

### RelayOptions

Ƭ **RelayOptions**: _object_

#### Type declaration

| Name | Type |
| :------ | :------ |
| `advertise?` | Relay.RelayAdvertiseOptions |
| `autoRelay?` | Relay.AutoRelayOptions |
| `enabled?` | *boolean* |
| `hop?` | Relay.HopOptions |

Defined in: node_modules/libp2p/dist/src/index.d.ts:526

---

### TransportFactory

Ƭ **TransportFactory**: TransportFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:490

---

### constructorOptions

Ƭ **constructorOptions**: _object_

#### Type declaration

| Name | Type |
| :------ | :------ |
| `peerId` | PeerId |

Defined in: node_modules/libp2p/dist/src/index.d.ts:434
