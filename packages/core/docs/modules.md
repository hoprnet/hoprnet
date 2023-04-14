[@hoprnet/hopr-core](README.md) / Exports

# @hoprnet/hopr-core

## Table of contents

### Enumerations

- [Health](enums/Health.md)
- [PeerOrigin](enums/PeerOrigin.md)

### Classes

- [PeerStatus](classes/PeerStatus.md)
- [ResolvedEnvironment](classes/ResolvedEnvironment.md)
- [SaneDefaults](classes/SaneDefaults.md)
- [StrategyFactory](classes/StrategyFactory.md)
- [StrategyTickResult](classes/StrategyTickResult.md)
- [default](classes/default.md)

### Interfaces

- [ChannelStrategyInterface](interfaces/ChannelStrategyInterface.md)

### Type Aliases

- [HoprOptions](modules.md#hoproptions)
- [NodeStatus](modules.md#nodestatus)
- [SendMessage](modules.md#sendmessage)
- [Strategy](modules.md#strategy)
- [Subscribe](modules.md#subscribe)

### Variables

- [ACKNOWLEDGEMENT\_TIMEOUT](modules.md#acknowledgement_timeout)
- [CHECK\_TIMEOUT](modules.md#check_timeout)
- [FULL\_VERSION](modules.md#full_version)
- [INTERMEDIATE\_HOPS](modules.md#intermediate_hops)
- [MAX\_HOPS](modules.md#max_hops)
- [MAX\_NEW\_CHANNELS\_PER\_TICK](modules.md#max_new_channels_per_tick)
- [MAX\_PARALLEL\_PINGS](modules.md#max_parallel_pings)
- [MAX\_PATH\_ITERATIONS](modules.md#max_path_iterations)
- [NETWORK\_QUALITY\_THRESHOLD](modules.md#network_quality_threshold)
- [PACKET\_SIZE](modules.md#packet_size)
- [PATH\_RANDOMNESS](modules.md#path_randomness)
- [PEER\_METADATA\_PROTOCOL\_VERSION](modules.md#peer_metadata_protocol_version)
- [PEER\_METADATA\_PROTOCOL\_VERSION](modules.md#peer_metadata_protocol_version)
- [VERSION](modules.md#version)
- [sampleOptions](modules.md#sampleoptions)

### Functions

- [CONSTANTS](modules.md#constants)
- [createHoprNode](modules.md#createhoprnode)
- [findPath](modules.md#findpath)
- [health\_to\_string](modules.md#health_to_string)
- [health\_to\_string](modules.md#health_to_string)
- [isStrategy](modules.md#isstrategy)
- [resolveEnvironment](modules.md#resolveenvironment)
- [supportedEnvironments](modules.md#supportedenvironments)

## Type Aliases

### HoprOptions

Ƭ **HoprOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowLocalConnections` | `boolean` |
| `allowPrivateConnections` | `boolean` |
| `announce` | `boolean` |
| `checkUnrealizedBalance?` | `boolean` |
| `connector?` | `HoprCoreEthereum` |
| `createDbIfNotExist` | `boolean` |
| `dataPath` | `string` |
| `environment` | [`ResolvedEnvironment`](classes/ResolvedEnvironment.md) |
| `forceCreateDB` | `boolean` |
| `heartbeatInterval?` | `number` |
| `heartbeatThreshold?` | `number` |
| `heartbeatVariance?` | `number` |
| `hosts` | { `ip4?`: `NetOptions` ; `ip6?`: `NetOptions`  } |
| `hosts.ip4?` | `NetOptions` |
| `hosts.ip6?` | `NetOptions` |
| `maxParallelConnections?` | `number` |
| `networkQualityThreshold?` | `number` |
| `onChainConfirmations?` | `number` |
| `password` | `string` |
| `strategy` | [`ChannelStrategyInterface`](interfaces/ChannelStrategyInterface.md) |
| `testing?` | { `announceLocalAddresses?`: `boolean` ; `localModeStun?`: `boolean` ; `mockedDHT?`: `Map`<`string`, `string`[]\> ; `mockedNetwork?`: `Libp2pEmitter`<`any`\> ; `noDirectConnections?`: `boolean` ; `noWebRTCUpgrade?`: `boolean` ; `preferLocalAddresses?`: `boolean` ; `useMockedLibp2p?`: `boolean`  } |
| `testing.announceLocalAddresses?` | `boolean` |
| `testing.localModeStun?` | `boolean` |
| `testing.mockedDHT?` | `Map`<`string`, `string`[]\> |
| `testing.mockedNetwork?` | `Libp2pEmitter`<`any`\> |
| `testing.noDirectConnections?` | `boolean` |
| `testing.noWebRTCUpgrade?` | `boolean` |
| `testing.preferLocalAddresses?` | `boolean` |
| `testing.useMockedLibp2p?` | `boolean` |

#### Defined in

[packages/core/src/index.ts:153](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L153)

___

### NodeStatus

Ƭ **NodeStatus**: ``"UNINITIALIZED"`` \| ``"INITIALIZING"`` \| ``"RUNNING"`` \| ``"DESTROYED"``

#### Defined in

[packages/core/src/index.ts:205](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L205)

___

### SendMessage

Ƭ **SendMessage**: (`dest`: `PeerId`, `protocols`: `string` \| `string`[], `msg`: `Uint8Array`, `includeReply`: ``true``, `opts?`: `DialOpts`) => `Promise`<`Uint8Array`[]\> & (`dest`: `PeerId`, `protocols`: `string` \| `string`[], `msg`: `Uint8Array`, `includeReply`: ``false``, `opts?`: `DialOpts`) => `Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:220](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L220)

___

### Strategy

Ƭ **Strategy**: typeof `STRATEGIES`[`number`]

#### Defined in

[packages/core/src/channel-strategy.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L29)

___

### Subscribe

Ƭ **Subscribe**: (`protocols`: `string` \| `string`[], `handler`: `LibP2PHandlerFunction`<`Promise`<`Uint8Array`\>\>, `includeReply`: ``true``, `errHandler`: (`err`: `any`) => `void`) => `void` & (`protocol`: `string` \| `string`[], `handler`: `LibP2PHandlerFunction`<`Promise`<`void`\> \| `void`\>, `includeReply`: ``false``, `errHandler`: (`err`: `any`) => `void`) => `void`

#### Defined in

[packages/core/src/index.ts:207](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L207)

## Variables

### ACKNOWLEDGEMENT\_TIMEOUT

• `Const` **ACKNOWLEDGEMENT\_TIMEOUT**: ``2000``

#### Defined in

[packages/core/src/constants.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L22)

___

### CHECK\_TIMEOUT

• `Const` **CHECK\_TIMEOUT**: ``60000``

#### Defined in

[packages/core/src/constants.ts:21](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L21)

___

### FULL\_VERSION

• `Const` **FULL\_VERSION**: `string` = `pkg.version`

#### Defined in

[packages/core/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L8)

___

### INTERMEDIATE\_HOPS

• `Const` **INTERMEDIATE\_HOPS**: ``3``

#### Defined in

[packages/core/src/constants.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L12)

___

### MAX\_HOPS

• `Const` **MAX\_HOPS**: ``3``

#### Defined in

[packages/core/src/constants.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L17)

___

### MAX\_NEW\_CHANNELS\_PER\_TICK

• `Const` **MAX\_NEW\_CHANNELS\_PER\_TICK**: ``5``

#### Defined in

[packages/core/src/constants.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L16)

___

### MAX\_PARALLEL\_PINGS

• `Const` **MAX\_PARALLEL\_PINGS**: ``14``

#### Defined in

[packages/core/src/constants.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L19)

___

### MAX\_PATH\_ITERATIONS

• `Const` **MAX\_PATH\_ITERATIONS**: ``100``

#### Defined in

[packages/core/src/constants.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L14)

___

### NETWORK\_QUALITY\_THRESHOLD

• `Const` **NETWORK\_QUALITY\_THRESHOLD**: ``0.5``

#### Defined in

[packages/core/src/constants.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L15)

___

### PACKET\_SIZE

• `Const` **PACKET\_SIZE**: ``500``

#### Defined in

[packages/core/src/constants.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L7)

___

### PATH\_RANDOMNESS

• `Const` **PATH\_RANDOMNESS**: ``0.1``

#### Defined in

[packages/core/src/constants.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L13)

___

### PEER\_METADATA\_PROTOCOL\_VERSION

• `Const` **PEER\_METADATA\_PROTOCOL\_VERSION**: ``"protocol_version"``

#### Defined in

[packages/core/src/constants.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L24)

___

### PEER\_METADATA\_PROTOCOL\_VERSION

• `Const` **PEER\_METADATA\_PROTOCOL\_VERSION**: ``"protocol_version"``

#### Defined in

[packages/core/src/constants.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L24)

___

### VERSION

• `Const` **VERSION**: `string`

#### Defined in

[packages/core/src/constants.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L10)

___

### sampleOptions

• `Const` **sampleOptions**: `Partial`<[`HoprOptions`](modules.md#hoproptions)\>

#### Defined in

[packages/core/src/index.mock.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.mock.ts#L3)

## Functions

### CONSTANTS

▸ **CONSTANTS**(): `CoreConstants`

Returns a struct with readonly constants, needs to be a function
because Rust does not support exporting constants to WASM

#### Returns

`CoreConstants`

#### Defined in

packages/core/lib/core_misc.d.ts:23

___

### createHoprNode

▸ **createHoprNode**(`peerId`, `options`, `automaticChainCreation?`): `Promise`<[`default`](classes/default.md)\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `peerId` | `PeerId` | `undefined` |
| `options` | [`HoprOptions`](modules.md#hoproptions) | `undefined` |
| `automaticChainCreation` | `boolean` | `true` |

#### Returns

`Promise`<[`default`](classes/default.md)\>

#### Defined in

[packages/core/src/main.ts:226](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/main.ts#L226)

___

### findPath

▸ **findPath**(`start`, `destination`, `hops`, `networkQualityOf`, `getOpenChannelsFromPeer`, `weight?`): `Promise`<`Path`\>

Find a path through the payment channels.

Depth first search through potential paths based on weight

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `start` | `PublicKey` | `undefined` |
| `destination` | `PublicKey` | `undefined` |
| `hops` | `number` | `undefined` |
| `networkQualityOf` | (`p`: `PublicKey`) => `number` | `undefined` |
| `getOpenChannelsFromPeer` | (`p`: `PublicKey`) => `Promise`<`ChannelEntry`[]\> | `undefined` |
| `weight` | (`edge`: `ChannelEntry`) => `Promise`<`BN`\> | `defaultWeight` |

#### Returns

`Promise`<`Path`\>

path as Array<PeerId> (including start, but not including
destination

#### Defined in

[packages/core/src/path/index.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/path/index.ts#L38)

___

### health\_to\_string

▸ **health_to_string**(`h`): `string`

#### Parameters

| Name | Type |
| :------ | :------ |
| `h` | `number` |

#### Returns

`string`

#### Defined in

packages/core/lib/core_network.d.ts:25

___

### health\_to\_string

▸ **health_to_string**(`h`): `string`

#### Parameters

| Name | Type |
| :------ | :------ |
| `h` | `number` |

#### Returns

`string`

#### Defined in

packages/core/lib/core_network.d.ts:25

___

### isStrategy

▸ **isStrategy**(`str`): str is string

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

str is string

#### Defined in

[packages/core/src/channel-strategy.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L30)
[packages/core/src/channel-strategy.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L30)

___

### resolveEnvironment

▸ **resolveEnvironment**(`environment_id`, `customProvider?`): [`ResolvedEnvironment`](classes/ResolvedEnvironment.md)

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `environment_id` | `string` | environment name |
| `customProvider?` | `string` |  |

#### Returns

[`ResolvedEnvironment`](classes/ResolvedEnvironment.md)

the environment details, throws if environment is not supported

#### Defined in

[packages/core/src/environment.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L45)

___

### supportedEnvironments

▸ **supportedEnvironments**(): `Environment`[]

#### Returns

`Environment`[]

environments that the given HOPR version should be able to use

#### Defined in

[packages/core/src/environment.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L36)
