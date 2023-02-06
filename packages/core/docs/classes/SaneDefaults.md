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

[packages/core/src/channel-strategy.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L99)

## Methods

### onChannelWillClose

▸ **onChannelWillClose**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | `ChannelEntry` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:80](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L80)

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

[packages/core/src/channel-strategy.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L94)
