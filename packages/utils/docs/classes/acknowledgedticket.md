[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / AcknowledgedTicket

# Class: AcknowledgedTicket

## Table of contents

### Constructors

- [constructor](acknowledgedticket.md#constructor)

### Properties

- [preImage](acknowledgedticket.md#preimage)
- [response](acknowledgedticket.md#response)
- [ticket](acknowledgedticket.md#ticket)

### Accessors

- [SIZE](acknowledgedticket.md#size)

### Methods

- [serialize](acknowledgedticket.md#serialize)
- [verify](acknowledgedticket.md#verify)
- [deserialize](acknowledgedticket.md#deserialize)

## Constructors

### constructor

\+ **new AcknowledgedTicket**(`ticket`: [_Ticket_](ticket.md), `response`: [_Hash_](hash.md), `preImage`: [_Hash_](hash.md)): [_AcknowledgedTicket_](acknowledgedticket.md)

#### Parameters

| Name       | Type                  |
| :--------- | :-------------------- |
| `ticket`   | [_Ticket_](ticket.md) |
| `response` | [_Hash_](hash.md)     |
| `preImage` | [_Hash_](hash.md)     |

**Returns:** [_AcknowledgedTicket_](acknowledgedticket.md)

Defined in: [types/acknowledgedTicket.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L5)

## Properties

### preImage

• `Readonly` **preImage**: [_Hash_](hash.md)

---

### response

• `Readonly` **response**: [_Hash_](hash.md)

---

### ticket

• `Readonly` **ticket**: [_Ticket_](ticket.md)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/acknowledgedTicket.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L28)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/acknowledgedTicket.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L8)

---

### verify

▸ **verify**(`ticketIssuer`: [_PublicKey_](publickey.md)): _boolean_

#### Parameters

| Name           | Type                        |
| :------------- | :-------------------------- |
| `ticketIssuer` | [_PublicKey_](publickey.md) |

**Returns:** _boolean_

Defined in: [types/acknowledgedTicket.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L16)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_AcknowledgedTicket_](acknowledgedticket.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_AcknowledgedTicket_](acknowledgedticket.md)

Defined in: [types/acknowledgedTicket.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L23)
