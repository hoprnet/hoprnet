[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / LibP2P

# Namespace: LibP2P

## Table of contents

### Type aliases

- [Connection](LibP2P.md#connection)
- [ContentRoutingModule](LibP2P.md#contentroutingmodule)
- [CreateOptions](LibP2P.md#createoptions)
- [Crypto](LibP2P.md#crypto)
- [Datastore](LibP2P.md#datastore)
- [DhtOptions](LibP2P.md#dhtoptions)
- [HandlerProps](LibP2P.md#handlerprops)
- [KeychainOptions](LibP2P.md#keychainoptions)
- [Libp2pConfig](LibP2P.md#libp2pconfig)
- [Libp2pModules](LibP2P.md#libp2pmodules)
- [Libp2pOptions](LibP2P.md#libp2poptions)
- [MetricsOptions](LibP2P.md#metricsoptions)
- [MuxedStream](LibP2P.md#muxedstream)
- [MuxerFactory](LibP2P.md#muxerfactory)
- [PeerDiscoveryFactory](LibP2P.md#peerdiscoveryfactory)
- [PeerRoutingModule](LibP2P.md#peerroutingmodule)
- [PeerStoreOptions](LibP2P.md#peerstoreoptions)
- [Protector](LibP2P.md#protector)
- [Pubsub](LibP2P.md#pubsub)
- [PubsubLocalOptions](LibP2P.md#pubsublocaloptions)
- [PubsubOptions](LibP2P.md#pubsuboptions)
- [RandomWalkOptions](LibP2P.md#randomwalkoptions)
- [RelayOptions](LibP2P.md#relayoptions)
- [TransportFactory](LibP2P.md#transportfactory)
- [constructorOptions](LibP2P.md#constructoroptions)

## Type aliases

### Connection

Ƭ **Connection**: `__module`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:485

___

### ContentRoutingModule

Ƭ **ContentRoutingModule**: `ContentRouting`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:492

___

### CreateOptions

Ƭ **CreateOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `peerId?` | `PeerId` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:486

___

### Crypto

Ƭ **Crypto**: `Crypto`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:495

___

### Datastore

Ƭ **Datastore**: `Datastore`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:498

___

### DhtOptions

Ƭ **DhtOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `clientMode?` | `boolean` |
| `enabled?` | `boolean` |
| `kBucketSize?` | `number` |
| `randomWalk?` | [`RandomWalkOptions`](LibP2P.md#randomwalkoptions) |
| `selectors?` | `DhtSelectors` |
| `validators?` | `DhtValidators` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:506

___

### HandlerProps

Ƭ **HandlerProps**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `connection` | [`Connection`](LibP2P.md#connection) |
| `protocol` | `string` |
| `stream` | [`MuxedStream`](LibP2P.md#muxedstream) |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:475

___

### KeychainOptions

Ƭ **KeychainOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `datastore?` | `Datastore` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:514

___

### Libp2pConfig

Ƭ **Libp2pConfig**: `Object`

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `dht?` | [`DhtOptions`](LibP2P.md#dhtoptions) | dht module options |
| `nat?` | `NatManager.NatManagerOptions` | - |
| `peerDiscovery?` | `Record`<`string`, `boolean` \| `Object`\> | - |
| `pubsub?` | [`PubsubLocalOptions`](LibP2P.md#pubsublocaloptions) & `PubsubOptions` | pubsub module options |
| `relay?` | [`RelayOptions`](LibP2P.md#relayoptions) | - |
| `transport?` | `Record`<`string`, `Object`\> | transport options indexed by transport key |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:451

___

### Libp2pModules

Ƭ **Libp2pModules**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `connEncryption` | [`Crypto`](LibP2P.md#crypto)[] |
| `connProtector?` | `__module` |
| `contentRouting?` | `ContentRouting`[] |
| `dht?` | `Object` |
| `peerDiscovery?` | `PeerDiscoveryFactory`[] |
| `peerRouting?` | `PeerRouting`[] |
| `pubsub?` | (...`args`: `any`[]) => [`Pubsub`](LibP2P.md#pubsub) |
| `streamMuxer` | [`MuxerFactory`](LibP2P.md#muxerfactory)[] |
| `transport` | `TransportFactory`[] |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:440

___

### Libp2pOptions

Ƭ **Libp2pOptions**: `Object`

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `addresses?` | `AddressManager.AddressManagerOptions` | - |
| `config?` | [`Libp2pConfig`](LibP2P.md#libp2pconfig) | - |
| `connectionManager?` | `ConnectionManager.ConnectionManagerOptions` | - |
| `datastore?` | `Datastore` | - |
| `dialer?` | `Dialer.DialerOptions` | - |
| `host?` | `IdentifyService.HostProperties` | libp2p host |
| `keychain?` | [`KeychainOptions`](LibP2P.md#keychainoptions) & `Keychain.KeychainOptions` | - |
| `metrics?` | [`MetricsOptions`](LibP2P.md#metricsoptions) & `Metrics.MetricsOptions` | - |
| `modules` | [`Libp2pModules`](LibP2P.md#libp2pmodules) | libp2p modules to use |
| `peerRouting?` | `PeerRouting.PeerRoutingOptions` | - |
| `peerStore?` | [`PeerStoreOptions`](LibP2P.md#peerstoreoptions) & `PersistentPeerStore.PersistentPeerStoreOptions` | - |
| `transportManager?` | `TransportManager.TransportManagerOptions` | - |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:414

___

### MetricsOptions

Ƭ **MetricsOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `enabled` | `boolean` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:523

___

### MuxedStream

Ƭ **MuxedStream**: `MuxedStream`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:489

___

### MuxerFactory

Ƭ **MuxerFactory**: `MuxerFactory`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:491

___

### PeerDiscoveryFactory

Ƭ **PeerDiscoveryFactory**: `PeerDiscoveryFactory`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:493

___

### PeerRoutingModule

Ƭ **PeerRoutingModule**: `PeerRouting`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:494

___

### PeerStoreOptions

Ƭ **PeerStoreOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `persistence` | `boolean` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:517

___

### Protector

Ƭ **Protector**: `__module`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:499

___

### Pubsub

Ƭ **Pubsub**: `__module`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:496

___

### PubsubLocalOptions

Ƭ **PubsubLocalOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `enabled` | `boolean` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:520

___

### PubsubOptions

Ƭ **PubsubOptions**: `PubsubOptions`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:497

___

### RandomWalkOptions

Ƭ **RandomWalkOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `enabled?` | `boolean` |
| `interval?` | `number` |
| `queriesPerPeriod?` | `number` |
| `timeout?` | `number` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:500

___

### RelayOptions

Ƭ **RelayOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `advertise?` | `Relay.RelayAdvertiseOptions` |
| `autoRelay?` | `Relay.AutoRelayOptions` |
| `enabled?` | `boolean` |
| `hop?` | `Relay.HopOptions` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:526

___

### TransportFactory

Ƭ **TransportFactory**: `TransportFactory`

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:490

___

### constructorOptions

Ƭ **constructorOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Defined in

node_modules/libp2p/dist/src/index.d.ts:434
