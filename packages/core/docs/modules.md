[@hoprnet/hopr-core](README.md) / Exports

# @hoprnet/hopr-core

## Table of contents

### Enumerations

- [NetworkHealthIndicator](enums/NetworkHealthIndicator.md)
- [NetworkPeersOrigin](enums/NetworkPeersOrigin.md)

### Classes

- [PassiveStrategy](classes/PassiveStrategy.md)
- [PromiscuousStrategy](classes/PromiscuousStrategy.md)
- [SaneDefaults](classes/SaneDefaults.md)
- [default](classes/default.md)

### Interfaces

- [ChannelStrategyInterface](interfaces/ChannelStrategyInterface.md)

### Type Aliases

- [HoprOptions](modules.md#hoproptions)
- [NodeStatus](modules.md#nodestatus)
- [ResolvedEnvironment](modules.md#resolvedenvironment)
- [SendMessage](modules.md#sendmessage)
- [StrategyTickResult](modules.md#strategytickresult)
- [Subscribe](modules.md#subscribe)

### Variables

- [ACKNOWLEDGEMENT\_TIMEOUT](modules.md#acknowledgement_timeout)
- [CHECK\_TIMEOUT](modules.md#check_timeout)
- [CONFIRMATIONS](modules.md#confirmations)
- [FULL\_VERSION](modules.md#full_version)
- [HEARTBEAT\_INTERVAL](modules.md#heartbeat_interval)
- [HEARTBEAT\_INTERVAL\_VARIANCE](modules.md#heartbeat_interval_variance)
- [HEARTBEAT\_THRESHOLD](modules.md#heartbeat_threshold)
- [INTERMEDIATE\_HOPS](modules.md#intermediate_hops)
- [MAX\_HOPS](modules.md#max_hops)
- [MAX\_NEW\_CHANNELS\_PER\_TICK](modules.md#max_new_channels_per_tick)
- [MAX\_PACKET\_DELAY](modules.md#max_packet_delay)
- [MAX\_PATH\_ITERATIONS](modules.md#max_path_iterations)
- [NETWORK\_QUALITY\_THRESHOLD](modules.md#network_quality_threshold)
- [PACKET\_SIZE](modules.md#packet_size)
- [PATH\_RANDOMNESS](modules.md#path_randomness)
- [VERSION](modules.md#version)
- [sampleOptions](modules.md#sampleoptions)

### Functions

- [createHoprNode](modules.md#createhoprnode)
- [findPath](modules.md#findpath)
- [resolveEnvironment](modules.md#resolveenvironment)
- [supportedEnvironments](modules.md#supportedenvironments)

## Type Aliases

### HoprOptions

Ƭ **HoprOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowLocalConnections?` | `boolean` |
| `allowPrivateConnections?` | `boolean` |
| `announce?` | `boolean` |
| `connector?` | `HoprCoreEthereum` |
| `createDbIfNotExist?` | `boolean` |
| `dataPath` | `string` |
| `environment` | [`ResolvedEnvironment`](modules.md#resolvedenvironment) |
| `forceCreateDB?` | `boolean` |
| `heartbeatInterval?` | `number` |
| `heartbeatThreshold?` | `number` |
| `heartbeatVariance?` | `number` |
| `hosts?` | { `ip4?`: `NetOptions` ; `ip6?`: `NetOptions`  } |
| `hosts.ip4?` | `NetOptions` |
| `hosts.ip6?` | `NetOptions` |
| `networkQualityThreshold?` | `number` |
| `onChainConfirmations?` | `number` |
| `password?` | `string` |
| `strategy?` | [`ChannelStrategyInterface`](interfaces/ChannelStrategyInterface.md) |
| `testing?` | { `announceLocalAddresses?`: `boolean` ; `mockedDHT?`: `Map`<`string`, `string`[]\> ; `mockedNetwork?`: `Libp2pEmitter`<`any`\> ; `noDirectConnections?`: `boolean` ; `noUPNP?`: `boolean` ; `noWebRTCUpgrade?`: `boolean` ; `preferLocalAddresses?`: `boolean` ; `useMockedLibp2p?`: `boolean`  } |
| `testing.announceLocalAddresses?` | `boolean` |
| `testing.mockedDHT?` | `Map`<`string`, `string`[]\> |
| `testing.mockedNetwork?` | `Libp2pEmitter`<`any`\> |
| `testing.noDirectConnections?` | `boolean` |
| `testing.noUPNP?` | `boolean` |
| `testing.noWebRTCUpgrade?` | `boolean` |
| `testing.preferLocalAddresses?` | `boolean` |
| `testing.useMockedLibp2p?` | `boolean` |

#### Defined in

[packages/core/src/index.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L112)

___

### NodeStatus

Ƭ **NodeStatus**: ``"UNINITIALIZED"`` \| ``"INITIALIZING"`` \| ``"RUNNING"`` \| ``"DESTROYED"``

#### Defined in

[packages/core/src/index.ts:163](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L163)

___

### ResolvedEnvironment

Ƭ **ResolvedEnvironment**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `boost_contract_address` | `string` |
| `channel_contract_deploy_block` | `number` |
| `channels_contract_address` | `string` |
| `environment_type` | `EnvironmentType` |
| `id` | `string` |
| `network` | `NetworkOptions` |
| `network_registry_contract_address` | `string` |
| `network_registry_proxy_contract_address` | `string` |
| `stake_contract_address` | `string` |
| `token_contract_address` | `string` |
| `xhopr_contract_address` | `string` |

#### Defined in

[packages/core/src/environment.ts:54](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L54)

___

### SendMessage

Ƭ **SendMessage**: (`dest`: `PeerId`, `protocols`: `string` \| `string`[], `msg`: `Uint8Array`, `includeReply`: ``true``, `opts?`: `DialOpts`) => `Promise`<`Uint8Array`[]\> & (`dest`: `PeerId`, `protocols`: `string` \| `string`[], `msg`: `Uint8Array`, `includeReply`: ``false``, `opts?`: `DialOpts`) => `Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:178](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L178)

___

### StrategyTickResult

Ƭ **StrategyTickResult**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `toClose` | { `destination`: `PublicKey`  }[] |
| `toOpen` | { `destination`: `PublicKey` ; `stake`: `BN`  }[] |

#### Defined in

[packages/core/src/channel-strategy.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L17)

___

### Subscribe

Ƭ **Subscribe**: (`protocols`: `string` \| `string`[], `handler`: `LibP2PHandlerFunction`<`Promise`<`Uint8Array`\>\>, `includeReply`: ``true``, `errHandler`: (`err`: `any`) => `void`) => `void` & (`protocol`: `string` \| `string`[], `handler`: `LibP2PHandlerFunction`<`Promise`<`void`\> \| `void`\>, `includeReply`: ``false``, `errHandler`: (`err`: `any`) => `void`) => `void`

#### Defined in

[packages/core/src/index.ts:165](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L165)

## Variables

### ACKNOWLEDGEMENT\_TIMEOUT

• `Const` **ACKNOWLEDGEMENT\_TIMEOUT**: ``2000``

#### Defined in

[packages/core/src/constants.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L28)

___

### CHECK\_TIMEOUT

• `Const` **CHECK\_TIMEOUT**: ``60000``

#### Defined in

[packages/core/src/constants.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L27)

___

### CONFIRMATIONS

• `Const` **CONFIRMATIONS**: ``8``

#### Defined in

packages/core-ethereum/lib/constants.d.ts:6

___

### FULL\_VERSION

• `Const` **FULL\_VERSION**: `any` = `pkg.version`

#### Defined in

[packages/core/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L8)

___

### HEARTBEAT\_INTERVAL

• `Const` **HEARTBEAT\_INTERVAL**: ``60000``

#### Defined in

[packages/core/src/constants.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L14)

___

### HEARTBEAT\_INTERVAL\_VARIANCE

• `Const` **HEARTBEAT\_INTERVAL\_VARIANCE**: ``2000``

#### Defined in

[packages/core/src/constants.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L16)

___

### HEARTBEAT\_THRESHOLD

• `Const` **HEARTBEAT\_THRESHOLD**: ``60000``

#### Defined in

[packages/core/src/constants.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L15)

___

### INTERMEDIATE\_HOPS

• `Const` **INTERMEDIATE\_HOPS**: ``3``

#### Defined in

[packages/core/src/constants.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L20)

___

### MAX\_HOPS

• `Const` **MAX\_HOPS**: ``3``

#### Defined in

[packages/core/src/constants.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L25)

___

### MAX\_NEW\_CHANNELS\_PER\_TICK

• `Const` **MAX\_NEW\_CHANNELS\_PER\_TICK**: ``5``

#### Defined in

[packages/core/src/constants.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L24)

___

### MAX\_PACKET\_DELAY

• `Const` **MAX\_PACKET\_DELAY**: ``200``

#### Defined in

[packages/core/src/constants.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L18)

___

### MAX\_PATH\_ITERATIONS

• `Const` **MAX\_PATH\_ITERATIONS**: ``100``

#### Defined in

[packages/core/src/constants.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L22)

___

### NETWORK\_QUALITY\_THRESHOLD

• `Const` **NETWORK\_QUALITY\_THRESHOLD**: ``0.5``

#### Defined in

[packages/core/src/constants.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L23)

___

### PACKET\_SIZE

• `Const` **PACKET\_SIZE**: ``500``

#### Defined in

[packages/core/src/constants.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L7)

___

### PATH\_RANDOMNESS

• `Const` **PATH\_RANDOMNESS**: ``0.1``

#### Defined in

[packages/core/src/constants.ts:21](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L21)

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

[packages/core/src/main.ts:204](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/main.ts#L204)

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

### resolveEnvironment

▸ **resolveEnvironment**(`environment_id`, `customProvider?`): [`ResolvedEnvironment`](modules.md#resolvedenvironment)

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `environment_id` | `string` | environment name |
| `customProvider?` | `string` |  |

#### Returns

[`ResolvedEnvironment`](modules.md#resolvedenvironment)

the environment details, throws if environment is not supported

#### Defined in

[packages/core/src/environment.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L90)

___

### supportedEnvironments

▸ **supportedEnvironments**(): `Environment`[]

#### Returns

`Environment`[]

environments that the given HOPR version should be able to use

#### Defined in

[packages/core/src/environment.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L72)
