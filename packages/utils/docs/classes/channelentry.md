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

\+ **new ChannelEntry**(`partyA`: [_Address_](address.md), `partyB`: [_Address_](address.md), `partyABalance`: [_Balance_](balance.md), `partyBBalance`: [_Balance_](balance.md), `commitmentPartyA`: [_Hash_](hash.md), `commitmentPartyB`: [_Hash_](hash.md), `partyATicketEpoch`: [_UINT256_](uint256.md), `partyBTicketEpoch`: [_UINT256_](uint256.md), `partyATicketIndex`: [_UINT256_](uint256.md), `partyBTicketIndex`: [_UINT256_](uint256.md), `status`: [_ChannelStatus_](../modules.md#channelstatus), `channelEpoch`: [_UINT256_](uint256.md), `closureTime`: [_UINT256_](uint256.md), `closureByPartyA`: _boolean_): [_ChannelEntry_](channelentry.md)

#### Parameters

| Name                | Type                                           |
| :------------------ | :--------------------------------------------- |
| `partyA`            | [_Address_](address.md)                        |
| `partyB`            | [_Address_](address.md)                        |
| `partyABalance`     | [_Balance_](balance.md)                        |
| `partyBBalance`     | [_Balance_](balance.md)                        |
| `commitmentPartyA`  | [_Hash_](hash.md)                              |
| `commitmentPartyB`  | [_Hash_](hash.md)                              |
| `partyATicketEpoch` | [_UINT256_](uint256.md)                        |
| `partyBTicketEpoch` | [_UINT256_](uint256.md)                        |
| `partyATicketIndex` | [_UINT256_](uint256.md)                        |
| `partyBTicketIndex` | [_UINT256_](uint256.md)                        |
| `status`            | [_ChannelStatus_](../modules.md#channelstatus) |
| `channelEpoch`      | [_UINT256_](uint256.md)                        |
| `closureTime`       | [_UINT256_](uint256.md)                        |
| `closureByPartyA`   | _boolean_                                      |

**Returns:** [_ChannelEntry_](channelentry.md)

Defined in: [types/channelEntry.ts:48](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L48)

## Properties

### channelEpoch

• `Readonly` **channelEpoch**: [_UINT256_](uint256.md)

---

### closureByPartyA

• `Readonly` **closureByPartyA**: _boolean_

---

### closureTime

• `Readonly` **closureTime**: [_UINT256_](uint256.md)

---

### commitmentPartyA

• `Readonly` **commitmentPartyA**: [_Hash_](hash.md)

---

### commitmentPartyB

• `Readonly` **commitmentPartyB**: [_Hash_](hash.md)

---

### partyA

• `Readonly` **partyA**: [_Address_](address.md)

---

### partyABalance

• `Readonly` **partyABalance**: [_Balance_](balance.md)

---

### partyATicketEpoch

• `Readonly` **partyATicketEpoch**: [_UINT256_](uint256.md)

---

### partyATicketIndex

• `Readonly` **partyATicketIndex**: [_UINT256_](uint256.md)

---

### partyB

• `Readonly` **partyB**: [_Address_](address.md)

---

### partyBBalance

• `Readonly` **partyBBalance**: [_Balance_](balance.md)

---

### partyBTicketEpoch

• `Readonly` **partyBTicketEpoch**: [_UINT256_](uint256.md)

---

### partyBTicketIndex

• `Readonly` **partyBTicketIndex**: [_UINT256_](uint256.md)

---

### status

• `Readonly` **status**: [_ChannelStatus_](../modules.md#channelstatus)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/channelEntry.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L66)

## Methods

### commitmentFor

▸ **commitmentFor**(`addr`: [_Address_](address.md)): [_Hash_](hash.md)

#### Parameters

| Name   | Type                    |
| :----- | :---------------------- |
| `addr` | [_Address_](address.md) |

**Returns:** [_Hash_](hash.md)

Defined in: [types/channelEntry.ts:144](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L144)

---

### getId

▸ **getId**(): [_Hash_](hash.md)

**Returns:** [_Hash_](hash.md)

Defined in: [types/channelEntry.ts:120](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L120)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/channelEntry.ts:101](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L101)

---

### ticketEpochFor

▸ **ticketEpochFor**(`addr`: [_Address_](address.md)): [_UINT256_](uint256.md)

#### Parameters

| Name   | Type                    |
| :----- | :---------------------- |
| `addr` | [_Address_](address.md) |

**Returns:** [_UINT256_](uint256.md)

Defined in: [types/channelEntry.ts:124](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L124)

---

### ticketIndexFor

▸ **ticketIndexFor**(`addr`: [_Address_](address.md)): [_UINT256_](uint256.md)

#### Parameters

| Name   | Type                    |
| :----- | :---------------------- |
| `addr` | [_Address_](address.md) |

**Returns:** [_UINT256_](uint256.md)

Defined in: [types/channelEntry.ts:134](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L134)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_ChannelEntry_](channelentry.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_ChannelEntry_](channelentry.md)

Defined in: [types/channelEntry.ts:70](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L70)

---

### fromSCEvent

▸ `Static` **fromSCEvent**(`event`: _any_): [_ChannelEntry_](channelentry.md)

#### Parameters

| Name    | Type  |
| :------ | :---- |
| `event` | _any_ |

**Returns:** [_ChannelEntry_](channelentry.md)

Defined in: [types/channelEntry.ts:80](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L80)
