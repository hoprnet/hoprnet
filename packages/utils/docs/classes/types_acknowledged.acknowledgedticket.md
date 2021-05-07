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

\+ **new AcknowledgedTicket**(`ticket`: [*Ticket*](types_ticket.ticket.md), `response`: [*Hash*](types_primitives.hash.md), `preImage`: [*Hash*](types_primitives.hash.md)): [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [*Ticket*](types_ticket.ticket.md) |
| `response` | [*Hash*](types_primitives.hash.md) |
| `preImage` | [*Hash*](types_primitives.hash.md) |

**Returns:** [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md)

Defined in: [types/acknowledged.ts:5](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L5)

## Properties

### preImage

• `Readonly` **preImage**: [*Hash*](types_primitives.hash.md)

___

### response

• `Readonly` **response**: [*Hash*](types_primitives.hash.md)

___

### ticket

• `Readonly` **ticket**: [*Ticket*](types_ticket.ticket.md)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/acknowledged.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L33)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/acknowledged.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L8)

___

### verify

▸ **verify**(`ticketIssuer`: [*PublicKey*](types_primitives.publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticketIssuer` | [*PublicKey*](types_primitives.publickey.md) |

**Returns:** *boolean*

Defined in: [types/acknowledged.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L16)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md)

Defined in: [types/acknowledged.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/acknowledged.ts#L28)
