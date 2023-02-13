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

[src/types/channelEntry.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L71)

## Properties

### balance

• `Readonly` **balance**: [`Balance`](Balance.md)

#### Defined in

[src/types/channelEntry.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L74)

___

### channelEpoch

• `Readonly` **channelEpoch**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/channelEntry.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L79)

___

### closureTime

• `Readonly` **closureTime**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/channelEntry.ts:80](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L80)

___

### commitment

• `Readonly` **commitment**: [`Hash`](Hash.md)

#### Defined in

[src/types/channelEntry.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L75)

___

### destination

• `Readonly` **destination**: [`PublicKey`](PublicKey.md)

#### Defined in

[src/types/channelEntry.ts:73](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L73)

___

### source

• `Readonly` **source**: [`PublicKey`](PublicKey.md)

#### Defined in

[src/types/channelEntry.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L72)

___

### status

• `Readonly` **status**: [`ChannelStatus`](../enums/ChannelStatus.md)

#### Defined in

[src/types/channelEntry.ts:78](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L78)

___

### ticketEpoch

• `Readonly` **ticketEpoch**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/channelEntry.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L76)

___

### ticketIndex

• `Readonly` **ticketIndex**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/channelEntry.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L77)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[src/types/channelEntry.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L83)

## Methods

### closureTimePassed

▸ **closureTimePassed**(): `boolean`

#### Returns

`boolean`

#### Defined in

[src/types/channelEntry.ts:181](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L181)

___

### getId

▸ **getId**(): [`Hash`](Hash.md)

#### Returns

[`Hash`](Hash.md)

#### Defined in

[src/types/channelEntry.ts:172](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L172)

___

### getRemainingClosureTime

▸ **getRemainingClosureTime**(): `BN`

Computes the remaining time in seconds until the channel can be closed.
Outputs `0` if there is no waiting time, and `-1` if the
closure time of this channel is unknown.

**`Dev`**

used to create more comprehensive debug logs

#### Returns

`BN`

#### Defined in

[src/types/channelEntry.ts:193](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L193)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[src/types/channelEntry.ts:142](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L142)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[src/types/channelEntry.ts:156](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L156)

___

### createMock

▸ `Static` **createMock**(): [`ChannelEntry`](ChannelEntry.md)

#### Returns

[`ChannelEntry`](ChannelEntry.md)

#### Defined in

[src/types/channelEntry.ts:204](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L204)

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

[src/types/channelEntry.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L97)

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

#### Defined in

[src/types/channelEntry.ts:123](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L123)
