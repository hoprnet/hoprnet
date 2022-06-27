[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / PassiveStrategy

# Class: PassiveStrategy

## Hierarchy

- [`SaneDefaults`](SaneDefaults.md)

  ↳ **`PassiveStrategy`**

## Implements

- [`ChannelStrategyInterface`](../interfaces/ChannelStrategyInterface.md)

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

[ChannelStrategyInterface](../interfaces/ChannelStrategyInterface.md).[name](../interfaces/ChannelStrategyInterface.md#name)

#### Defined in

[packages/core/src/channel-strategy.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L88)

___

### tickInterval

• **tickInterval**: `number` = `CHECK_TIMEOUT`

#### Implementation of

[ChannelStrategyInterface](../interfaces/ChannelStrategyInterface.md).[tickInterval](../interfaces/ChannelStrategyInterface.md#tickinterval)

#### Inherited from

[SaneDefaults](SaneDefaults.md).[tickInterval](SaneDefaults.md#tickinterval)

#### Defined in

[packages/core/src/channel-strategy.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L83)

## Methods

### onChannelWillClose

▸ **onChannelWillClose**(`channel`, `chain`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | `ChannelEntry` |
| `chain` | `default` |

#### Returns

`Promise`<`void`\>

#### Implementation of

[ChannelStrategyInterface](../interfaces/ChannelStrategyInterface.md).[onChannelWillClose](../interfaces/ChannelStrategyInterface.md#onchannelwillclose)

#### Inherited from

[SaneDefaults](SaneDefaults.md).[onChannelWillClose](SaneDefaults.md#onchannelwillclose)

#### Defined in

[packages/core/src/channel-strategy.ts:65](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L65)

___

### onWinningTicket

▸ **onWinningTicket**(`ackTicket`, `chain`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | `AcknowledgedTicket` |
| `chain` | `default` |

#### Returns

`Promise`<`void`\>

#### Implementation of

[ChannelStrategyInterface](../interfaces/ChannelStrategyInterface.md).[onWinningTicket](../interfaces/ChannelStrategyInterface.md#onwinningticket)

#### Inherited from

[SaneDefaults](SaneDefaults.md).[onWinningTicket](SaneDefaults.md#onwinningticket)

#### Defined in

[packages/core/src/channel-strategy.ts:59](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L59)

___

### shouldCommitToChannel

▸ **shouldCommitToChannel**(`c`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `ChannelEntry` |

#### Returns

`Promise`<`boolean`\>

#### Implementation of

[ChannelStrategyInterface](../interfaces/ChannelStrategyInterface.md).[shouldCommitToChannel](../interfaces/ChannelStrategyInterface.md#shouldcommittochannel)

#### Inherited from

[SaneDefaults](SaneDefaults.md).[shouldCommitToChannel](SaneDefaults.md#shouldcommittochannel)

#### Defined in

[packages/core/src/channel-strategy.ts:78](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L78)

___

### tick

▸ **tick**(`_balance`, `_c`, `_p`): `Promise`<[`StrategyTickResult`](../modules.md#strategytickresult)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_balance` | `BN` |
| `_c` | `ChannelEntry`[] |
| `_p` | `NetworkPeers` |

#### Returns

`Promise`<[`StrategyTickResult`](../modules.md#strategytickresult)\>

#### Implementation of

[ChannelStrategyInterface](../interfaces/ChannelStrategyInterface.md).[tick](../interfaces/ChannelStrategyInterface.md#tick)

#### Defined in

[packages/core/src/channel-strategy.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L90)
