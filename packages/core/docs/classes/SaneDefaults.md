[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / SaneDefaults

# Class: SaneDefaults

## Hierarchy

- **`SaneDefaults`**

  ↳ [`PassiveStrategy`](PassiveStrategy.md)

  ↳ [`PromiscuousStrategy`](PromiscuousStrategy.md)

## Table of contents

### Constructors

- [constructor](SaneDefaults.md#constructor)

### Properties

- [tickInterval](SaneDefaults.md#tickinterval)

### Methods

- [onChannelWillClose](SaneDefaults.md#onchannelwillclose)
- [onWinningTicket](SaneDefaults.md#onwinningticket)
- [shouldCommitToChannel](SaneDefaults.md#shouldcommittochannel)

## Constructors

### constructor

• **new SaneDefaults**()

## Properties

### tickInterval

• **tickInterval**: `number` = `CHECK_TIMEOUT`

#### Defined in

[packages/core/src/channel-strategy.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L76)

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

#### Defined in

[packages/core/src/channel-strategy.ts:58](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L58)

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

#### Defined in

[packages/core/src/channel-strategy.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L52)

___

### shouldCommitToChannel

▸ **shouldCommitToChannel**(`c`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `ChannelEntry` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

[packages/core/src/channel-strategy.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L71)
