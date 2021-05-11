[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / UnacknowledgedTicket

# Class: UnacknowledgedTicket

## Table of contents

### Constructors

- [constructor](unacknowledgedticket.md#constructor)

### Properties

- [ownKey](unacknowledgedticket.md#ownkey)
- [ticket](unacknowledgedticket.md#ticket)

### Methods

- [getChallenge](unacknowledgedticket.md#getchallenge)
- [getResponse](unacknowledgedticket.md#getresponse)
- [serialize](unacknowledgedticket.md#serialize)
- [verify](unacknowledgedticket.md#verify)
- [verifySignature](unacknowledgedticket.md#verifysignature)
- [SIZE](unacknowledgedticket.md#size)
- [deserialize](unacknowledgedticket.md#deserialize)

## Constructors

### constructor

\+ **new UnacknowledgedTicket**(`ticket`: [*Ticket*](ticket.md), `ownKey`: [*HalfKey*](halfkey.md)): [*UnacknowledgedTicket*](unacknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [*Ticket*](ticket.md) |
| `ownKey` | [*HalfKey*](halfkey.md) |

**Returns:** [*UnacknowledgedTicket*](unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L4)

## Properties

### ownKey

• `Readonly` **ownKey**: [*HalfKey*](halfkey.md)

___

### ticket

• `Readonly` **ticket**: [*Ticket*](ticket.md)

## Methods

### getChallenge

▸ **getChallenge**(): [*HalfKeyChallenge*](halfkeychallenge.md)

**Returns:** [*HalfKeyChallenge*](halfkeychallenge.md)

Defined in: [types/unacknowledgedTicket.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L28)

___

### getResponse

▸ **getResponse**(`acknowledgement`: [*HalfKey*](halfkey.md)): [*Response*](response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [*HalfKey*](halfkey.md) |

**Returns:** [*Response*](response.md)

Defined in: [types/unacknowledgedTicket.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L24)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/unacknowledgedTicket.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L13)

___

### verify

▸ **verify**(`signer`: [*PublicKey*](publickey.md), `acknowledgement`: [*HalfKey*](halfkey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | [*PublicKey*](publickey.md) |
| `acknowledgement` | [*HalfKey*](halfkey.md) |

**Returns:** *boolean*

Defined in: [types/unacknowledgedTicket.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L32)

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

Defined in: [types/unacknowledgedTicket.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L36)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*UnacknowledgedTicket*](unacknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*UnacknowledgedTicket*](unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L7)
