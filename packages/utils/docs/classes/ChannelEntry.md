[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / ChannelEntry

# Class: ChannelEntry

## Table of contents

### Constructors

- [constructor](ChannelEntry.md#constructor)

### Properties

- [balance](ChannelEntry.md#balance)
- [channelEpoch](ChannelEntry.md#channelepoch)
- [closureTime](ChannelEntry.md#closuretime)
- [commitment](ChannelEntry.md#commitment)
- [destination](ChannelEntry.md#destination)
- [source](ChannelEntry.md#source)
- [status](ChannelEntry.md#status)
- [ticketEpoch](ChannelEntry.md#ticketepoch)
- [ticketIndex](ChannelEntry.md#ticketindex)

### Accessors

- [SIZE](ChannelEntry.md#size)

### Methods

- [getId](ChannelEntry.md#getid)
- [serialize](ChannelEntry.md#serialize)
- [toString](ChannelEntry.md#tostring)
- [createMock](ChannelEntry.md#createmock)
- [deserialize](ChannelEntry.md#deserialize)
- [fromSCEvent](ChannelEntry.md#fromscevent)

## Constructors

### constructor

• **new ChannelEntry**(`source`, `destination`, `balance`, `commitment`, `ticketEpoch`, `ticketIndex`, `status`, `channelEpoch`, `closureTime`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | [`PublicKey`](PublicKey.md) |
| `destination` | [`PublicKey`](PublicKey.md) |
| `balance` | [`Balance`](Balance.md) |
| `commitment` | [`Hash`](Hash.md) |
| `ticketEpoch` | [`UINT256`](UINT256.md) |
| `ticketIndex` | [`UINT256`](UINT256.md) |
| `status` | [`ChannelStatus`](../enums/ChannelStatus.md) |
| `channelEpoch` | [`UINT256`](UINT256.md) |
| `closureTime` | [`UINT256`](UINT256.md) |

#### Defined in

[types/channelEntry.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L55)

## Properties

### balance

• `Readonly` **balance**: [`Balance`](Balance.md)

___

### channelEpoch

• `Readonly` **channelEpoch**: [`UINT256`](UINT256.md)

___

### closureTime

• `Readonly` **closureTime**: [`UINT256`](UINT256.md)

___

### commitment

• `Readonly` **commitment**: [`Hash`](Hash.md)

___

### destination

• `Readonly` **destination**: [`PublicKey`](PublicKey.md)

___

### source

• `Readonly` **source**: [`PublicKey`](PublicKey.md)

___

### status

• `Readonly` **status**: [`ChannelStatus`](../enums/ChannelStatus.md)

___

### ticketEpoch

• `Readonly` **ticketEpoch**: [`UINT256`](UINT256.md)

___

### ticketIndex

• `Readonly` **ticketIndex**: [`UINT256`](UINT256.md)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/channelEntry.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L67)

## Methods

### getId

▸ **getId**(): [`Hash`](Hash.md)

#### Returns

[`Hash`](Hash.md)

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

▸ `Static` **createMock**(): [`ChannelEntry`](ChannelEntry.md)

#### Returns

[`ChannelEntry`](ChannelEntry.md)

#### Defined in

[types/channelEntry.ts:132](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L132)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`ChannelEntry`](ChannelEntry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`ChannelEntry`](ChannelEntry.md)

#### Defined in

[types/channelEntry.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L71)

___

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`, `keyFor`): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `any` |
| `keyFor` | (`a`: [`Address`](Address.md)) => `Promise`<[`PublicKey`](PublicKey.md)\> |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[types/channelEntry.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L82)
