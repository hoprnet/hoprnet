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
- [toString](ticket.md#tostring)
- [verify](ticket.md#verify)
- [create](ticket.md#create)
- [deserialize](ticket.md#deserialize)

## Constructors

### constructor

• **new Ticket**(`counterparty`, `challenge`, `epoch`, `index`, `amount`, `winProb`, `channelIteration`, `signature`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [`Address`](address.md) |
| `challenge` | [`EthereumChallenge`](ethereumchallenge.md) |
| `epoch` | [`UINT256`](uint256.md) |
| `index` | [`UINT256`](uint256.md) |
| `amount` | [`Balance`](balance.md) |
| `winProb` | [`UINT256`](uint256.md) |
| `channelIteration` | [`UINT256`](uint256.md) |
| `signature` | [`Signature`](signature.md) |

#### Defined in

[types/ticket.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L49)

## Properties

### amount

• `Readonly` **amount**: [`Balance`](balance.md)

___

### challenge

• `Readonly` **challenge**: [`EthereumChallenge`](ethereumchallenge.md)

___

### channelIteration

• `Readonly` **channelIteration**: [`UINT256`](uint256.md)

___

### counterparty

• `Readonly` **counterparty**: [`Address`](address.md)

___

### epoch

• `Readonly` **epoch**: [`UINT256`](uint256.md)

___

### index

• `Readonly` **index**: [`UINT256`](uint256.md)

___

### signature

• `Readonly` **signature**: [`Signature`](signature.md)

___

### winProb

• `Readonly` **winProb**: [`UINT256`](uint256.md)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/ticket.ts:140](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L140)

## Methods

### getHash

▸ **getHash**(): [`Hash`](hash.md)

#### Returns

[`Hash`](hash.md)

#### Defined in

[types/ticket.ts:136](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L136)

___

### getLuck

▸ **getLuck**(`preImage`, `challengeResponse`): [`UINT256`](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `preImage` | [`Hash`](hash.md) |
| `challengeResponse` | [`Response`](response.md) |

#### Returns

[`UINT256`](uint256.md)

#### Defined in

[types/ticket.ts:161](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L161)

___

### getPathPosition

▸ **getPathPosition**(): `number`

#### Returns

`number`

#### Defined in

[types/ticket.ts:185](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L185)

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
| `preImage` | [`Hash`](hash.md) | preImage of the current onChainSecret |
| `challengeResponse` | [`Response`](response.md) | response that solves the signed challenge |
| `winProb` | [`UINT256`](uint256.md) | winning probability of the ticket |

#### Returns

`boolean`

#### Defined in

[types/ticket.ts:180](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L180)

___

### recoverSigner

▸ **recoverSigner**(): [`PublicKey`](publickey.md)

#### Returns

[`PublicKey`](publickey.md)

#### Defined in

[types/ticket.ts:153](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L153)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/ticket.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L91)

___

### serializeUnsigned

▸ **serializeUnsigned**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/ticket.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L95)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[types/ticket.ts:122](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L122)

___

### verify

▸ **verify**(`pubKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `pubKey` | [`PublicKey`](publickey.md) |

#### Returns

`boolean`

#### Defined in

[types/ticket.ts:157](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L157)

___

### create

▸ `Static` **create**(`counterparty`, `challenge`, `epoch`, `index`, `amount`, `winProb`, `channelIteration`, `signPriv`): [`Ticket`](ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [`Address`](address.md) |
| `challenge` | [`Challenge`](challenge.md) |
| `epoch` | [`UINT256`](uint256.md) |
| `index` | [`UINT256`](uint256.md) |
| `amount` | [`Balance`](balance.md) |
| `winProb` | [`UINT256`](uint256.md) |
| `channelIteration` | [`UINT256`](uint256.md) |
| `signPriv` | `Uint8Array` |

#### Returns

[`Ticket`](ticket.md)

#### Defined in

[types/ticket.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L61)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Ticket`](ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Ticket`](ticket.md)

#### Defined in

[types/ticket.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L99)
