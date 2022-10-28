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

- [closureTimePassed](ChannelEntry.md#closuretimepassed)
- [getId](ChannelEntry.md#getid)
- [getRemainingClosureTime](ChannelEntry.md#getremainingclosuretime)
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

## Properties

### balance

• `Readonly` **balance**: [`Balance`](Balance.md)

#### Defined in

[src/types/channelEntry.ts:78](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L78)

___

### channelEpoch

• `Readonly` **channelEpoch**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/channelEntry.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L83)

___

### closureTime

• `Readonly` **closureTime**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/channelEntry.ts:84](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L84)

___

### commitment

• `Readonly` **commitment**: [`Hash`](Hash.md)

#### Defined in

[src/types/channelEntry.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L79)

___

### destination

• `Readonly` **destination**: [`PublicKey`](PublicKey.md)

#### Defined in

[src/types/channelEntry.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L77)

___

### source

• `Readonly` **source**: [`PublicKey`](PublicKey.md)

#### Defined in

[src/types/channelEntry.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L76)

___

### status

• `Readonly` **status**: [`ChannelStatus`](../enums/ChannelStatus.md)

#### Defined in

[src/types/channelEntry.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L82)

___

### ticketEpoch

• `Readonly` **ticketEpoch**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/channelEntry.ts:80](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L80)

___

### ticketIndex

• `Readonly` **ticketIndex**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/channelEntry.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L81)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

## Methods

### closureTimePassed

▸ **closureTimePassed**(): `boolean`

#### Returns

`boolean`

___

### getId

▸ **getId**(): [`Hash`](Hash.md)

#### Returns

[`Hash`](Hash.md)

___

### getRemainingClosureTime

▸ **getRemainingClosureTime**(): `BN`

Computes the remaining time in seconds until the channel can be closed.
Outputs `0` if there is no waiting time, and `-1` if the
closure time of this channel is unknown.

**`dev`** used to create more comprehensive debug logs

#### Returns

`BN`

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

___

### createMock

▸ `Static` **createMock**(): [`ChannelEntry`](ChannelEntry.md)

#### Returns

[`ChannelEntry`](ChannelEntry.md)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`ChannelEntry`](ChannelEntry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`ChannelEntry`](ChannelEntry.md)

___

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`, `keyFor`): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEvent` |
| `keyFor` | (`a`: [`Address`](Address.md)) => `Promise`<[`PublicKey`](PublicKey.md)\> |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>
