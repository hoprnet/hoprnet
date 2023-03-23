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

[packages/core/src/channel-strategy.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L75)

___

### tickInterval

• **tickInterval**: `number` = `CHECK_TIMEOUT`

#### Defined in

[packages/core/src/channel-strategy.ts:114](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L114)

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

[packages/core/src/channel-strategy.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L77)

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

[packages/core/src/channel-strategy.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L91)

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

[packages/core/src/channel-strategy.ts:109](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L109)
