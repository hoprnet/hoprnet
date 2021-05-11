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
- [deserialize](channelentry.md#deserialize)
- [fromSCEvent](channelentry.md#fromscevent)

## Constructors

### constructor

\+ **new ChannelEntry**(`partyA`: [*Address*](address.md), `partyB`: [*Address*](address.md), `partyABalance`: [*Balance*](balance.md), `partyBBalance`: [*Balance*](balance.md), `commitmentPartyA`: [*Hash*](hash.md), `commitmentPartyB`: [*Hash*](hash.md), `partyATicketEpoch`: [*UINT256*](uint256.md), `partyBTicketEpoch`: [*UINT256*](uint256.md), `partyATicketIndex`: [*UINT256*](uint256.md), `partyBTicketIndex`: [*UINT256*](uint256.md), `status`: [*ChannelStatus*](../modules.md#channelstatus), `channelEpoch`: [*UINT256*](uint256.md), `closureTime`: [*UINT256*](uint256.md), `closureByPartyA`: *boolean*): [*ChannelEntry*](channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `partyA` | [*Address*](address.md) |
| `partyB` | [*Address*](address.md) |
| `partyABalance` | [*Balance*](balance.md) |
| `partyBBalance` | [*Balance*](balance.md) |
| `commitmentPartyA` | [*Hash*](hash.md) |
| `commitmentPartyB` | [*Hash*](hash.md) |
| `partyATicketEpoch` | [*UINT256*](uint256.md) |
| `partyBTicketEpoch` | [*UINT256*](uint256.md) |
| `partyATicketIndex` | [*UINT256*](uint256.md) |
| `partyBTicketIndex` | [*UINT256*](uint256.md) |
| `status` | [*ChannelStatus*](../modules.md#channelstatus) |
| `channelEpoch` | [*UINT256*](uint256.md) |
| `closureTime` | [*UINT256*](uint256.md) |
| `closureByPartyA` | *boolean* |

**Returns:** [*ChannelEntry*](channelentry.md)

Defined in: [types/channelEntry.ts:48](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L48)

## Properties

### channelEpoch

• `Readonly` **channelEpoch**: [*UINT256*](uint256.md)

___

### closureByPartyA

• `Readonly` **closureByPartyA**: *boolean*

___

### closureTime

• `Readonly` **closureTime**: [*UINT256*](uint256.md)

___

### commitmentPartyA

• `Readonly` **commitmentPartyA**: [*Hash*](hash.md)

___

### commitmentPartyB

• `Readonly` **commitmentPartyB**: [*Hash*](hash.md)

___

### partyA

• `Readonly` **partyA**: [*Address*](address.md)

___

### partyABalance

• `Readonly` **partyABalance**: [*Balance*](balance.md)

___

### partyATicketEpoch

• `Readonly` **partyATicketEpoch**: [*UINT256*](uint256.md)

___

### partyATicketIndex

• `Readonly` **partyATicketIndex**: [*UINT256*](uint256.md)

___

### partyB

• `Readonly` **partyB**: [*Address*](address.md)

___

### partyBBalance

• `Readonly` **partyBBalance**: [*Balance*](balance.md)

___

### partyBTicketEpoch

• `Readonly` **partyBTicketEpoch**: [*UINT256*](uint256.md)

___

### partyBTicketIndex

• `Readonly` **partyBTicketIndex**: [*UINT256*](uint256.md)

___

### status

• `Readonly` **status**: [*ChannelStatus*](../modules.md#channelstatus)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/channelEntry.ts:66](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L66)

## Methods

### commitmentFor

▸ **commitmentFor**(`addr`: [*Address*](address.md)): [*Hash*](hash.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [*Address*](address.md) |

**Returns:** [*Hash*](hash.md)

Defined in: [types/channelEntry.ts:144](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L144)

___

### getId

▸ **getId**(): [*Hash*](hash.md)

**Returns:** [*Hash*](hash.md)

Defined in: [types/channelEntry.ts:120](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L120)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/channelEntry.ts:101](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L101)

___

### ticketEpochFor

▸ **ticketEpochFor**(`addr`: [*Address*](address.md)): [*UINT256*](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [*Address*](address.md) |

**Returns:** [*UINT256*](uint256.md)

Defined in: [types/channelEntry.ts:124](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L124)

___

### ticketIndexFor

▸ **ticketIndexFor**(`addr`: [*Address*](address.md)): [*UINT256*](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [*Address*](address.md) |

**Returns:** [*UINT256*](uint256.md)

Defined in: [types/channelEntry.ts:134](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L134)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*ChannelEntry*](channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*ChannelEntry*](channelentry.md)

Defined in: [types/channelEntry.ts:70](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L70)

___

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`: *any*): [*ChannelEntry*](channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *any* |

**Returns:** [*ChannelEntry*](channelentry.md)

Defined in: [types/channelEntry.ts:80](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L80)
