[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Ticket

# Class: Ticket

## Table of contents

### Constructors

- [constructor](ticket.md#constructor)

### Properties

- [amount](ticket.md#amount)
- [challenge](ticket.md#challenge)
- [channelIteration](ticket.md#channeliteration)
- [counterparty](ticket.md#counterparty)
- [epoch](ticket.md#epoch)
- [index](ticket.md#index)
- [signature](ticket.md#signature)
- [winProb](ticket.md#winprob)

### Accessors

- [SIZE](ticket.md#size)

### Methods

- [getHash](ticket.md#gethash)
- [isWinningTicket](ticket.md#iswinningticket)
- [recoverSigner](ticket.md#recoversigner)
- [serialize](ticket.md#serialize)
- [verify](ticket.md#verify)
- [create](ticket.md#create)
- [deserialize](ticket.md#deserialize)

## Constructors

### constructor

\+ **new Ticket**(`counterparty`: [_Address_](address.md), `challenge`: [_Address_](address.md), `epoch`: [_UINT256_](uint256.md), `index`: [_UINT256_](uint256.md), `amount`: [_Balance_](balance.md), `winProb`: [_UINT256_](uint256.md), `channelIteration`: [_UINT256_](uint256.md), `signature`: [_Signature_](signature.md)): [_Ticket_](ticket.md)

#### Parameters

| Name               | Type                        |
| :----------------- | :-------------------------- |
| `counterparty`     | [_Address_](address.md)     |
| `challenge`        | [_Address_](address.md)     |
| `epoch`            | [_UINT256_](uint256.md)     |
| `index`            | [_UINT256_](uint256.md)     |
| `amount`           | [_Balance_](balance.md)     |
| `winProb`          | [_UINT256_](uint256.md)     |
| `channelIteration` | [_UINT256_](uint256.md)     |
| `signature`        | [_Signature_](signature.md) |

**Returns:** [_Ticket_](ticket.md)

Defined in: [types/ticket.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L47)

## Properties

### amount

• `Readonly` **amount**: [_Balance_](balance.md)

---

### challenge

• `Readonly` **challenge**: [_Address_](address.md)

---

### channelIteration

• `Readonly` **channelIteration**: [_UINT256_](uint256.md)

---

### counterparty

• `Readonly` **counterparty**: [_Address_](address.md)

---

### epoch

• `Readonly` **epoch**: [_UINT256_](uint256.md)

---

### index

• `Readonly` **index**: [_UINT256_](uint256.md)

---

### signature

• `Readonly` **signature**: [_Signature_](signature.md)

---

### winProb

• `Readonly` **winProb**: [_UINT256_](uint256.md)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/ticket.ts:133](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L133)

## Methods

### getHash

▸ **getHash**(): [_Hash_](hash.md)

**Returns:** [_Hash_](hash.md)

Defined in: [types/ticket.ts:117](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L117)

---

### isWinningTicket

▸ **isWinningTicket**(`preImage`: [_Hash_](hash.md), `challengeResponse`: [_Hash_](hash.md), `winProb`: [_UINT256_](uint256.md)): _boolean_

Decides whether a ticket is a win or not.
Note that this mimics the on-chain logic.

**`dev`** Purpose of the function is to check the validity of
a ticket before we submit it to the blockchain.

#### Parameters

| Name                | Type                    | Description                               |
| :------------------ | :---------------------- | :---------------------------------------- |
| `preImage`          | [_Hash_](hash.md)       | preImage of the current onChainSecret     |
| `challengeResponse` | [_Hash_](hash.md)       | response that solves the signed challenge |
| `winProb`           | [_UINT256_](uint256.md) | winning probability of the ticket         |

**Returns:** _boolean_

Defined in: [types/ticket.ts:163](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L163)

---

### recoverSigner

▸ **recoverSigner**(): [_PublicKey_](publickey.md)

**Returns:** [_PublicKey_](publickey.md)

Defined in: [types/ticket.ts:146](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L146)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/ticket.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L89)

---

### verify

▸ **verify**(`pubKey`: [_PublicKey_](publickey.md)): _boolean_

#### Parameters

| Name     | Type                        |
| :------- | :-------------------------- |
| `pubKey` | [_PublicKey_](publickey.md) |

**Returns:** _boolean_

Defined in: [types/ticket.ts:150](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L150)

---

### create

▸ `Static` **create**(`counterparty`: [_Address_](address.md), `challenge`: [_PublicKey_](publickey.md), `epoch`: [_UINT256_](uint256.md), `index`: [_UINT256_](uint256.md), `amount`: [_Balance_](balance.md), `winProb`: [_UINT256_](uint256.md), `channelIteration`: [_UINT256_](uint256.md), `signPriv`: _Uint8Array_): [_Ticket_](ticket.md)

#### Parameters

| Name               | Type                        |
| :----------------- | :-------------------------- |
| `counterparty`     | [_Address_](address.md)     |
| `challenge`        | [_PublicKey_](publickey.md) |
| `epoch`            | [_UINT256_](uint256.md)     |
| `index`            | [_UINT256_](uint256.md)     |
| `amount`           | [_Balance_](balance.md)     |
| `winProb`          | [_UINT256_](uint256.md)     |
| `channelIteration` | [_UINT256_](uint256.md)     |
| `signPriv`         | _Uint8Array_                |

**Returns:** [_Ticket_](ticket.md)

Defined in: [types/ticket.ts:59](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L59)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_Ticket_](ticket.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_Ticket_](ticket.md)

Defined in: [types/ticket.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L94)
