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
- [ResolvedEnvironment](modules.md#resolvedenvironment)
- [SendMessage](modules.md#sendmessage)
- [Subscribe](modules.md#subscribe)

### Variables

- [ACKNOWLEDGEMENT\_TIMEOUT](modules.md#acknowledgement_timeout)
- [CHECK\_TIMEOUT](modules.md#check_timeout)
- [DEFAULT\_STUN\_PORT](modules.md#default_stun_port)
- [FULL\_VERSION](modules.md#full_version)
- [HEARTBEAT\_INTERVAL](modules.md#heartbeat_interval)
- [HEARTBEAT\_INTERVAL\_VARIANCE](modules.md#heartbeat_interval_variance)
- [HEARTBEAT\_TIMEOUT](modules.md#heartbeat_timeout)
- [INTERMEDIATE\_HOPS](modules.md#intermediate_hops)
- [MAX\_HOPS](modules.md#max_hops)
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
- [resolveEnvironment](modules.md#resolveenvironment)
- [supportedEnvironments](modules.md#supportedenvironments)

## Type aliases

### ChannelsToClose

Ƭ **ChannelsToClose**: `PublicKey`

#### Defined in

[packages/core/src/channel-strategy.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L17)

___

### ChannelsToOpen

Ƭ **ChannelsToOpen**: [`PublicKey`, `BN`]

#### Defined in

[packages/core/src/channel-strategy.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L16)

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
| `environment` | [`ResolvedEnvironment`](modules.md#resolvedenvironment) |
| `forceCreateDB?` | `boolean` |
| `hosts?` | `Object` |
| `hosts.ip4?` | `NetOptions` |
| `hosts.ip6?` | `NetOptions` |
| `password?` | `string` |
| `preferLocalAddresses?` | `boolean` |
| `strategy?` | `ChannelStrategy` |

#### Defined in

[packages/core/src/index.ts:80](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L80)

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
| `gasPrice?` | `number` |
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

[packages/core/src/index.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L102)

___

### ProtocolConfig

Ƭ **ProtocolConfig**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `environments` | `Environment`[] |
| `networks` | [`Network`](modules.md#network)[] |

#### Defined in

[packages/core/src/environment.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L23)

___

### ResolvedEnvironment

Ƭ **ResolvedEnvironment**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `channel_contract_deploy_block` | `number` |
| `channels_contract_address` | `string` |
| `id` | `string` |
| `network` | [`Network`](modules.md#network) |
| `token_contract_address` | `string` |

#### Defined in

[packages/core/src/environment.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L28)

___

### SendMessage

Ƭ **SendMessage**: (`dest`: `PeerId`, `protocol`: `string`, `msg`: `Uint8Array`, `includeReply`: ``false``, `opts`: `DialOpts`) => `Promise`<`void`\> & (`dest`: `PeerId`, `protocol`: `string`, `msg`: `Uint8Array`, `includeReply`: ``true``, `opts`: `DialOpts`) => `Promise`<`Uint8Array`[]\>

#### Defined in

[packages/core/src/index.ts:117](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L117)

___

### Subscribe

Ƭ **Subscribe**: (`protocol`: `string`, `handler`: `LibP2PHandlerFunction`<`Promise`<`void`\>\>, `includeReply`: ``false``, `errHandler`: (`err`: `any`) => `void`) => `void` & (`protocol`: `string`, `handler`: `LibP2PHandlerFunction`<`Promise`<`Uint8Array`\>\>, `includeReply`: ``true``, `errHandler`: (`err`: `any`) => `void`) => `void`

#### Defined in

[packages/core/src/index.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L104)

## Variables

### ACKNOWLEDGEMENT\_TIMEOUT

• **ACKNOWLEDGEMENT\_TIMEOUT**: ``2000``

#### Defined in

[packages/core/src/constants.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L29)

___

### CHECK\_TIMEOUT

• **CHECK\_TIMEOUT**: ``60000``

#### Defined in

[packages/core/src/constants.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L28)

___

### DEFAULT\_STUN\_PORT

• **DEFAULT\_STUN\_PORT**: ``3478``

#### Defined in

[packages/core/src/constants.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L10)

___

### FULL\_VERSION

• **FULL\_VERSION**: `any` = `pkg.version`

#### Defined in

[packages/core/src/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L6)

___

### HEARTBEAT\_INTERVAL

• **HEARTBEAT\_INTERVAL**: ``3000``

#### Defined in

[packages/core/src/constants.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L12)

___

### HEARTBEAT\_INTERVAL\_VARIANCE

• **HEARTBEAT\_INTERVAL\_VARIANCE**: ``2000``

#### Defined in

[packages/core/src/constants.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L13)

___

### HEARTBEAT\_TIMEOUT

• **HEARTBEAT\_TIMEOUT**: ``4000``

#### Defined in

[packages/core/src/constants.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L17)

___

### INTERMEDIATE\_HOPS

• **INTERMEDIATE\_HOPS**: ``3``

#### Defined in

[packages/core/src/constants.ts:21](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L21)

___

### MAX\_HOPS

• **MAX\_HOPS**: ``3``

#### Defined in

[packages/core/src/constants.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L26)

___

### MAX\_NEW\_CHANNELS\_PER\_TICK

• **MAX\_NEW\_CHANNELS\_PER\_TICK**: ``5``

#### Defined in

[packages/core/src/constants.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L25)

___

### MAX\_PACKET\_DELAY

• **MAX\_PACKET\_DELAY**: ``200``

#### Defined in

[packages/core/src/constants.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L19)

___

### MAX\_PARALLEL\_CONNECTIONS

• **MAX\_PARALLEL\_CONNECTIONS**: ``5``

#### Defined in

[packages/core/src/constants.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L15)

___

### MAX\_PATH\_ITERATIONS

• **MAX\_PATH\_ITERATIONS**: ``100``

#### Defined in

[packages/core/src/constants.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L23)

___

### NETWORK\_QUALITY\_THRESHOLD

• **NETWORK\_QUALITY\_THRESHOLD**: ``0.5``

#### Defined in

[packages/core/src/constants.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L24)

___

### PACKET\_SIZE

• **PACKET\_SIZE**: ``500``

#### Defined in

[packages/core/src/constants.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L5)

___

### PATH\_RANDOMNESS

• **PATH\_RANDOMNESS**: ``0.1``

#### Defined in

[packages/core/src/constants.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L22)

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

___

### resolveEnvironment

▸ **resolveEnvironment**(`environment_id`): [`ResolvedEnvironment`](modules.md#resolvedenvironment)

#### Parameters

| Name | Type |
| :------ | :------ |
| `environment_id` | `string` |

#### Returns

[`ResolvedEnvironment`](modules.md#resolvedenvironment)

#### Defined in

[packages/core/src/environment.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L42)

___

### supportedEnvironments

▸ **supportedEnvironments**(): `Environment`[]

#### Returns

`Environment`[]

#### Defined in

[packages/core/src/environment.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/environment.ts#L36)
