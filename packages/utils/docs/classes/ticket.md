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
- [getLuck](ticket.md#getluck)
- [getPathPosition](ticket.md#getpathposition)
- [isWinningTicket](ticket.md#iswinningticket)
- [recoverSigner](ticket.md#recoversigner)
- [serialize](ticket.md#serialize)
- [serializeUnsigned](ticket.md#serializeunsigned)
- [verify](ticket.md#verify)
- [create](ticket.md#create)
- [deserialize](ticket.md#deserialize)

## Constructors

### constructor

• **new Ticket**(`counterparty`, `challenge`, `epoch`, `index`, `amount`, `winProb`, `channelIteration`, `signature`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [Address](address.md) |
| `challenge` | [EthereumChallenge](ethereumchallenge.md) |
| `epoch` | [UINT256](uint256.md) |
| `index` | [UINT256](uint256.md) |
| `amount` | [Balance](balance.md) |
| `winProb` | [UINT256](uint256.md) |
| `channelIteration` | [UINT256](uint256.md) |
| `signature` | [Signature](signature.md) |

#### Defined in

[types/ticket.ts:48](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L48)

## Properties

### amount

• `Readonly` **amount**: [Balance](balance.md)

___

### challenge

• `Readonly` **challenge**: [EthereumChallenge](ethereumchallenge.md)

___

### channelIteration

• `Readonly` **channelIteration**: [UINT256](uint256.md)

___

### counterparty

• `Readonly` **counterparty**: [Address](address.md)

___

### epoch

• `Readonly` **epoch**: [UINT256](uint256.md)

___

### index

• `Readonly` **index**: [UINT256](uint256.md)

___

### signature

• `Readonly` **signature**: [Signature](signature.md)

___

### winProb

• `Readonly` **winProb**: [UINT256](uint256.md)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/ticket.ts:125](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L125)

## Methods

### getHash

▸ **getHash**(): [Hash](hash.md)

#### Returns

[Hash](hash.md)

#### Defined in

[types/ticket.ts:121](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L121)

___

### getLuck

▸ **getLuck**(`preImage`, `challengeResponse`): [UINT256](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `preImage` | [Hash](hash.md) |
| `challengeResponse` | [Response](response.md) |

#### Returns

[UINT256](uint256.md)

#### Defined in

[types/ticket.ts:146](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L146)

___

### getPathPosition

▸ **getPathPosition**(`pricePerTicket`, `inverseTicketWinProb`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `pricePerTicket` | `BN` |
| `inverseTicketWinProb` | `BN` |

#### Returns

`number`

#### Defined in

[types/ticket.ts:170](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L170)

___

### isWinningTicket

▸ **isWinningTicket**(`preImage`, `challengeResponse`, `winProb`): `boolean`

Decides whether a ticket is a win or not.
Note that this mimics the on-chain logic.

**`dev`** Purpose of the function is to check the validity of
a ticket before we submit it to the blockchain.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `preImage` | [Hash](hash.md) | preImage of the current onChainSecret |
| `challengeResponse` | [Response](response.md) | response that solves the signed challenge |
| `winProb` | [UINT256](uint256.md) | winning probability of the ticket |

#### Returns

`boolean`

#### Defined in

[types/ticket.ts:165](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L165)

___

### recoverSigner

▸ **recoverSigner**(): [PublicKey](publickey.md)

#### Returns

[PublicKey](publickey.md)

#### Defined in

[types/ticket.ts:138](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L138)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/ticket.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L90)

___

### serializeUnsigned

▸ **serializeUnsigned**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/ticket.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L94)

___

### verify

▸ **verify**(`pubKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `pubKey` | [PublicKey](publickey.md) |

#### Returns

`boolean`

#### Defined in

[types/ticket.ts:142](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L142)

___

### create

▸ `Static` **create**(`counterparty`, `challenge`, `epoch`, `index`, `amount`, `winProb`, `channelIteration`, `signPriv`): [Ticket](ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [Address](address.md) |
| `challenge` | [Challenge](challenge.md) |
| `epoch` | [UINT256](uint256.md) |
| `index` | [UINT256](uint256.md) |
| `amount` | [Balance](balance.md) |
| `winProb` | [UINT256](uint256.md) |
| `channelIteration` | [UINT256](uint256.md) |
| `signPriv` | `Uint8Array` |

#### Returns

[Ticket](ticket.md)

#### Defined in

[types/ticket.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L60)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [Ticket](ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[Ticket](ticket.md)

#### Defined in

[types/ticket.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L98)
