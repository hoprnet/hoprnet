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

\+ **new UnacknowledgedTicket**(`ticket`: [*Ticket*](types_ticket.ticket.md), `ownKey`: [*Hash*](types_primitives.hash.md)): [*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [*Ticket*](types_ticket.ticket.md) |
| `ownKey` | [*Hash*](types_primitives.hash.md) |

**Returns:** [*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L4)

## Properties

### ownKey

• `Readonly` **ownKey**: [*Hash*](types_primitives.hash.md)

___

### ticket

• `Readonly` **ticket**: [*Ticket*](types_ticket.ticket.md)

## Methods

### getResponse

▸ **getResponse**(`acknowledgement`: [*Hash*](types_primitives.hash.md)): { `response`: *Uint8Array* ; `valid`: ``true``  } \| { `valid`: ``false``  }

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [*Hash*](types_primitives.hash.md) |

**Returns:** { `response`: *Uint8Array* ; `valid`: ``true``  } \| { `valid`: ``false``  }

Defined in: [types/unacknowledgedTicket.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L24)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/unacknowledgedTicket.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L13)

___

### verify

▸ **verify**(`signer`: [*PublicKey*](types_primitives.publickey.md), `acknowledgement`: [*Hash*](types_primitives.hash.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | [*PublicKey*](types_primitives.publickey.md) |
| `acknowledgement` | [*Hash*](types_primitives.hash.md) |

**Returns:** *boolean*

Defined in: [types/unacknowledgedTicket.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L32)

___

### verifySignature

▸ **verifySignature**(`signer`: [*PublicKey*](types_primitives.publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | [*PublicKey*](types_primitives.publickey.md) |

**Returns:** *boolean*

Defined in: [types/unacknowledgedTicket.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L20)

___

### SIZE

▸ `Static` **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/unacknowledgedTicket.ts:40](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L40)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/unacknowledgedTicket.ts#L7)
