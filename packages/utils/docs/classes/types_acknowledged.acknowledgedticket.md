[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/acknowledged](../modules/types_acknowledged.md) / AcknowledgedTicket

# Class: AcknowledgedTicket

[types/acknowledged](../modules/types_acknowledged.md).AcknowledgedTicket

## Table of contents

### Constructors

- [constructor](types_acknowledged.acknowledgedticket.md#constructor)

### Properties

- [preImage](types_acknowledged.acknowledgedticket.md#preimage)
- [response](types_acknowledged.acknowledgedticket.md#response)
- [ticket](types_acknowledged.acknowledgedticket.md#ticket)

### Accessors

- [SIZE](types_acknowledged.acknowledgedticket.md#size)

### Methods

- [serialize](types_acknowledged.acknowledgedticket.md#serialize)
- [verify](types_acknowledged.acknowledgedticket.md#verify)
- [deserialize](types_acknowledged.acknowledgedticket.md#deserialize)

## Constructors

### constructor

\+ **new AcknowledgedTicket**(`ticket`: [_Ticket_](types_ticket.ticket.md), `response`: [_Hash_](types_primitives.hash.md), `preImage`: [_Hash_](types_primitives.hash.md)): [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md)

#### Parameters

| Name       | Type                               |
| :--------- | :--------------------------------- |
| `ticket`   | [_Ticket_](types_ticket.ticket.md) |
| `response` | [_Hash_](types_primitives.hash.md) |
| `preImage` | [_Hash_](types_primitives.hash.md) |

**Returns:** [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md)

Defined in: [types/acknowledged.ts:5](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L5)

## Properties

### preImage

• `Readonly` **preImage**: [_Hash_](types_primitives.hash.md)

---

### response

• `Readonly` **response**: [_Hash_](types_primitives.hash.md)

---

### ticket

• `Readonly` **ticket**: [_Ticket_](types_ticket.ticket.md)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/acknowledged.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L33)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/acknowledged.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L8)

---

### verify

▸ **verify**(`ticketIssuer`: [_PublicKey_](types_primitives.publickey.md)): _boolean_

#### Parameters

| Name           | Type                                         |
| :------------- | :------------------------------------------- |
| `ticketIssuer` | [_PublicKey_](types_primitives.publickey.md) |

**Returns:** _boolean_

Defined in: [types/acknowledged.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L16)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md)

Defined in: [types/acknowledged.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L28)
