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

---

### CreateOptions

Ƭ **CreateOptions**: _object_

#### Type declaration

| Name      | Type       |
| :-------- | :--------- |
| `peerId?` | \_\_module |

Defined in: node_modules/libp2p/dist/src/index.d.ts:248

---

### Crypto

Ƭ **Crypto**: Crypto

Defined in: node_modules/libp2p/dist/src/index.d.ts:258

---

### Events

Ƭ **Events**: EventEmitterFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:254

---

### Libp2pConfig

Ƭ **Libp2pConfig**: _object_

#### Type declaration

| Name             | Type                                     | Description                                |
| :--------------- | :--------------------------------------- | :----------------------------------------- |
| `dht?`           | _any_                                    | dht module options                         |
| `peerDiscovery?` | _any_                                    | -                                          |
| `pubsub?`        | \_\_module                               | pubsub module options                      |
| `relay?`         | [_RelayOptions_](libp2p.md#relayoptions) | -                                          |
| `transport?`     | _Record_<string, any\>                   | transport options indexed by transport key |

Defined in: node_modules/libp2p/dist/src/index.d.ts:269

---

### Libp2pModules

Ƭ **Libp2pModules**: _object_

#### Type declaration

| Name             | Type                                       |
| :--------------- | :----------------------------------------- |
| `connEncryption` | [_Crypto_](libp2p.md#crypto)[]             |
| `streamMuxer`    | [_MuxerFactory_](libp2p.md#muxerfactory)[] |
| `transport`      | TransportFactory[]                         |

Defined in: node_modules/libp2p/dist/src/index.d.ts:285

---

### Libp2pOptions

Ƭ **Libp2pOptions**: _object_

#### Type declaration

| Name                 | Type                                                                          | Description           |
| :------------------- | :---------------------------------------------------------------------------- | :-------------------- |
| `addresses?`         | AddressManagerOptions                                                         | -                     |
| `config?`            | [_Libp2pConfig_](libp2p.md#libp2pconfig)                                      | -                     |
| `connectionManager?` | ConnectionManagerOptions                                                      | -                     |
| `dialer?`            | DialerOptions                                                                 | -                     |
| `keychain?`          | _any_                                                                         | -                     |
| `metrics?`           | MetricsOptions                                                                | -                     |
| `modules`            | [_Libp2pModules_](libp2p.md#libp2pmodules)                                    | libp2p modules to use |
| `peerStore?`         | [_PeerStoreOptions_](libp2p.md#peerstoreoptions) & PersistentPeerStoreOptions | -                     |
| `transportManager?`  | TransportManagerOptions                                                       | -                     |

Defined in: node_modules/libp2p/dist/src/index.d.ts:234

---

### Multiaddr

Ƭ **Multiaddr**: \_\_module

Defined in: node_modules/libp2p/dist/src/index.d.ts:232

---

### MuxedStream

Ƭ **MuxedStream**: MuxedStream

Defined in: node_modules/libp2p/dist/src/index.d.ts:255

---

### MuxerFactory

Ƭ **MuxerFactory**: MuxerFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:257

---

### PeerStoreOptions

Ƭ **PeerStoreOptions**: _object_

#### Type declaration

| Name          | Type      |
| :------------ | :-------- |
| `persistence` | _boolean_ |

Defined in: node_modules/libp2p/dist/src/index.d.ts:260

---

### Pubsub

Ƭ **Pubsub**: \_\_module

Defined in: node_modules/libp2p/dist/src/index.d.ts:259

---

### RelayOptions

Ƭ **RelayOptions**: _object_

#### Type declaration

| Name        | Type                  |
| :---------- | :-------------------- |
| `advertise` | RelayAdvertiseOptions |
| `autoRelay` | AutoRelayOptions      |
| `enabled`   | _boolean_             |
| `hop`       | HopOptions            |

Defined in: node_modules/libp2p/dist/src/index.d.ts:263

---

### TransportFactory

Ƭ **TransportFactory**: TransportFactory

Defined in: node_modules/libp2p/dist/src/index.d.ts:256

---

### constructorOptions

Ƭ **constructorOptions**: _object_

#### Type declaration

| Name     | Type       |
| :------- | :--------- |
| `peerId` | \_\_module |

Defined in: node_modules/libp2p/dist/src/index.d.ts:251
