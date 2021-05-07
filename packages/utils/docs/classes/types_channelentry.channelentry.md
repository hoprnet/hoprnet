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

\+ **new ChannelEntry**(`partyA`: [_Address_](types_primitives.address.md), `partyB`: [_Address_](types_primitives.address.md), `partyABalance`: [_Balance_](types_primitives.balance.md), `partyBBalance`: [_Balance_](types_primitives.balance.md), `commitmentPartyA`: [_Hash_](types_primitives.hash.md), `commitmentPartyB`: [_Hash_](types_primitives.hash.md), `partyATicketEpoch`: [_UINT256_](types_solidity.uint256.md), `partyBTicketEpoch`: [_UINT256_](types_solidity.uint256.md), `partyATicketIndex`: [_UINT256_](types_solidity.uint256.md), `partyBTicketIndex`: [_UINT256_](types_solidity.uint256.md), `status`: [_ChannelStatus_](../modules/types_channelentry.md#channelstatus), `channelEpoch`: [_UINT256_](types_solidity.uint256.md), `closureTime`: [_UINT256_](types_solidity.uint256.md), `closureByPartyA`: _boolean_): [_ChannelEntry_](types_channelentry.channelentry.md)

#### Parameters

| Name                | Type                                                              |
| :------------------ | :---------------------------------------------------------------- |
| `partyA`            | [_Address_](types_primitives.address.md)                          |
| `partyB`            | [_Address_](types_primitives.address.md)                          |
| `partyABalance`     | [_Balance_](types_primitives.balance.md)                          |
| `partyBBalance`     | [_Balance_](types_primitives.balance.md)                          |
| `commitmentPartyA`  | [_Hash_](types_primitives.hash.md)                                |
| `commitmentPartyB`  | [_Hash_](types_primitives.hash.md)                                |
| `partyATicketEpoch` | [_UINT256_](types_solidity.uint256.md)                            |
| `partyBTicketEpoch` | [_UINT256_](types_solidity.uint256.md)                            |
| `partyATicketIndex` | [_UINT256_](types_solidity.uint256.md)                            |
| `partyBTicketIndex` | [_UINT256_](types_solidity.uint256.md)                            |
| `status`            | [_ChannelStatus_](../modules/types_channelentry.md#channelstatus) |
| `channelEpoch`      | [_UINT256_](types_solidity.uint256.md)                            |
| `closureTime`       | [_UINT256_](types_solidity.uint256.md)                            |
| `closureByPartyA`   | _boolean_                                                         |

**Returns:** [_ChannelEntry_](types_channelentry.channelentry.md)

Defined in: [types/channelEntry.ts:48](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L48)

## Properties

### channelEpoch

• `Readonly` **channelEpoch**: [_UINT256_](types_solidity.uint256.md)

---

### closureByPartyA

• `Readonly` **closureByPartyA**: _boolean_

---

### closureTime

• `Readonly` **closureTime**: [_UINT256_](types_solidity.uint256.md)

---

### commitmentPartyA

• `Readonly` **commitmentPartyA**: [_Hash_](types_primitives.hash.md)

---

### commitmentPartyB

• `Readonly` **commitmentPartyB**: [_Hash_](types_primitives.hash.md)

---

### partyA

• `Readonly` **partyA**: [_Address_](types_primitives.address.md)

---

### partyABalance

• `Readonly` **partyABalance**: [_Balance_](types_primitives.balance.md)

---

### partyATicketEpoch

• `Readonly` **partyATicketEpoch**: [_UINT256_](types_solidity.uint256.md)

---

### partyATicketIndex

• `Readonly` **partyATicketIndex**: [_UINT256_](types_solidity.uint256.md)

---

### partyB

• `Readonly` **partyB**: [_Address_](types_primitives.address.md)

---

### partyBBalance

• `Readonly` **partyBBalance**: [_Balance_](types_primitives.balance.md)

---

### partyBTicketEpoch

• `Readonly` **partyBTicketEpoch**: [_UINT256_](types_solidity.uint256.md)

---

### partyBTicketIndex

• `Readonly` **partyBTicketIndex**: [_UINT256_](types_solidity.uint256.md)

---

### status

• `Readonly` **status**: [_ChannelStatus_](../modules/types_channelentry.md#channelstatus)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/channelEntry.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L66)

## Methods

### commitmentFor

▸ **commitmentFor**(`addr`: [_Address_](types_primitives.address.md)): [_Hash_](types_primitives.hash.md)

#### Parameters

| Name   | Type                                     |
| :----- | :--------------------------------------- |
| `addr` | [_Address_](types_primitives.address.md) |

**Returns:** [_Hash_](types_primitives.hash.md)

Defined in: [types/channelEntry.ts:144](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L144)

---

### getId

▸ **getId**(): [_Hash_](types_primitives.hash.md)

**Returns:** [_Hash_](types_primitives.hash.md)

Defined in: [types/channelEntry.ts:120](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L120)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/channelEntry.ts:101](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L101)

---

### ticketEpochFor

▸ **ticketEpochFor**(`addr`: [_Address_](types_primitives.address.md)): [_UINT256_](types_solidity.uint256.md)

#### Parameters

| Name   | Type                                     |
| :----- | :--------------------------------------- |
| `addr` | [_Address_](types_primitives.address.md) |

**Returns:** [_UINT256_](types_solidity.uint256.md)

Defined in: [types/channelEntry.ts:124](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L124)

---

### ticketIndexFor

▸ **ticketIndexFor**(`addr`: [_Address_](types_primitives.address.md)): [_UINT256_](types_solidity.uint256.md)

#### Parameters

| Name   | Type                                     |
| :----- | :--------------------------------------- |
| `addr` | [_Address_](types_primitives.address.md) |

**Returns:** [_UINT256_](types_solidity.uint256.md)

Defined in: [types/channelEntry.ts:134](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L134)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_ChannelEntry_](types_channelentry.channelentry.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_ChannelEntry_](types_channelentry.channelentry.md)

Defined in: [types/channelEntry.ts:70](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L70)

---

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`: _any_): [_ChannelEntry_](types_channelentry.channelentry.md)

#### Parameters

| Name    | Type  |
| :------ | :---- |
| `event` | _any_ |

**Returns:** [_ChannelEntry_](types_channelentry.channelentry.md)

Defined in: [types/channelEntry.ts:80](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L80)
