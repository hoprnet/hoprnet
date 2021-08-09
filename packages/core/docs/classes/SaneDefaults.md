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

## Constructors

### constructor

• **new SaneDefaults**()

## Properties

### tickInterval

• **tickInterval**: `number`

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

#### Defined in

[packages/core/src/channel-strategy.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L50)
