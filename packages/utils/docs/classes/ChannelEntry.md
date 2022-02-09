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

#### Defined in

[types/channelEntry.ts:70](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L70)

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

[types/channelEntry.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L82)

## Methods

### closureTimePassed

▸ **closureTimePassed**(): `boolean`

#### Returns

`boolean`

#### Defined in

[types/channelEntry.ts:152](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L152)

___

### getId

▸ **getId**(): [`Hash`](Hash.md)

#### Returns

[`Hash`](Hash.md)

#### Defined in

[types/channelEntry.ts:143](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L143)

___

### getRemainingClosureTime

▸ **getRemainingClosureTime**(): `BN`

Computes the remaining time in seconds until the channel can be closed.
Outputs `0` if there is no waiting time, and `-1` if the
closure time of this channel is unknown.

**`dev`** used to create more comprehensive debug logs

#### Returns

`BN`

#### Defined in

[types/channelEntry.ts:164](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L164)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/channelEntry.ts:113](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L113)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[types/channelEntry.ts:127](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L127)

___

### createMock

▸ `Static` **createMock**(): [`ChannelEntry`](ChannelEntry.md)

#### Returns

[`ChannelEntry`](ChannelEntry.md)

#### Defined in

[types/channelEntry.ts:175](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L175)

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

[types/channelEntry.ts:86](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L86)

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

[types/channelEntry.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L97)
