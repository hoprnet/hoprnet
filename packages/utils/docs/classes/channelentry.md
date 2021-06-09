[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / ChannelEntry

# Class: ChannelEntry

## Table of contents

### Constructors

- [constructor](channelentry.md#constructor)

### Properties

- [channelEpoch](channelentry.md#channelepoch)
- [closureByPartyA](channelentry.md#closurebypartya)
- [closureTime](channelentry.md#closuretime)
- [commitmentPartyA](channelentry.md#commitmentpartya)
- [commitmentPartyB](channelentry.md#commitmentpartyb)
- [partyA](channelentry.md#partya)
- [partyABalance](channelentry.md#partyabalance)
- [partyATicketEpoch](channelentry.md#partyaticketepoch)
- [partyATicketIndex](channelentry.md#partyaticketindex)
- [partyB](channelentry.md#partyb)
- [partyBBalance](channelentry.md#partybbalance)
- [partyBTicketEpoch](channelentry.md#partybticketepoch)
- [partyBTicketIndex](channelentry.md#partybticketindex)
- [status](channelentry.md#status)

### Accessors

- [SIZE](channelentry.md#size)

### Methods

- [commitmentFor](channelentry.md#commitmentfor)
- [getId](channelentry.md#getid)
- [serialize](channelentry.md#serialize)
- [ticketEpochFor](channelentry.md#ticketepochfor)
- [ticketIndexFor](channelentry.md#ticketindexfor)
- [toString](channelentry.md#tostring)
- [totalBalance](channelentry.md#totalbalance)
- [deserialize](channelentry.md#deserialize)
- [fromSCEvent](channelentry.md#fromscevent)

## Constructors

### constructor

• **new ChannelEntry**(`partyA`, `partyB`, `partyABalance`, `partyBBalance`, `commitmentPartyA`, `commitmentPartyB`, `partyATicketEpoch`, `partyBTicketEpoch`, `partyATicketIndex`, `partyBTicketIndex`, `status`, `channelEpoch`, `closureTime`, `closureByPartyA`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `partyA` | [Address](address.md) |
| `partyB` | [Address](address.md) |
| `partyABalance` | [Balance](balance.md) |
| `partyBBalance` | [Balance](balance.md) |
| `commitmentPartyA` | [Hash](hash.md) |
| `commitmentPartyB` | [Hash](hash.md) |
| `partyATicketEpoch` | [UINT256](uint256.md) |
| `partyBTicketEpoch` | [UINT256](uint256.md) |
| `partyATicketIndex` | [UINT256](uint256.md) |
| `partyBTicketIndex` | [UINT256](uint256.md) |
| `status` | [ChannelStatus](../enums/channelstatus.md) |
| `channelEpoch` | [UINT256](uint256.md) |
| `closureTime` | [UINT256](uint256.md) |
| `closureByPartyA` | `boolean` |

#### Defined in

[types/channelEntry.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L66)

## Properties

### channelEpoch

• `Readonly` **channelEpoch**: [UINT256](uint256.md)

___

### closureByPartyA

• `Readonly` **closureByPartyA**: `boolean`

___

### closureTime

• `Readonly` **closureTime**: [UINT256](uint256.md)

___

### commitmentPartyA

• `Readonly` **commitmentPartyA**: [Hash](hash.md)

___

### commitmentPartyB

• `Readonly` **commitmentPartyB**: [Hash](hash.md)

___

### partyA

• `Readonly` **partyA**: [Address](address.md)

___

### partyABalance

• `Readonly` **partyABalance**: [Balance](balance.md)

___

### partyATicketEpoch

• `Readonly` **partyATicketEpoch**: [UINT256](uint256.md)

___

### partyATicketIndex

• `Readonly` **partyATicketIndex**: [UINT256](uint256.md)

___

### partyB

• `Readonly` **partyB**: [Address](address.md)

___

### partyBBalance

• `Readonly` **partyBBalance**: [Balance](balance.md)

___

### partyBTicketEpoch

• `Readonly` **partyBTicketEpoch**: [UINT256](uint256.md)

___

### partyBTicketIndex

• `Readonly` **partyBTicketIndex**: [UINT256](uint256.md)

___

### status

• `Readonly` **status**: [ChannelStatus](../enums/channelstatus.md)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/channelEntry.ts:84](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L84)

## Methods

### commitmentFor

▸ **commitmentFor**(`addr`): [Hash](hash.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [Address](address.md) |

#### Returns

[Hash](hash.md)

#### Defined in

[types/channelEntry.ts:191](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L191)

___

### getId

▸ **getId**(): [Hash](hash.md)

#### Returns

[Hash](hash.md)

#### Defined in

[types/channelEntry.ts:163](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L163)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/channelEntry.ts:123](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L123)

___

### ticketEpochFor

▸ **ticketEpochFor**(`addr`): [UINT256](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [Address](address.md) |

#### Returns

[UINT256](uint256.md)

#### Defined in

[types/channelEntry.ts:171](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L171)

___

### ticketIndexFor

▸ **ticketIndexFor**(`addr`): [UINT256](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [Address](address.md) |

#### Returns

[UINT256](uint256.md)

#### Defined in

[types/channelEntry.ts:181](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L181)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[types/channelEntry.ts:142](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L142)

___

### totalBalance

▸ **totalBalance**(): [Balance](balance.md)

#### Returns

[Balance](balance.md)

#### Defined in

[types/channelEntry.ts:167](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L167)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [ChannelEntry](channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[ChannelEntry](channelentry.md)

#### Defined in

[types/channelEntry.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L88)

___

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`): [ChannelEntry](channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `any` |

#### Returns

[ChannelEntry](channelentry.md)

#### Defined in

[types/channelEntry.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L99)
