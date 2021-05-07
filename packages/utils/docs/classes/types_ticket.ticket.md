[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/ticket](../modules/types_ticket.md) / Ticket

# Class: Ticket

[types/ticket](../modules/types_ticket.md).Ticket

## Table of contents

### Constructors

- [constructor](types_ticket.ticket.md#constructor)

### Properties

- [amount](types_ticket.ticket.md#amount)
- [challenge](types_ticket.ticket.md#challenge)
- [channelIteration](types_ticket.ticket.md#channeliteration)
- [counterparty](types_ticket.ticket.md#counterparty)
- [epoch](types_ticket.ticket.md#epoch)
- [index](types_ticket.ticket.md#index)
- [signature](types_ticket.ticket.md#signature)
- [winProb](types_ticket.ticket.md#winprob)

### Accessors

- [SIZE](types_ticket.ticket.md#size)

### Methods

- [getHash](types_ticket.ticket.md#gethash)
- [isWinningTicket](types_ticket.ticket.md#iswinningticket)
- [recoverSigner](types_ticket.ticket.md#recoversigner)
- [serialize](types_ticket.ticket.md#serialize)
- [verify](types_ticket.ticket.md#verify)
- [create](types_ticket.ticket.md#create)
- [deserialize](types_ticket.ticket.md#deserialize)

## Constructors

### constructor

\+ **new Ticket**(`counterparty`: [_Address_](types_primitives.address.md), `challenge`: [_Address_](types_primitives.address.md), `epoch`: [_UINT256_](types_solidity.uint256.md), `index`: [_UINT256_](types_solidity.uint256.md), `amount`: [_Balance_](types_primitives.balance.md), `winProb`: [_UINT256_](types_solidity.uint256.md), `channelIteration`: [_UINT256_](types_solidity.uint256.md), `signature`: [_Signature_](types_primitives.signature.md)): [_Ticket_](types_ticket.ticket.md)

#### Parameters

| Name               | Type                                         |
| :----------------- | :------------------------------------------- |
| `counterparty`     | [_Address_](types_primitives.address.md)     |
| `challenge`        | [_Address_](types_primitives.address.md)     |
| `epoch`            | [_UINT256_](types_solidity.uint256.md)       |
| `index`            | [_UINT256_](types_solidity.uint256.md)       |
| `amount`           | [_Balance_](types_primitives.balance.md)     |
| `winProb`          | [_UINT256_](types_solidity.uint256.md)       |
| `channelIteration` | [_UINT256_](types_solidity.uint256.md)       |
| `signature`        | [_Signature_](types_primitives.signature.md) |

**Returns:** [_Ticket_](types_ticket.ticket.md)

Defined in: [types/ticket.ts:47](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L47)

## Properties

### amount

• `Readonly` **amount**: [_Balance_](types_primitives.balance.md)

---

### challenge

• `Readonly` **challenge**: [_Address_](types_primitives.address.md)

---

### channelIteration

• `Readonly` **channelIteration**: [_UINT256_](types_solidity.uint256.md)

---

### counterparty

• `Readonly` **counterparty**: [_Address_](types_primitives.address.md)

---

### epoch

• `Readonly` **epoch**: [_UINT256_](types_solidity.uint256.md)

---

### index

• `Readonly` **index**: [_UINT256_](types_solidity.uint256.md)

---

### signature

• `Readonly` **signature**: [_Signature_](types_primitives.signature.md)

---

### winProb

• `Readonly` **winProb**: [_UINT256_](types_solidity.uint256.md)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/ticket.ts:133](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L133)

## Methods

### getHash

▸ **getHash**(): [_Hash_](types_primitives.hash.md)

**Returns:** [_Hash_](types_primitives.hash.md)

Defined in: [types/ticket.ts:117](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L117)

---

### isWinningTicket

▸ **isWinningTicket**(`preImage`: [_Hash_](types_primitives.hash.md), `challengeResponse`: [_Hash_](types_primitives.hash.md), `winProb`: [_UINT256_](types_solidity.uint256.md)): _boolean_

Decides whether a ticket is a win or not.
Note that this mimics the on-chain logic.

**`dev`** Purpose of the function is to check the validity of
a ticket before we submit it to the blockchain.

#### Parameters

| Name                | Type                                   | Description                               |
| :------------------ | :------------------------------------- | :---------------------------------------- |
| `preImage`          | [_Hash_](types_primitives.hash.md)     | preImage of the current onChainSecret     |
| `challengeResponse` | [_Hash_](types_primitives.hash.md)     | response that solves the signed challenge |
| `winProb`           | [_UINT256_](types_solidity.uint256.md) | winning probability of the ticket         |

**Returns:** _boolean_

Defined in: [types/ticket.ts:163](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L163)

---

### recoverSigner

▸ **recoverSigner**(): [_PublicKey_](types_primitives.publickey.md)

**Returns:** [_PublicKey_](types_primitives.publickey.md)

Defined in: [types/ticket.ts:146](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L146)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/ticket.ts:89](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L89)

---

### verify

▸ **verify**(`pubKey`: [_PublicKey_](types_primitives.publickey.md)): _boolean_

#### Parameters

| Name     | Type                                         |
| :------- | :------------------------------------------- |
| `pubKey` | [_PublicKey_](types_primitives.publickey.md) |

**Returns:** _boolean_

Defined in: [types/ticket.ts:150](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L150)

---

### create

▸ `Static` **create**(`counterparty`: [_Address_](types_primitives.address.md), `challenge`: [_PublicKey_](types_primitives.publickey.md), `epoch`: [_UINT256_](types_solidity.uint256.md), `index`: [_UINT256_](types_solidity.uint256.md), `amount`: [_Balance_](types_primitives.balance.md), `winProb`: [_UINT256_](types_solidity.uint256.md), `channelIteration`: [_UINT256_](types_solidity.uint256.md), `signPriv`: _Uint8Array_): [_Ticket_](types_ticket.ticket.md)

#### Parameters

| Name               | Type                                         |
| :----------------- | :------------------------------------------- |
| `counterparty`     | [_Address_](types_primitives.address.md)     |
| `challenge`        | [_PublicKey_](types_primitives.publickey.md) |
| `epoch`            | [_UINT256_](types_solidity.uint256.md)       |
| `index`            | [_UINT256_](types_solidity.uint256.md)       |
| `amount`           | [_Balance_](types_primitives.balance.md)     |
| `winProb`          | [_UINT256_](types_solidity.uint256.md)       |
| `channelIteration` | [_UINT256_](types_solidity.uint256.md)       |
| `signPriv`         | _Uint8Array_                                 |

**Returns:** [_Ticket_](types_ticket.ticket.md)

Defined in: [types/ticket.ts:59](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L59)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_Ticket_](types_ticket.ticket.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_Ticket_](types_ticket.ticket.md)

Defined in: [types/ticket.ts:94](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L94)
