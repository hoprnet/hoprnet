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

[packages/core/src/channel-strategy.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L71)

___

### tickInterval

• **tickInterval**: `number`

#### Implementation of

ChannelStrategy.tickInterval

#### Inherited from

[SaneDefaults](SaneDefaults.md).[tickInterval](SaneDefaults.md#tickinterval)

#### Defined in

[packages/core/src/channel-strategy.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L66)

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

[packages/core/src/channel-strategy.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L56)

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

[packages/core/src/channel-strategy.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L51)

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

[packages/core/src/channel-strategy.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L61)

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

[packages/core/src/channel-strategy.ts:73](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L73)
