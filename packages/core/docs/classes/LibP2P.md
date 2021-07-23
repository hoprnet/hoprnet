[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / LibP2P

# Class: LibP2P

## Table of contents

### Constructors

- [constructor](LibP2P.md#constructor)

### Properties

- [\_dht](LibP2P.md#_dht)
- [addressManager](LibP2P.md#addressmanager)
- [connectionManager](LibP2P.md#connectionmanager)
- [dial](LibP2P.md#dial)
- [dialProtocol](LibP2P.md#dialprotocol)
- [dialer](LibP2P.md#dialer)
- [emit](LibP2P.md#emit)
- [handle](LibP2P.md#handle)
- [hangUp](LibP2P.md#hangup)
- [multiaddrs](LibP2P.md#multiaddrs)
- [peerId](LibP2P.md#peerid)
- [peerRouting](LibP2P.md#peerrouting)
- [peerStore](LibP2P.md#peerstore)
- [registrar](LibP2P.md#registrar)
- [transportManager](LibP2P.md#transportmanager)

### Methods

- [isStarted](LibP2P.md#isstarted)
- [start](LibP2P.md#start)
- [stop](LibP2P.md#stop)
- [create](LibP2P.md#create)

## Constructors

### constructor

• **new LibP2P**(`options`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `options` | `any` |

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:110

## Properties

### \_dht

• **\_dht**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `peerRouting` | `PeerRouting` |

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:136

___

### addressManager

• **addressManager**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `getListenAddrs` | () => `Multiaddr`[] |

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:112

___

### connectionManager

• **connectionManager**: `ConnectionManager`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:134

___

### dial

• **dial**: (`addr`: `PeerId` \| `Multiaddr`, `options?`: { `signal`: `AbortSignal`  }) => `Promise`<`Handler`\>

#### Type declaration

▸ (`addr`, `options?`): `Promise`<`Handler`\>

##### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `PeerId` \| `Multiaddr` |
| `options?` | `Object` |
| `options.signal` | `AbortSignal` |

##### Returns

`Promise`<`Handler`\>

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:120

___

### dialProtocol

• **dialProtocol**: (`addr`: `PeerId` \| `Multiaddr`, `protocol`: `string`, `options?`: { `signal`: `AbortSignal`  }) => `Promise`<`Handler`\>

#### Type declaration

▸ (`addr`, `protocol`, `options?`): `Promise`<`Handler`\>

##### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `PeerId` \| `Multiaddr` |
| `protocol` | `string` |
| `options?` | `Object` |
| `options.signal` | `AbortSignal` |

##### Returns

`Promise`<`Handler`\>

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:124

___

### dialer

• **dialer**: `Dialer`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:123

___

### emit

• **emit**: (`event`: `string`, ...`args`: `any`[]) => `void`

#### Type declaration

▸ (`event`, ...`args`): `void`

##### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` |
| `...args` | `any`[] |

##### Returns

`void`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:119

___

### handle

• **handle**: (`protocol`: `string` \| `string`[], `handler`: (`struct`: `Handler`) => `void`) => `void`

#### Type declaration

▸ (`protocol`, `handler`): `void`

##### Parameters

| Name | Type |
| :------ | :------ |
| `protocol` | `string` \| `string`[] |
| `handler` | (`struct`: `Handler`) => `void` |

##### Returns

`void`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:130

___

### hangUp

• **hangUp**: (`addr`: `string` \| `PeerId` \| `Multiaddr`) => `Promise`<`void`\>

#### Type declaration

▸ (`addr`): `Promise`<`void`\>

##### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `string` \| `PeerId` \| `Multiaddr` |

##### Returns

`Promise`<`void`\>

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:127

___

### multiaddrs

• **multiaddrs**: `Multiaddr`[]

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:133

___

### peerId

• **peerId**: `PeerId`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:139

___

### peerRouting

• **peerRouting**: `PeerRouting`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:129

___

### peerStore

• **peerStore**: `PeerStore`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:128

___

### registrar

• **registrar**: `Registrar`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:135

___

### transportManager

• **transportManager**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `getAddrs` | () => `Multiaddr`[] |

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:115

## Methods

### isStarted

▸ **isStarted**(): `boolean`

#### Returns

`boolean`

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:118

___

### start

▸ **start**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:131

___

### stop

▸ **stop**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:132

___

### create

▸ `Static` **create**(`options`): `Promise`<[`LibP2P`](LibP2P.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `options` | `any` |

#### Returns

`Promise`<[`LibP2P`](LibP2P.md)\>

#### Defined in

node_modules/@hoprnet/hopr-connect/lib/@types/libp2p.d.ts:111
