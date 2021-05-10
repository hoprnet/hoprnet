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

\+ **new UnacknowledgedTicket**(`ticket`: [_Ticket_](ticket.md), `ownKey`: [_Hash_](hash.md)): [_UnacknowledgedTicket_](unacknowledgedticket.md)

#### Parameters

| Name     | Type                  |
| :------- | :-------------------- |
| `ticket` | [_Ticket_](ticket.md) |
| `ownKey` | [_Hash_](hash.md)     |

**Returns:** [_UnacknowledgedTicket_](unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L4)

## Properties

### ownKey

• `Readonly` **ownKey**: [_Hash_](hash.md)

---

### ticket

• `Readonly` **ticket**: [_Ticket_](ticket.md)

## Methods

### getResponse

▸ **getResponse**(`acknowledgement`: [_Hash_](hash.md)): { `response`: _Uint8Array_ ; `valid`: `true` } \| { `valid`: `false` }

#### Parameters

| Name              | Type              |
| :---------------- | :---------------- |
| `acknowledgement` | [_Hash_](hash.md) |

**Returns:** { `response`: _Uint8Array_ ; `valid`: `true` } \| { `valid`: `false` }

Defined in: [types/unacknowledgedTicket.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L24)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/unacknowledgedTicket.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L13)

---

### verify

▸ **verify**(`signer`: [_PublicKey_](publickey.md), `acknowledgement`: [_Hash_](hash.md)): _boolean_

#### Parameters

| Name              | Type                        |
| :---------------- | :-------------------------- |
| `signer`          | [_PublicKey_](publickey.md) |
| `acknowledgement` | [_Hash_](hash.md)           |

**Returns:** _boolean_

Defined in: [types/unacknowledgedTicket.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L28)

---

### verifySignature

▸ **verifySignature**(`signer`: [_PublicKey_](publickey.md)): _boolean_

#### Parameters

| Name     | Type                        |
| :------- | :-------------------------- |
| `signer` | [_PublicKey_](publickey.md) |

**Returns:** _boolean_

Defined in: [types/unacknowledgedTicket.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L20)

---

### SIZE

▸ `Static` **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/unacknowledgedTicket.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L35)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_UnacknowledgedTicket_](unacknowledgedticket.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_UnacknowledgedTicket_](unacknowledgedticket.md)

Defined in: [types/unacknowledgedTicket.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L7)
