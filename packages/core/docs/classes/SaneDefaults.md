[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / SaneDefaults

# Class: SaneDefaults

## Table of contents

### Constructors

- [constructor](SaneDefaults.md#constructor)

### Properties

- [autoRedeemTickets](SaneDefaults.md#autoredeemtickets)
- [tickInterval](SaneDefaults.md#tickinterval)

### Methods

- [onAckedTicket](SaneDefaults.md#onackedticket)
- [onChannelWillClose](SaneDefaults.md#onchannelwillclose)
- [shouldCommitToChannel](SaneDefaults.md#shouldcommittochannel)

## Constructors

### constructor

• **new SaneDefaults**()

## Properties

### autoRedeemTickets

• `Protected` **autoRedeemTickets**: `boolean` = `true`

#### Defined in

[packages/core/src/channel-strategy.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L74)

___

### tickInterval

• **tickInterval**: `number` = `CHECK_TIMEOUT`

#### Defined in

[packages/core/src/channel-strategy.ts:113](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L113)

## Methods

### onAckedTicket

▸ **onAckedTicket**(`ackTicket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | `AcknowledgedTicket` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L76)

___

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

[packages/core/src/channel-strategy.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L90)

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

[packages/core/src/channel-strategy.ts:108](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L108)
