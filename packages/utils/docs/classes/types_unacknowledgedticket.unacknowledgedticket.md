[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/unacknowledgedTicket](../modules/types_unacknowledgedticket.md) / UnacknowledgedTicket

# Class: UnacknowledgedTicket

[types/unacknowledgedTicket](../modules/types_unacknowledgedticket.md).UnacknowledgedTicket

## Table of contents

### Constructors

- [constructor](types_unacknowledgedticket.unacknowledgedticket.md#constructor)

### Properties

- [ownKey](types_unacknowledgedticket.unacknowledgedticket.md#ownkey)
- [ticket](types_unacknowledgedticket.unacknowledgedticket.md#ticket)

### Methods

- [getResponse](types_unacknowledgedticket.unacknowledgedticket.md#getresponse)
- [serialize](types_unacknowledgedticket.unacknowledgedticket.md#serialize)
- [verify](types_unacknowledgedticket.unacknowledgedticket.md#verify)
- [verifySignature](types_unacknowledgedticket.unacknowledgedticket.md#verifysignature)
- [SIZE](types_unacknowledgedticket.unacknowledgedticket.md#size)
- [deserialize](types_unacknowledgedticket.unacknowledgedticket.md#deserialize)

## Constructors

### constructor

\+ **new UnacknowledgedTicket**(`ticket`: [_Ticket_](types_ticket.ticket.md), `ownKey`: [_Hash_](types_primitives.hash.md)): [_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)

#### Parameters

| Name     | Type                               |
| :------- | :--------------------------------- |
| `ticket` | [_Ticket_](types_ticket.ticket.md) |
| `ownKey` | [_Hash_](types_primitives.hash.md) |

**Returns:** [_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L4)

## Properties

### ownKey

• `Readonly` **ownKey**: [_Hash_](types_primitives.hash.md)

---

### ticket

• `Readonly` **ticket**: [_Ticket_](types_ticket.ticket.md)

## Methods

### getResponse

▸ **getResponse**(`acknowledgement`: [_Hash_](types_primitives.hash.md)): { `response`: _Uint8Array_ ; `valid`: `true` } \| { `valid`: `false` }

#### Parameters

| Name              | Type                               |
| :---------------- | :--------------------------------- |
| `acknowledgement` | [_Hash_](types_primitives.hash.md) |

**Returns:** { `response`: _Uint8Array_ ; `valid`: `true` } \| { `valid`: `false` }

Defined in: [types/unacknowledgedTicket.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L24)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/unacknowledgedTicket.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L13)

---

### verify

▸ **verify**(`signer`: [_PublicKey_](types_primitives.publickey.md), `acknowledgement`: [_Hash_](types_primitives.hash.md)): _boolean_

#### Parameters

| Name              | Type                                         |
| :---------------- | :------------------------------------------- |
| `signer`          | [_PublicKey_](types_primitives.publickey.md) |
| `acknowledgement` | [_Hash_](types_primitives.hash.md)           |

**Returns:** _boolean_

Defined in: [types/unacknowledgedTicket.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L32)

---

### verifySignature

▸ **verifySignature**(`signer`: [_PublicKey_](types_primitives.publickey.md)): _boolean_

#### Parameters

| Name     | Type                                         |
| :------- | :------------------------------------------- |
| `signer` | [_PublicKey_](types_primitives.publickey.md) |

**Returns:** _boolean_

Defined in: [types/unacknowledgedTicket.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L20)

---

### SIZE

▸ `Static` **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/unacknowledgedTicket.ts:40](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L40)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L7)
