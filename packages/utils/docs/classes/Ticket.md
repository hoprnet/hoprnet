[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Ticket

# Class: Ticket

## Table of contents

### Constructors

- [constructor](Ticket.md#constructor)

### Properties

- [amount](Ticket.md#amount)
- [challenge](Ticket.md#challenge)
- [channelEpoch](Ticket.md#channelepoch)
- [counterparty](Ticket.md#counterparty)
- [epoch](Ticket.md#epoch)
- [index](Ticket.md#index)
- [signature](Ticket.md#signature)
- [winProb](Ticket.md#winprob)

### Accessors

- [SIZE](Ticket.md#size)

### Methods

- [getHash](Ticket.md#gethash)
- [getLuck](Ticket.md#getluck)
- [getPathPosition](Ticket.md#getpathposition)
- [isWinningTicket](Ticket.md#iswinningticket)
- [recoverSigner](Ticket.md#recoversigner)
- [serialize](Ticket.md#serialize)
- [serializeUnsigned](Ticket.md#serializeunsigned)
- [toString](Ticket.md#tostring)
- [verify](Ticket.md#verify)
- [create](Ticket.md#create)
- [deserialize](Ticket.md#deserialize)

## Constructors

### constructor

• **new Ticket**(`counterparty`, `challenge`, `epoch`, `index`, `amount`, `winProb`, `channelEpoch`, `signature`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [`Address`](Address.md) |
| `challenge` | [`EthereumChallenge`](EthereumChallenge.md) |
| `epoch` | [`UINT256`](UINT256.md) |
| `index` | [`UINT256`](UINT256.md) |
| `amount` | [`Balance`](Balance.md) |
| `winProb` | [`UINT256`](UINT256.md) |
| `channelEpoch` | [`UINT256`](UINT256.md) |
| `signature` | [`Signature`](Signature.md) |

## Properties

### amount

• `Readonly` **amount**: [`Balance`](Balance.md)

#### Defined in

[src/types/ticket.ts:54](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L54)

___

### challenge

• `Readonly` **challenge**: [`EthereumChallenge`](EthereumChallenge.md)

#### Defined in

[src/types/ticket.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L51)

___

### channelEpoch

• `Readonly` **channelEpoch**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/ticket.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L56)

___

### counterparty

• `Readonly` **counterparty**: [`Address`](Address.md)

#### Defined in

[src/types/ticket.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L50)

___

### epoch

• `Readonly` **epoch**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/ticket.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L52)

___

### index

• `Readonly` **index**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/ticket.ts:53](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L53)

___

### signature

• `Readonly` **signature**: [`Signature`](Signature.md)

#### Defined in

[src/types/ticket.ts:57](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L57)

___

### winProb

• `Readonly` **winProb**: [`UINT256`](UINT256.md)

#### Defined in

[src/types/ticket.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L55)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

## Methods

### getHash

▸ **getHash**(): [`Hash`](Hash.md)

#### Returns

[`Hash`](Hash.md)

___

### getLuck

▸ **getLuck**(`preImage`, `challengeResponse`): [`UINT256`](UINT256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `preImage` | [`Hash`](Hash.md) |
| `challengeResponse` | [`Response`](Response.md) |

#### Returns

[`UINT256`](UINT256.md)

___

### getPathPosition

▸ **getPathPosition**(): `number`

#### Returns

`number`

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
| `preImage` | [`Hash`](Hash.md) | preImage of the current onChainSecret |
| `challengeResponse` | [`Response`](Response.md) | response that solves the signed challenge |
| `winProb` | [`UINT256`](UINT256.md) | winning probability of the ticket |

#### Returns

`boolean`

___

### recoverSigner

▸ **recoverSigner**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### serializeUnsigned

▸ **serializeUnsigned**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

___

### verify

▸ **verify**(`pubKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `pubKey` | [`PublicKey`](PublicKey.md) |

#### Returns

`boolean`

___

### create

▸ `Static` **create**(`counterparty`, `challenge`, `epoch`, `index`, `amount`, `winProb`, `channelEpoch`, `signPriv`): [`Ticket`](Ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [`Address`](Address.md) |
| `challenge` | [`Challenge`](Challenge.md) |
| `epoch` | [`UINT256`](UINT256.md) |
| `index` | [`UINT256`](UINT256.md) |
| `amount` | [`Balance`](Balance.md) |
| `winProb` | [`UINT256`](UINT256.md) |
| `channelEpoch` | [`UINT256`](UINT256.md) |
| `signPriv` | `Uint8Array` |

#### Returns

[`Ticket`](Ticket.md)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Ticket`](Ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Ticket`](Ticket.md)
