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
- [shouldCommitToChannel](PromiscuousStrategy.md#shouldcommittochannel)
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

[packages/core/src/channel-strategy.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L81)

___

### tickInterval

• **tickInterval**: `number` = `CHECK_TIMEOUT`

#### Implementation of

ChannelStrategy.tickInterval

#### Inherited from

[SaneDefaults](SaneDefaults.md).[tickInterval](SaneDefaults.md#tickinterval)

#### Defined in

[packages/core/src/channel-strategy.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L67)

## Methods

### onChannelWillClose

▸ **onChannelWillClose**(`_c`, `chain`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_c` | `ChannelEntry` |
| `chain` | `default` |

#### Returns

`Promise`<`void`\>

#### Implementation of

ChannelStrategy.onChannelWillClose

#### Inherited from

[SaneDefaults](SaneDefaults.md).[onChannelWillClose](SaneDefaults.md#onchannelwillclose)

#### Defined in

[packages/core/src/channel-strategy.ts:57](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L57)

___

### onWinningTicket

▸ **onWinningTicket**(`_a`, `chain`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_a` | `AcknowledgedTicket` |
| `chain` | `default` |

#### Returns

`Promise`<`void`\>

#### Implementation of

ChannelStrategy.onWinningTicket

#### Inherited from

[SaneDefaults](SaneDefaults.md).[onWinningTicket](SaneDefaults.md#onwinningticket)

#### Defined in

[packages/core/src/channel-strategy.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L52)

___

### shouldCommitToChannel

▸ **shouldCommitToChannel**(`_c`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_c` | `ChannelEntry` |

#### Returns

`Promise`<`boolean`\>

#### Implementation of

ChannelStrategy.shouldCommitToChannel

#### Inherited from

[SaneDefaults](SaneDefaults.md).[shouldCommitToChannel](SaneDefaults.md#shouldcommittochannel)

#### Defined in

[packages/core/src/channel-strategy.ts:62](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L62)

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

[packages/core/src/channel-strategy.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L83)
