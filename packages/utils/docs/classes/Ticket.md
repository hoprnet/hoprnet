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

#### Defined in

[types/ticket.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L50)

## Properties

### amount

• `Readonly` **amount**: [`Balance`](Balance.md)

___

### challenge

• `Readonly` **challenge**: [`EthereumChallenge`](EthereumChallenge.md)

___

### channelEpoch

• `Readonly` **channelEpoch**: [`UINT256`](UINT256.md)

___

### counterparty

• `Readonly` **counterparty**: [`Address`](Address.md)

___

### epoch

• `Readonly` **epoch**: [`UINT256`](UINT256.md)

___

### index

• `Readonly` **index**: [`UINT256`](UINT256.md)

___

### signature

• `Readonly` **signature**: [`Signature`](Signature.md)

___

### winProb

• `Readonly` **winProb**: [`UINT256`](UINT256.md)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/ticket.ts:140](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L140)

## Methods

### getHash

▸ **getHash**(): [`Hash`](Hash.md)

#### Returns

[`Hash`](Hash.md)

#### Defined in

[types/ticket.ts:136](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L136)

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
| `preImage` | [`Hash`](Hash.md) | preImage of the current onChainSecret |
| `challengeResponse` | [`Response`](Response.md) | response that solves the signed challenge |
| `winProb` | [`UINT256`](UINT256.md) | winning probability of the ticket |

#### Returns

`boolean`

#### Defined in

[types/ticket.ts:180](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L180)

___

### recoverSigner

▸ **recoverSigner**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

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
| `pubKey` | [`PublicKey`](PublicKey.md) |

#### Returns

`boolean`

#### Defined in

[types/ticket.ts:157](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L157)

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

#### Defined in

[types/ticket.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L61)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Ticket`](Ticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Ticket`](Ticket.md)

#### Defined in

[types/ticket.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ticket.ts#L99)
