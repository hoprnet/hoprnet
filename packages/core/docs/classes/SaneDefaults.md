[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / SaneDefaults

# Class: SaneDefaults

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

[packages/core/src/channel-strategy.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L103)

## Methods

### onChannelWillClose

▸ **onChannelWillClose**(`channel`): `Promise`<`void`\>

When an incoming channel is going to be closed, auto redeem tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `channel` | `ChannelEntry` | channel that will be closed |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:84](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L84)

___

### onWinningTicket

▸ **onWinningTicket**(`ackTicket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | `AcknowledgedTicket` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L74)

___

### shouldCommitToChannel

▸ **shouldCommitToChannel**(`c`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `ChannelEntry` |

#### Returns

`boolean`

#### Defined in

[packages/core/src/channel-strategy.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L98)
