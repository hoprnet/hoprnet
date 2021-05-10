[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / LibP2P

# Namespace: LibP2P

## Table of contents

### Type aliases

- [Connection](libp2p.md#connection)
- [CreateOptions](libp2p.md#createoptions)
- [Crypto](libp2p.md#crypto)
- [Events](libp2p.md#events)
- [Libp2pConfig](libp2p.md#libp2pconfig)
- [Libp2pModules](libp2p.md#libp2pmodules)
- [Libp2pOptions](libp2p.md#libp2poptions)
- [Multiaddr](libp2p.md#multiaddr)
- [MuxedStream](libp2p.md#muxedstream)
- [MuxerFactory](libp2p.md#muxerfactory)
- [PeerStoreOptions](libp2p.md#peerstoreoptions)
- [Pubsub](libp2p.md#pubsub)
- [RelayOptions](libp2p.md#relayoptions)
- [TransportFactory](libp2p.md#transportfactory)
- [constructorOptions](libp2p.md#constructoroptions)

## Type aliases

### Connection

Ƭ **Connection**: \_\_module

Defined in: node_modules/libp2p/dist/src/index.d.ts:233

___

### CreateOptions

Ƭ **CreateOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `peerId?` | \_\_module |

Defined in: node_modules/libp2p/dist/src/index.d.ts:248

___

### Crypto

Ƭ **Crypto**: Crypto

Defined in: node_modules/libp2p/dist/src/index.d.ts:258

___

### Events

Ƭ **Events**: EventEmitterFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:254

___

### Libp2pConfig

Ƭ **Libp2pConfig**: *object*

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `dht?` | *any* | dht module options |
| `peerDiscovery?` | *any* | - |
| `pubsub?` | \_\_module | pubsub module options |
| `relay?` | [*RelayOptions*](libp2p.md#relayoptions) | - |
| `transport?` | *Record*<string, any\> | transport options indexed by transport key |

Defined in: node_modules/libp2p/dist/src/index.d.ts:269

___

### Libp2pModules

Ƭ **Libp2pModules**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `connEncryption` | [*Crypto*](libp2p.md#crypto)[] |
| `streamMuxer` | [*MuxerFactory*](libp2p.md#muxerfactory)[] |
| `transport` | TransportFactory[] |

Defined in: node_modules/libp2p/dist/src/index.d.ts:285

___

### Libp2pOptions

Ƭ **Libp2pOptions**: *object*

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `addresses?` | AddressManagerOptions | - |
| `config?` | [*Libp2pConfig*](libp2p.md#libp2pconfig) | - |
| `connectionManager?` | ConnectionManagerOptions | - |
| `dialer?` | DialerOptions | - |
| `keychain?` | *any* | - |
| `metrics?` | MetricsOptions | - |
| `modules` | [*Libp2pModules*](libp2p.md#libp2pmodules) | libp2p modules to use |
| `peerStore?` | [*PeerStoreOptions*](libp2p.md#peerstoreoptions) & PersistentPeerStoreOptions | - |
| `transportManager?` | TransportManagerOptions | - |

Defined in: node_modules/libp2p/dist/src/index.d.ts:234

___

### Multiaddr

Ƭ **Multiaddr**: \_\_module

Defined in: node_modules/libp2p/dist/src/index.d.ts:232

___

### MuxedStream

Ƭ **MuxedStream**: MuxedStream

Defined in: node_modules/libp2p/dist/src/index.d.ts:255

___

### MuxerFactory

Ƭ **MuxerFactory**: MuxerFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:257

___

### PeerStoreOptions

Ƭ **PeerStoreOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `persistence` | *boolean* |

Defined in: node_modules/libp2p/dist/src/index.d.ts:260

___

### Pubsub

Ƭ **Pubsub**: \_\_module

Defined in: node_modules/libp2p/dist/src/index.d.ts:259

___

### RelayOptions

Ƭ **RelayOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `advertise` | RelayAdvertiseOptions |
| `autoRelay` | AutoRelayOptions |
| `enabled` | *boolean* |
| `hop` | HopOptions |

Defined in: node_modules/libp2p/dist/src/index.d.ts:263

___

### TransportFactory

Ƭ **TransportFactory**: TransportFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:256

___

### constructorOptions

Ƭ **constructorOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `peerId` | \_\_module |

Defined in: node_modules/libp2p/dist/src/index.d.ts:251
