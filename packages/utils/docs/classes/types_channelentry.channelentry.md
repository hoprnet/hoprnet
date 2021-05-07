[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/channelEntry](../modules/types_channelentry.md) / ChannelEntry

# Class: ChannelEntry

[types/channelEntry](../modules/types_channelentry.md).ChannelEntry

## Table of contents

### Constructors

- [constructor](types_channelentry.channelentry.md#constructor)

### Properties

- [channelEpoch](types_channelentry.channelentry.md#channelepoch)
- [closureByPartyA](types_channelentry.channelentry.md#closurebypartya)
- [closureTime](types_channelentry.channelentry.md#closuretime)
- [commitmentPartyA](types_channelentry.channelentry.md#commitmentpartya)
- [commitmentPartyB](types_channelentry.channelentry.md#commitmentpartyb)
- [partyA](types_channelentry.channelentry.md#partya)
- [partyABalance](types_channelentry.channelentry.md#partyabalance)
- [partyATicketEpoch](types_channelentry.channelentry.md#partyaticketepoch)
- [partyATicketIndex](types_channelentry.channelentry.md#partyaticketindex)
- [partyB](types_channelentry.channelentry.md#partyb)
- [partyBBalance](types_channelentry.channelentry.md#partybbalance)
- [partyBTicketEpoch](types_channelentry.channelentry.md#partybticketepoch)
- [partyBTicketIndex](types_channelentry.channelentry.md#partybticketindex)
- [status](types_channelentry.channelentry.md#status)

### Accessors

- [SIZE](types_channelentry.channelentry.md#size)

### Methods

- [commitmentFor](types_channelentry.channelentry.md#commitmentfor)
- [getId](types_channelentry.channelentry.md#getid)
- [serialize](types_channelentry.channelentry.md#serialize)
- [ticketEpochFor](types_channelentry.channelentry.md#ticketepochfor)
- [ticketIndexFor](types_channelentry.channelentry.md#ticketindexfor)
- [deserialize](types_channelentry.channelentry.md#deserialize)
- [fromSCEvent](types_channelentry.channelentry.md#fromscevent)

## Constructors

### constructor

\+ **new ChannelEntry**(`partyA`: [*Address*](types_primitives.address.md), `partyB`: [*Address*](types_primitives.address.md), `partyABalance`: [*Balance*](types_primitives.balance.md), `partyBBalance`: [*Balance*](types_primitives.balance.md), `commitmentPartyA`: [*Hash*](types_primitives.hash.md), `commitmentPartyB`: [*Hash*](types_primitives.hash.md), `partyATicketEpoch`: [*UINT256*](types_solidity.uint256.md), `partyBTicketEpoch`: [*UINT256*](types_solidity.uint256.md), `partyATicketIndex`: [*UINT256*](types_solidity.uint256.md), `partyBTicketIndex`: [*UINT256*](types_solidity.uint256.md), `status`: [*ChannelStatus*](../modules/types_channelentry.md#channelstatus), `channelEpoch`: [*UINT256*](types_solidity.uint256.md), `closureTime`: [*UINT256*](types_solidity.uint256.md), `closureByPartyA`: *boolean*): [*ChannelEntry*](types_channelentry.channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `partyA` | [*Address*](types_primitives.address.md) |
| `partyB` | [*Address*](types_primitives.address.md) |
| `partyABalance` | [*Balance*](types_primitives.balance.md) |
| `partyBBalance` | [*Balance*](types_primitives.balance.md) |
| `commitmentPartyA` | [*Hash*](types_primitives.hash.md) |
| `commitmentPartyB` | [*Hash*](types_primitives.hash.md) |
| `partyATicketEpoch` | [*UINT256*](types_solidity.uint256.md) |
| `partyBTicketEpoch` | [*UINT256*](types_solidity.uint256.md) |
| `partyATicketIndex` | [*UINT256*](types_solidity.uint256.md) |
| `partyBTicketIndex` | [*UINT256*](types_solidity.uint256.md) |
| `status` | [*ChannelStatus*](../modules/types_channelentry.md#channelstatus) |
| `channelEpoch` | [*UINT256*](types_solidity.uint256.md) |
| `closureTime` | [*UINT256*](types_solidity.uint256.md) |
| `closureByPartyA` | *boolean* |

**Returns:** [*ChannelEntry*](types_channelentry.channelentry.md)

Defined in: [types/channelEntry.ts:48](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L48)

## Properties

### channelEpoch

• `Readonly` **channelEpoch**: [*UINT256*](types_solidity.uint256.md)

___

### closureByPartyA

• `Readonly` **closureByPartyA**: *boolean*

___

### closureTime

• `Readonly` **closureTime**: [*UINT256*](types_solidity.uint256.md)

___

### commitmentPartyA

• `Readonly` **commitmentPartyA**: [*Hash*](types_primitives.hash.md)

___

### commitmentPartyB

• `Readonly` **commitmentPartyB**: [*Hash*](types_primitives.hash.md)

___

### partyA

• `Readonly` **partyA**: [*Address*](types_primitives.address.md)

___

### partyABalance

• `Readonly` **partyABalance**: [*Balance*](types_primitives.balance.md)

___

### partyATicketEpoch

• `Readonly` **partyATicketEpoch**: [*UINT256*](types_solidity.uint256.md)

___

### partyATicketIndex

• `Readonly` **partyATicketIndex**: [*UINT256*](types_solidity.uint256.md)

___

### partyB

• `Readonly` **partyB**: [*Address*](types_primitives.address.md)

___

### partyBBalance

• `Readonly` **partyBBalance**: [*Balance*](types_primitives.balance.md)

___

### partyBTicketEpoch

• `Readonly` **partyBTicketEpoch**: [*UINT256*](types_solidity.uint256.md)

___

### partyBTicketIndex

• `Readonly` **partyBTicketIndex**: [*UINT256*](types_solidity.uint256.md)

___

### status

• `Readonly` **status**: [*ChannelStatus*](../modules/types_channelentry.md#channelstatus)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/channelEntry.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L66)

## Methods

### commitmentFor

▸ **commitmentFor**(`addr`: [*Address*](types_primitives.address.md)): [*Hash*](types_primitives.hash.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [*Address*](types_primitives.address.md) |

**Returns:** [*Hash*](types_primitives.hash.md)

Defined in: [types/channelEntry.ts:144](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L144)

___

### getId

▸ **getId**(): [*Hash*](types_primitives.hash.md)

**Returns:** [*Hash*](types_primitives.hash.md)

Defined in: [types/channelEntry.ts:120](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L120)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/channelEntry.ts:101](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L101)

___

### ticketEpochFor

▸ **ticketEpochFor**(`addr`: [*Address*](types_primitives.address.md)): [*UINT256*](types_solidity.uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [*Address*](types_primitives.address.md) |

**Returns:** [*UINT256*](types_solidity.uint256.md)

Defined in: [types/channelEntry.ts:124](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L124)

___

### ticketIndexFor

▸ **ticketIndexFor**(`addr`: [*Address*](types_primitives.address.md)): [*UINT256*](types_solidity.uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | [*Address*](types_primitives.address.md) |

**Returns:** [*UINT256*](types_solidity.uint256.md)

Defined in: [types/channelEntry.ts:134](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L134)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*ChannelEntry*](types_channelentry.channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*ChannelEntry*](types_channelentry.channelentry.md)

Defined in: [types/channelEntry.ts:70](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L70)

___

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`: *any*): [*ChannelEntry*](types_channelentry.channelentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *any* |

**Returns:** [*ChannelEntry*](types_channelentry.channelentry.md)

Defined in: [types/channelEntry.ts:80](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L80)
