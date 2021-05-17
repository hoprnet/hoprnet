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

\+ **new AcknowledgedTicket**(`ticket`: [*Ticket*](ticket.md), `response`: [*Response*](response.md), `preImage`: [*Hash*](hash.md)): [*AcknowledgedTicket*](acknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [*Ticket*](ticket.md) |
| `response` | [*Response*](response.md) |
| `preImage` | [*Hash*](hash.md) |

**Returns:** [*AcknowledgedTicket*](acknowledgedticket.md)

Defined in: [types/acknowledgedTicket.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L4)

## Properties

### preImage

• `Readonly` **preImage**: [*Hash*](hash.md)

___

### response

• `Readonly` **response**: [*Response*](response.md)

___

### ticket

• `Readonly` **ticket**: [*Ticket*](ticket.md)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/acknowledgedTicket.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L28)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/acknowledgedTicket.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L7)

___

### verify

▸ **verify**(`ticketIssuer`: [*PublicKey*](publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticketIssuer` | [*PublicKey*](publickey.md) |

**Returns:** *boolean*

Defined in: [types/acknowledgedTicket.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L15)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*AcknowledgedTicket*](acknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*AcknowledgedTicket*](acknowledgedticket.md)

Defined in: [types/acknowledgedTicket.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L19)
