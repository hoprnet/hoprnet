[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / ChannelEntry

# Class: ChannelEntry

## Table of contents

### Constructors

- [constructor](channelentry.md#constructor)

### Properties

- [balance](channelentry.md#balance)
- [channelEpoch](channelentry.md#channelepoch)
- [closureTime](channelentry.md#closuretime)
- [commitment](channelentry.md#commitment)
- [destination](channelentry.md#destination)
- [source](channelentry.md#source)
- [status](channelentry.md#status)
- [ticketEpoch](channelentry.md#ticketepoch)
- [ticketIndex](channelentry.md#ticketindex)

### Accessors

- [SIZE](channelentry.md#size)

### Methods

- [getId](channelentry.md#getid)
- [serialize](channelentry.md#serialize)
- [toString](channelentry.md#tostring)
- [createMock](channelentry.md#createmock)
- [deserialize](channelentry.md#deserialize)
- [fromSCEvent](channelentry.md#fromscevent)

## Constructors

### constructor

• **new ChannelEntry**(`source`, `destination`, `balance`, `commitment`, `ticketEpoch`, `ticketIndex`, `status`, `channelEpoch`, `closureTime`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | `PublicKey` |
| `destination` | `PublicKey` |
| `balance` | `Balance` |
| `commitment` | `Hash` |
| `ticketEpoch` | `UINT256` |
| `ticketIndex` | `UINT256` |
| `status` | `ChannelStatus` |
| `channelEpoch` | `UINT256` |
| `closureTime` | `UINT256` |

#### Defined in

utils/lib/types/channelEntry.d.ts:19

## Properties

### balance

• `Readonly` **balance**: `Balance`

#### Defined in

utils/lib/types/channelEntry.d.ts:13

___

### channelEpoch

• `Readonly` **channelEpoch**: `UINT256`

#### Defined in

utils/lib/types/channelEntry.d.ts:18

___

### closureTime

• `Readonly` **closureTime**: `UINT256`

#### Defined in

utils/lib/types/channelEntry.d.ts:19

___

### commitment

• `Readonly` **commitment**: `Hash`

#### Defined in

utils/lib/types/channelEntry.d.ts:14

___

### destination

• `Readonly` **destination**: `PublicKey`

#### Defined in

utils/lib/types/channelEntry.d.ts:12

___

### source

• `Readonly` **source**: `PublicKey`

#### Defined in

utils/lib/types/channelEntry.d.ts:11

___

### status

• `Readonly` **status**: `ChannelStatus`

#### Defined in

utils/lib/types/channelEntry.d.ts:17

___

### ticketEpoch

• `Readonly` **ticketEpoch**: `UINT256`

#### Defined in

utils/lib/types/channelEntry.d.ts:15

___

### ticketIndex

• `Readonly` **ticketIndex**: `UINT256`

#### Defined in

utils/lib/types/channelEntry.d.ts:16

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

utils/lib/types/channelEntry.d.ts:21

## Methods

### getId

▸ **getId**(): `Hash`

#### Returns

`Hash`

#### Defined in

utils/lib/types/channelEntry.d.ts:26

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

utils/lib/types/channelEntry.d.ts:24

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

utils/lib/types/channelEntry.d.ts:25

___

### createMock

▸ `Static` **createMock**(): [`ChannelEntry`](channelentry.md)

#### Returns

[`ChannelEntry`](channelentry.md)

#### Defined in

utils/lib/types/channelEntry.d.ts:27

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`ChannelEntry`](channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`ChannelEntry`](channelentry.md)

#### Defined in

utils/lib/types/channelEntry.d.ts:22

___

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`, `keyFor`): `Promise`<[`ChannelEntry`](channelentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `any` |
| `keyFor` | (`a`: `Address`) => `Promise`<`PublicKey`\> |

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)\>

#### Defined in

utils/lib/types/channelEntry.d.ts:23
