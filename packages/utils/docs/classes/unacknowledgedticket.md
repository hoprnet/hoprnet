[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / UnacknowledgedTicket

# Class: UnacknowledgedTicket

## Table of contents

### Constructors

- [constructor](unacknowledgedticket.md#constructor)

### Properties

- [ownKey](unacknowledgedticket.md#ownkey)
- [ticket](unacknowledgedticket.md#ticket)

### Methods

- [getResponse](unacknowledgedticket.md#getresponse)
- [serialize](unacknowledgedticket.md#serialize)
- [verify](unacknowledgedticket.md#verify)
- [verifySignature](unacknowledgedticket.md#verifysignature)
- [SIZE](unacknowledgedticket.md#size)
- [deserialize](unacknowledgedticket.md#deserialize)

## Constructors

### constructor

\+ **new UnacknowledgedTicket**(`ticket`: [*Ticket*](ticket.md), `ownKey`: [*Hash*](hash.md)): [*UnacknowledgedTicket*](unacknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [*Ticket*](ticket.md) |
| `ownKey` | [*Hash*](hash.md) |

**Returns:** [*UnacknowledgedTicket*](unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L4)

## Properties

### ownKey

• `Readonly` **ownKey**: [*Hash*](hash.md)

___

### ticket

• `Readonly` **ticket**: [*Ticket*](ticket.md)

## Methods

### getResponse

▸ **getResponse**(`acknowledgement`: [*Hash*](hash.md)): { `response`: *Uint8Array* ; `valid`: ``true``  } \| { `valid`: ``false``  }

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [*Hash*](hash.md) |

**Returns:** { `response`: *Uint8Array* ; `valid`: ``true``  } \| { `valid`: ``false``  }

Defined in: [types/unacknowledgedTicket.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L24)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/unacknowledgedTicket.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L13)

___

### verify

▸ **verify**(`signer`: [*PublicKey*](publickey.md), `acknowledgement`: [*Hash*](hash.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | [*PublicKey*](publickey.md) |
| `acknowledgement` | [*Hash*](hash.md) |

**Returns:** *boolean*

Defined in: [types/unacknowledgedTicket.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L28)

___

### verifySignature

▸ **verifySignature**(`signer`: [*PublicKey*](publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | [*PublicKey*](publickey.md) |

**Returns:** *boolean*

Defined in: [types/unacknowledgedTicket.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L20)

___

### SIZE

▸ `Static` **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/unacknowledgedTicket.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L35)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*UnacknowledgedTicket*](unacknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*UnacknowledgedTicket*](unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L7)
