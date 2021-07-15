[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / ChannelEntry

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
| `source` | [`PublicKey`](publickey.md) |
| `destination` | [`PublicKey`](publickey.md) |
| `balance` | [`Balance`](balance.md) |
| `commitment` | [`Hash`](hash.md) |
| `ticketEpoch` | [`UINT256`](uint256.md) |
| `ticketIndex` | [`UINT256`](uint256.md) |
| `status` | [`ChannelStatus`](../enums/channelstatus.md) |
| `channelEpoch` | [`UINT256`](uint256.md) |
| `closureTime` | [`UINT256`](uint256.md) |

#### Defined in

[types/channelEntry.ts:54](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L54)

## Properties

### balance

• `Readonly` **balance**: [`Balance`](balance.md)

___

### channelEpoch

• `Readonly` **channelEpoch**: [`UINT256`](uint256.md)

___

### closureTime

• `Readonly` **closureTime**: [`UINT256`](uint256.md)

___

### commitment

• `Readonly` **commitment**: [`Hash`](hash.md)

___

### destination

• `Readonly` **destination**: [`PublicKey`](publickey.md)

___

### source

• `Readonly` **source**: [`PublicKey`](publickey.md)

___

### status

• `Readonly` **status**: [`ChannelStatus`](../enums/channelstatus.md)

___

### ticketEpoch

• `Readonly` **ticketEpoch**: [`UINT256`](uint256.md)

___

### ticketIndex

• `Readonly` **ticketIndex**: [`UINT256`](uint256.md)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/channelEntry.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L67)

## Methods

### getId

▸ **getId**(): [`Hash`](hash.md)

#### Returns

[`Hash`](hash.md)

#### Defined in

[types/channelEntry.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L128)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/channelEntry.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L98)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[types/channelEntry.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L112)

___

### createMock

▸ `Static` **createMock**(): [`ChannelEntry`](channelentry.md)

#### Returns

[`ChannelEntry`](channelentry.md)

#### Defined in

[types/channelEntry.ts:132](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L132)

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

[types/channelEntry.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L71)

___

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`, `keyFor`): `Promise`<[`ChannelEntry`](channelentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `any` |
| `keyFor` | (`a`: [`Address`](address.md)) => `Promise`<[`PublicKey`](publickey.md)\> |

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)\>

#### Defined in

[types/channelEntry.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L82)
