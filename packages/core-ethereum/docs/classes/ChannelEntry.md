[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / ChannelEntry

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

packages/utils/lib/types/channelEntry.d.ts:35

## Properties

### balance

• `Readonly` **balance**: `Balance`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:28

___

### channelEpoch

• `Readonly` **channelEpoch**: `UINT256`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:33

___

### closureTime

• `Readonly` **closureTime**: `UINT256`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:34

___

### commitment

• `Readonly` **commitment**: `Hash`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:29

___

### destination

• `Readonly` **destination**: `PublicKey`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:27

___

### source

• `Readonly` **source**: `PublicKey`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:26

___

### status

• `Readonly` **status**: `ChannelStatus`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:32

___

### ticketEpoch

• `Readonly` **ticketEpoch**: `UINT256`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:30

___

### ticketIndex

• `Readonly` **ticketIndex**: `UINT256`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:31

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:36

## Methods

### closureTimePassed

▸ **closureTimePassed**(): `boolean`

#### Returns

`boolean`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:42

___

### getId

▸ **getId**(): `Hash`

#### Returns

`Hash`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:41

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

packages/utils/lib/types/channelEntry.d.ts:49

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:39

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:40

___

### createMock

▸ `Static` **createMock**(): [`ChannelEntry`](ChannelEntry.md)

#### Returns

[`ChannelEntry`](ChannelEntry.md)

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:50

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

packages/utils/lib/types/channelEntry.d.ts:37

___

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`, `keyFor`): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEvent` |
| `keyFor` | (`a`: `Address`) => `Promise`<`PublicKey`\> |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

packages/utils/lib/types/channelEntry.d.ts:38
