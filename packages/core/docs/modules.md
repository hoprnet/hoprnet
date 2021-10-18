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
- [NodeStatus](modules.md#nodestatus)

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
- [PROTOCOL\_ACKNOWLEDGEMENT](modules.md#protocol_acknowledgement)
- [PROTOCOL\_HEARTBEAT](modules.md#protocol_heartbeat)
- [PROTOCOL\_ONCHAIN\_KEY](modules.md#protocol_onchain_key)
- [PROTOCOL\_PAYMENT\_CHANNEL](modules.md#protocol_payment_channel)
- [PROTOCOL\_STRING](modules.md#protocol_string)
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
| `forceCreateDB?` | `boolean` |
| `hosts?` | `Object` |
| `hosts.ip4?` | `NetOptions` |
| `hosts.ip6?` | `NetOptions` |
| `password?` | `string` |
| `preferLocalAddresses?` | `boolean` |
| `provider` | `string` |
| `strategy?` | `ChannelStrategy` |

#### Defined in

[packages/core/src/index.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L74)

___

### NodeStatus

Ƭ **NodeStatus**: ``"UNINITIALIZED"`` \| ``"INITIALIZING"`` \| ``"RUNNING"`` \| ``"DESTROYED"``

#### Defined in

[packages/core/src/index.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L96)

## Variables

### CHECK\_TIMEOUT

• **CHECK\_TIMEOUT**: ``60000``

#### Defined in

[packages/core/src/constants.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L34)

___

### DEFAULT\_STUN\_PORT

• **DEFAULT\_STUN\_PORT**: ``3478``

#### Defined in

[packages/core/src/constants.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L17)

___

### FULL\_VERSION

• **FULL\_VERSION**: `any` = `pkg.version`

#### Defined in

[packages/core/src/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L6)

___

### HEARTBEAT\_INTERVAL

• **HEARTBEAT\_INTERVAL**: ``3000``

#### Defined in

[packages/core/src/constants.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L19)

___

### HEARTBEAT\_INTERVAL\_VARIANCE

• **HEARTBEAT\_INTERVAL\_VARIANCE**: ``2000``

#### Defined in

[packages/core/src/constants.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L20)

___

### HEARTBEAT\_TIMEOUT

• **HEARTBEAT\_TIMEOUT**: ``4000``

#### Defined in

[packages/core/src/constants.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L24)

___

### INTERMEDIATE\_HOPS

• **INTERMEDIATE\_HOPS**: ``3``

#### Defined in

[packages/core/src/constants.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L28)

___

### MAX\_NEW\_CHANNELS\_PER\_TICK

• **MAX\_NEW\_CHANNELS\_PER\_TICK**: ``5``

#### Defined in

[packages/core/src/constants.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L32)

___

### MAX\_PACKET\_DELAY

• **MAX\_PACKET\_DELAY**: ``200``

#### Defined in

[packages/core/src/constants.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L26)

___

### MAX\_PARALLEL\_CONNECTIONS

• **MAX\_PARALLEL\_CONNECTIONS**: ``5``

#### Defined in

[packages/core/src/constants.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L22)

___

### MAX\_PATH\_ITERATIONS

• **MAX\_PATH\_ITERATIONS**: ``100``

#### Defined in

[packages/core/src/constants.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L30)

___

### NETWORK\_QUALITY\_THRESHOLD

• **NETWORK\_QUALITY\_THRESHOLD**: ``0.5``

#### Defined in

[packages/core/src/constants.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L31)

___

### PACKET\_SIZE

• **PACKET\_SIZE**: ``500``

#### Defined in

[packages/core/src/constants.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L5)

___

### PATH\_RANDOMNESS

• **PATH\_RANDOMNESS**: ``0.1``

#### Defined in

[packages/core/src/constants.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L29)

___

### PROTOCOL\_ACKNOWLEDGEMENT

• **PROTOCOL\_ACKNOWLEDGEMENT**: `string`

#### Defined in

[packages/core/src/constants.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L13)

___

### PROTOCOL\_HEARTBEAT

• **PROTOCOL\_HEARTBEAT**: `string`

#### Defined in

[packages/core/src/constants.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L16)

___

### PROTOCOL\_ONCHAIN\_KEY

• **PROTOCOL\_ONCHAIN\_KEY**: `string`

#### Defined in

[packages/core/src/constants.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L15)

___

### PROTOCOL\_PAYMENT\_CHANNEL

• **PROTOCOL\_PAYMENT\_CHANNEL**: `string`

#### Defined in

[packages/core/src/constants.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L14)

___

### PROTOCOL\_STRING

• **PROTOCOL\_STRING**: `string`

#### Defined in

[packages/core/src/constants.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L12)

___

### VERSION

• **VERSION**: `string`

#### Defined in

[packages/core/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L8)

## Functions

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
