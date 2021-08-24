[@hoprnet/hopr-core](README.md) / Exports

# @hoprnet/hopr-core

## Table of contents

### Namespaces

- [LibP2P](modules/LibP2P.md)

### Classes

- [LibP2P](classes/LibP2P.md)
- [PassiveStrategy](classes/PassiveStrategy.md)
- [PromiscuousStrategy](classes/PromiscuousStrategy.md)
- [SaneDefaults](classes/SaneDefaults.md)
- [default](classes/default.md)

### Type aliases

- [ChannelsToClose](modules.md#channelstoclose)
- [ChannelsToOpen](modules.md#channelstoopen)
- [HoprOptions](modules.md#hoproptions)
- [Network](modules.md#network)
- [NodeStatus](modules.md#nodestatus)
- [ProtocolConfig](modules.md#protocolconfig)

### Variables

- [CHECK\_TIMEOUT](modules.md#check_timeout)
- [DEFAULT\_STUN\_PORT](modules.md#default_stun_port)
- [FULL\_VERSION](modules.md#full_version)
- [HEARTBEAT\_INTERVAL](modules.md#heartbeat_interval)
- [HEARTBEAT\_INTERVAL\_VARIANCE](modules.md#heartbeat_interval_variance)
- [HEARTBEAT\_TIMEOUT](modules.md#heartbeat_timeout)
- [INTERMEDIATE\_HOPS](modules.md#intermediate_hops)
- [MAX\_NEW\_CHANNELS\_PER\_TICK](modules.md#max_new_channels_per_tick)
- [MAX\_PACKET\_DELAY](modules.md#max_packet_delay)
- [MAX\_PARALLEL\_CONNECTIONS](modules.md#max_parallel_connections)
- [MAX\_PATH\_ITERATIONS](modules.md#max_path_iterations)
- [NETWORK\_QUALITY\_THRESHOLD](modules.md#network_quality_threshold)
- [PACKET\_SIZE](modules.md#packet_size)
- [PATH\_RANDOMNESS](modules.md#path_randomness)
- [VERSION](modules.md#version)

### Functions

- [findPath](modules.md#findpath)

## Type aliases

### ChannelsToClose

Ƭ **ChannelsToClose**: `PublicKey`

#### Defined in

[packages/core/src/channel-strategy.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L16)

___

### ChannelsToOpen

Ƭ **ChannelsToOpen**: [`PublicKey`, `BN`]

#### Defined in

[packages/core/src/channel-strategy.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L15)

___

### HoprOptions

Ƭ **HoprOptions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `announce?` | `boolean` |
| `announceLocalAddresses?` | `boolean` |
| `connector?` | `HoprCoreEthereum` |
| `createDbIfNotExist?` | `boolean` |
| `dbPath?` | `string` |
| `environment` | `ResolvedEnvironment` |
| `forceCreateDB?` | `boolean` |
| `hosts?` | `Object` |
| `hosts.ip4?` | `NetOptions` |
| `hosts.ip6?` | `NetOptions` |
| `password?` | `string` |
| `preferLocalAddresses?` | `boolean` |
| `strategy?` | `ChannelStrategy` |

#### Defined in

[packages/core/src/index.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L76)

___

### Network

Ƭ **Network**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `chain_id` | `number` |
| `default_provider` | `string` |
| `description` | `string` |
| `gas?` | `string` |
| `gas_multiplier` | `number` |
| `hopr_token_name` | `string` |
| `id` | `string` |
| `live` | `boolean` |
| `native_token_name` | `string` |
| `tags` | `string`[] |

#### Defined in

[packages/core/src/environment.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L1)

___

### NodeStatus

Ƭ **NodeStatus**: ``"UNINITIALIZED"`` \| ``"INITIALIZING"`` \| ``"RUNNING"`` \| ``"DESTROYED"``

#### Defined in

[packages/core/src/index.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L98)

___

### ProtocolConfig

Ƭ **ProtocolConfig**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `environments` | `Environment`[] |
| `networks` | [`Network`](modules.md#network)[] |

#### Defined in

[packages/core/src/environment.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L22)

## Variables

### CHECK\_TIMEOUT

• `Const` **CHECK\_TIMEOUT**: ``60000``

#### Defined in

[packages/core/src/constants.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L26)

___

### DEFAULT\_STUN\_PORT

• `Const` **DEFAULT\_STUN\_PORT**: ``3478``

#### Defined in

[packages/core/src/constants.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L9)

___

### FULL\_VERSION

• `Const` **FULL\_VERSION**: `any`

#### Defined in

[packages/core/src/constants.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L4)

___

### HEARTBEAT\_INTERVAL

• `Const` **HEARTBEAT\_INTERVAL**: ``3000``

#### Defined in

[packages/core/src/constants.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L11)

___

### HEARTBEAT\_INTERVAL\_VARIANCE

• `Const` **HEARTBEAT\_INTERVAL\_VARIANCE**: ``2000``

#### Defined in

[packages/core/src/constants.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L12)

___

### HEARTBEAT\_TIMEOUT

• `Const` **HEARTBEAT\_TIMEOUT**: ``4000``

#### Defined in

[packages/core/src/constants.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L16)

___

### INTERMEDIATE\_HOPS

• `Const` **INTERMEDIATE\_HOPS**: ``3``

#### Defined in

[packages/core/src/constants.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L20)

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

### MAX\_PARALLEL\_CONNECTIONS

• `Const` **MAX\_PARALLEL\_CONNECTIONS**: ``5``

#### Defined in

[packages/core/src/constants.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L14)

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

[packages/core/src/constants.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L3)

___

### PATH\_RANDOMNESS

• `Const` **PATH\_RANDOMNESS**: ``0.1``

#### Defined in

[packages/core/src/constants.ts:21](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L21)

___

### VERSION

• `Const` **VERSION**: `string`

#### Defined in

[packages/core/src/constants.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L7)

## Functions

### findPath

▸ **findPath**(`start`, `destination`, `hops`, `networkQualityOf`, `getOpenChannelsFromPeer`, `weight?`): `Promise`<`Path`\>

Find a path through the payment channels.

Depth first search through potential paths based on weight

#### Parameters

| Name | Type |
| :------ | :------ |
| `start` | `PublicKey` |
| `destination` | `PublicKey` |
| `hops` | `number` |
| `networkQualityOf` | (`p`: `PublicKey`) => `number` |
| `getOpenChannelsFromPeer` | (`p`: `PublicKey`) => `Promise`<`ChannelEntry`[]\> |
| `weight` | (`edge`: `ChannelEntry`) => `Promise`<`BN`\> |

#### Returns

`Promise`<`Path`\>

path as Array<PeerId> (including start, but not including
destination

#### Defined in

[packages/core/src/path/index.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/path/index.ts#L38)
