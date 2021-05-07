[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [index](index.md) / LibP2P

# Namespace: LibP2P

[index](index.md).LibP2P

## Table of contents

### Type aliases

- [Connection](index.libp2p.md#connection)
- [CreateOptions](index.libp2p.md#createoptions)
- [Crypto](index.libp2p.md#crypto)
- [Events](index.libp2p.md#events)
- [Libp2pConfig](index.libp2p.md#libp2pconfig)
- [Libp2pModules](index.libp2p.md#libp2pmodules)
- [Libp2pOptions](index.libp2p.md#libp2poptions)
- [Multiaddr](index.libp2p.md#multiaddr)
- [MuxedStream](index.libp2p.md#muxedstream)
- [MuxerFactory](index.libp2p.md#muxerfactory)
- [PeerStoreOptions](index.libp2p.md#peerstoreoptions)
- [Pubsub](index.libp2p.md#pubsub)
- [RelayOptions](index.libp2p.md#relayoptions)
- [TransportFactory](index.libp2p.md#transportfactory)
- [constructorOptions](index.libp2p.md#constructoroptions)

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
| `relay?` | [*RelayOptions*](index.libp2p.md#relayoptions) | - |
| `transport?` | *Record*<string, any\> | transport options indexed by transport key |

Defined in: node_modules/libp2p/dist/src/index.d.ts:269

___

### Libp2pModules

Ƭ **Libp2pModules**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `connEncryption` | [*Crypto*](index.libp2p.md#crypto)[] |
| `streamMuxer` | [*MuxerFactory*](index.libp2p.md#muxerfactory)[] |
| `transport` | TransportFactory[] |

Defined in: node_modules/libp2p/dist/src/index.d.ts:285

___

### Libp2pOptions

Ƭ **Libp2pOptions**: *object*

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `addresses?` | AddressManagerOptions | - |
| `config?` | [*Libp2pConfig*](index.libp2p.md#libp2pconfig) | - |
| `connectionManager?` | ConnectionManagerOptions | - |
| `dialer?` | DialerOptions | - |
| `keychain?` | *any* | - |
| `metrics?` | MetricsOptions | - |
| `modules` | [*Libp2pModules*](index.libp2p.md#libp2pmodules) | libp2p modules to use |
| `peerStore?` | [*PeerStoreOptions*](index.libp2p.md#peerstoreoptions) & PersistentPeerStoreOptions | - |
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
