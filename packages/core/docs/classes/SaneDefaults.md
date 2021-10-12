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

• **tickInterval**: `number`

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

#### Defined in

[packages/core/src/channel-strategy.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L61)
