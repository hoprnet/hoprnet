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

\+ **new Ticket**(`counterparty`: [*Address*](types_primitives.address.md), `challenge`: [*Address*](types_primitives.address.md), `epoch`: [*UINT256*](types_solidity.uint256.md), `index`: [*UINT256*](types_solidity.uint256.md), `amount`: [*Balance*](types_primitives.balance.md), `winProb`: [*UINT256*](types_solidity.uint256.md), `channelIteration`: [*UINT256*](types_solidity.uint256.md), `signature`: [*Signature*](types_primitives.signature.md)): [*Ticket*](types_ticket.ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [*Address*](types_primitives.address.md) |
| `challenge` | [*Address*](types_primitives.address.md) |
| `epoch` | [*UINT256*](types_solidity.uint256.md) |
| `index` | [*UINT256*](types_solidity.uint256.md) |
| `amount` | [*Balance*](types_primitives.balance.md) |
| `winProb` | [*UINT256*](types_solidity.uint256.md) |
| `channelIteration` | [*UINT256*](types_solidity.uint256.md) |
| `signature` | [*Signature*](types_primitives.signature.md) |

**Returns:** [*Ticket*](types_ticket.ticket.md)

Defined in: [types/ticket.ts:47](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L47)

## Properties

### amount

• `Readonly` **amount**: [*Balance*](types_primitives.balance.md)

___

### challenge

• `Readonly` **challenge**: [*Address*](types_primitives.address.md)

___

### channelIteration

• `Readonly` **channelIteration**: [*UINT256*](types_solidity.uint256.md)

___

### counterparty

• `Readonly` **counterparty**: [*Address*](types_primitives.address.md)

___

### epoch

• `Readonly` **epoch**: [*UINT256*](types_solidity.uint256.md)

___

### index

• `Readonly` **index**: [*UINT256*](types_solidity.uint256.md)

___

### signature

• `Readonly` **signature**: [*Signature*](types_primitives.signature.md)

___

### winProb

• `Readonly` **winProb**: [*UINT256*](types_solidity.uint256.md)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/ticket.ts:133](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L133)

## Methods

### getHash

▸ **getHash**(): [*Hash*](types_primitives.hash.md)

**Returns:** [*Hash*](types_primitives.hash.md)

Defined in: [types/ticket.ts:117](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L117)

___

### isWinningTicket

▸ **isWinningTicket**(`preImage`: [*Hash*](types_primitives.hash.md), `challengeResponse`: [*Hash*](types_primitives.hash.md), `winProb`: [*UINT256*](types_solidity.uint256.md)): *boolean*

Decides whether a ticket is a win or not.
Note that this mimics the on-chain logic.

**`dev`** Purpose of the function is to check the validity of
a ticket before we submit it to the blockchain.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `preImage` | [*Hash*](types_primitives.hash.md) | preImage of the current onChainSecret |
| `challengeResponse` | [*Hash*](types_primitives.hash.md) | response that solves the signed challenge |
| `winProb` | [*UINT256*](types_solidity.uint256.md) | winning probability of the ticket |

**Returns:** *boolean*

Defined in: [types/ticket.ts:163](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L163)

___

### recoverSigner

▸ **recoverSigner**(): [*PublicKey*](types_primitives.publickey.md)

**Returns:** [*PublicKey*](types_primitives.publickey.md)

Defined in: [types/ticket.ts:146](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L146)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/ticket.ts:89](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L89)

___

### verify

▸ **verify**(`pubKey`: [*PublicKey*](types_primitives.publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `pubKey` | [*PublicKey*](types_primitives.publickey.md) |

**Returns:** *boolean*

Defined in: [types/ticket.ts:150](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L150)

___

### create

▸ `Static` **create**(`counterparty`: [*Address*](types_primitives.address.md), `challenge`: [*PublicKey*](types_primitives.publickey.md), `epoch`: [*UINT256*](types_solidity.uint256.md), `index`: [*UINT256*](types_solidity.uint256.md), `amount`: [*Balance*](types_primitives.balance.md), `winProb`: [*UINT256*](types_solidity.uint256.md), `channelIteration`: [*UINT256*](types_solidity.uint256.md), `signPriv`: *Uint8Array*): [*Ticket*](types_ticket.ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [*Address*](types_primitives.address.md) |
| `challenge` | [*PublicKey*](types_primitives.publickey.md) |
| `epoch` | [*UINT256*](types_solidity.uint256.md) |
| `index` | [*UINT256*](types_solidity.uint256.md) |
| `amount` | [*Balance*](types_primitives.balance.md) |
| `winProb` | [*UINT256*](types_solidity.uint256.md) |
| `channelIteration` | [*UINT256*](types_solidity.uint256.md) |
| `signPriv` | *Uint8Array* |

**Returns:** [*Ticket*](types_ticket.ticket.md)

Defined in: [types/ticket.ts:59](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L59)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Ticket*](types_ticket.ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Ticket*](types_ticket.ticket.md)

Defined in: [types/ticket.ts:94](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/ticket.ts#L94)
