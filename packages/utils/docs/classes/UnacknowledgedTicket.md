[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / UnacknowledgedTicket

# Class: UnacknowledgedTicket

## Table of contents

### Constructors

- [constructor](UnacknowledgedTicket.md#constructor)

### Properties

- [ownKey](UnacknowledgedTicket.md#ownkey)
- [signer](UnacknowledgedTicket.md#signer)
- [ticket](UnacknowledgedTicket.md#ticket)

### Methods

- [getChallenge](UnacknowledgedTicket.md#getchallenge)
- [getResponse](UnacknowledgedTicket.md#getresponse)
- [serialize](UnacknowledgedTicket.md#serialize)
- [verifyChallenge](UnacknowledgedTicket.md#verifychallenge)
- [verifySignature](UnacknowledgedTicket.md#verifysignature)
- [SIZE](UnacknowledgedTicket.md#size)
- [deserialize](UnacknowledgedTicket.md#deserialize)

## Constructors

### constructor

• **new UnacknowledgedTicket**(`ticket`, `ownKey`, `signer`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [`Ticket`](Ticket.md) |
| `ownKey` | [`HalfKey`](HalfKey.md) |
| `signer` | [`PublicKey`](PublicKey.md) |

## Properties

### ownKey

• `Readonly` **ownKey**: [`HalfKey`](HalfKey.md)

#### Defined in

[src/types/unacknowledgedTicket.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L5)

___

### signer

• `Readonly` **signer**: [`PublicKey`](PublicKey.md)

#### Defined in

[src/types/unacknowledgedTicket.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L5)

___

### ticket

• `Readonly` **ticket**: [`Ticket`](Ticket.md)

#### Defined in

[src/types/unacknowledgedTicket.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L5)

## Methods

### getChallenge

▸ **getChallenge**(): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

___

### getResponse

▸ **getResponse**(`acknowledgement`): [`Response`](Response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [`HalfKey`](HalfKey.md) |

#### Returns

[`Response`](Response.md)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### verifyChallenge

▸ **verifyChallenge**(`acknowledgement`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [`HalfKey`](HalfKey.md) |

#### Returns

`boolean`

___

### verifySignature

▸ **verifySignature**(): `boolean`

#### Returns

`boolean`

___

### SIZE

▸ `Static` **SIZE**(): `number`

#### Returns

`number`

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`UnacknowledgedTicket`](UnacknowledgedTicket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`UnacknowledgedTicket`](UnacknowledgedTicket.md)
