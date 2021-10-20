[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / PassiveStrategy

# Class: PassiveStrategy

## Hierarchy

- [`SaneDefaults`](SaneDefaults.md)

  ↳ **`PassiveStrategy`**

## Implements

- `ChannelStrategy`

## Table of contents

### Constructors

- [constructor](PassiveStrategy.md#constructor)

### Properties

- [name](PassiveStrategy.md#name)
- [tickInterval](PassiveStrategy.md#tickinterval)

### Methods

- [onChannelWillClose](PassiveStrategy.md#onchannelwillclose)
- [onWinningTicket](PassiveStrategy.md#onwinningticket)
- [shouldCommitToChannel](PassiveStrategy.md#shouldcommittochannel)
- [tick](PassiveStrategy.md#tick)

## Constructors

### constructor

• **new PassiveStrategy**()

#### Inherited from

[SaneDefaults](SaneDefaults.md).[constructor](SaneDefaults.md#constructor)

## Properties

### name

• **name**: `string` = `'passive'`

#### Implementation of

ChannelStrategy.name

#### Defined in

[packages/core/src/channel-strategy.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L72)

___

### tickInterval

• **tickInterval**: `number`

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

▸ **tick**(`_balance`, `_c`, `_p`): `Promise`<[[`ChannelsToOpen`](../modules.md#channelstoopen)[], `PublicKey`[]]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_balance` | `BN` |
| `_c` | `ChannelEntry`[] |
| `_p` | `NetworkPeers` |

#### Returns

`Promise`<[[`ChannelsToOpen`](../modules.md#channelstoopen)[], `PublicKey`[]]\>

#### Implementation of

ChannelStrategy.tick

#### Defined in

[packages/core/src/channel-strategy.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L74)
