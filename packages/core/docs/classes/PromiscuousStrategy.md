[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / PromiscuousStrategy

# Class: PromiscuousStrategy

## Hierarchy

- [`SaneDefaults`](SaneDefaults.md)

  ↳ **`PromiscuousStrategy`**

## Implements

- `ChannelStrategy`

## Table of contents

### Constructors

- [constructor](PromiscuousStrategy.md#constructor)

### Properties

- [name](PromiscuousStrategy.md#name)
- [tickInterval](PromiscuousStrategy.md#tickinterval)

### Methods

- [onChannelWillClose](PromiscuousStrategy.md#onchannelwillclose)
- [onWinningTicket](PromiscuousStrategy.md#onwinningticket)
- [tick](PromiscuousStrategy.md#tick)

## Constructors

### constructor

• **new PromiscuousStrategy**()

#### Inherited from

[SaneDefaults](SaneDefaults.md).[constructor](SaneDefaults.md#constructor)

## Properties

### name

• **name**: `string` = `'promiscuous'`

#### Implementation of

ChannelStrategy.name

#### Defined in

[packages/core/src/channel-strategy.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L74)

___

### tickInterval

• **tickInterval**: `number`

#### Implementation of

ChannelStrategy.tickInterval

#### Inherited from

[SaneDefaults](SaneDefaults.md).[tickInterval](SaneDefaults.md#tickinterval)

#### Defined in

[packages/core/src/channel-strategy.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L60)

## Methods

### onChannelWillClose

▸ **onChannelWillClose**(`c`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `Channel` |

#### Returns

`Promise`<`void`\>

#### Implementation of

ChannelStrategy.onChannelWillClose

#### Inherited from

[SaneDefaults](SaneDefaults.md).[onChannelWillClose](SaneDefaults.md#onchannelwillclose)

#### Defined in

[packages/core/src/channel-strategy.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L55)

___

### onWinningTicket

▸ **onWinningTicket**(`ack`, `c`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ack` | `AcknowledgedTicket` |
| `c` | `Channel` |

#### Returns

`Promise`<`void`\>

#### Implementation of

ChannelStrategy.onWinningTicket

#### Inherited from

[SaneDefaults](SaneDefaults.md).[onWinningTicket](SaneDefaults.md#onwinningticket)

#### Defined in

[packages/core/src/channel-strategy.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L50)

___

### tick

▸ **tick**(`balance`, `currentChannels`, `peers`, `getRandomChannel`): `Promise`<[[`ChannelsToOpen`](../modules.md#channelstoopen)[], `PublicKey`[]]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `balance` | `BN` |
| `currentChannels` | `ChannelEntry`[] |
| `peers` | `NetworkPeers` |
| `getRandomChannel` | () => `Promise`<`ChannelEntry`\> |

#### Returns

`Promise`<[[`ChannelsToOpen`](../modules.md#channelstoopen)[], `PublicKey`[]]\>

#### Implementation of

ChannelStrategy.tick

#### Defined in

[packages/core/src/channel-strategy.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L76)
