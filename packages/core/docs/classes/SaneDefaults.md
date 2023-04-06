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

• `Protected` **autoRedeemTickets**: `boolean` = `false`

#### Defined in

[packages/core/src/channel-strategy.ts:73](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L73)

___

### tickInterval

• **tickInterval**: `number` = `CHECK_TIMEOUT`

#### Defined in

[packages/core/src/channel-strategy.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L112)

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

[packages/core/src/channel-strategy.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L75)

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

[packages/core/src/channel-strategy.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L89)

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

[packages/core/src/channel-strategy.ts:107](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L107)
