[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / UnacknowledgedTicket

# Class: UnacknowledgedTicket

## Table of contents

### Constructors

- [constructor](unacknowledgedticket.md#constructor)

### Properties

- [ownKey](unacknowledgedticket.md#ownkey)
- [signer](unacknowledgedticket.md#signer)
- [ticket](unacknowledgedticket.md#ticket)

### Methods

- [getChallenge](unacknowledgedticket.md#getchallenge)
- [getResponse](unacknowledgedticket.md#getresponse)
- [serialize](unacknowledgedticket.md#serialize)
- [verifyChallenge](unacknowledgedticket.md#verifychallenge)
- [verifySignature](unacknowledgedticket.md#verifysignature)
- [SIZE](unacknowledgedticket.md#size)
- [deserialize](unacknowledgedticket.md#deserialize)

## Constructors

### constructor

• **new UnacknowledgedTicket**(`ticket`, `ownKey`, `signer`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [`Ticket`](ticket.md) |
| `ownKey` | [`HalfKey`](halfkey.md) |
| `signer` | [`PublicKey`](publickey.md) |

#### Defined in

[types/unacknowledgedTicket.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L4)

## Properties

### ownKey

• `Readonly` **ownKey**: [`HalfKey`](halfkey.md)

___

### signer

• `Readonly` **signer**: [`PublicKey`](publickey.md)

___

### ticket

• `Readonly` **ticket**: [`Ticket`](ticket.md)

## Methods

### getChallenge

▸ **getChallenge**(): [`HalfKeyChallenge`](halfkeychallenge.md)

#### Returns

[`HalfKeyChallenge`](halfkeychallenge.md)

#### Defined in

[types/unacknowledgedTicket.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L41)

___

### getResponse

▸ **getResponse**(`acknowledgement`): [`Response`](response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [`HalfKey`](halfkey.md) |

#### Returns

[`Response`](response.md)

#### Defined in

[types/unacknowledgedTicket.ts:37](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L37)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/unacknowledgedTicket.ts:21](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L21)

___

### verifyChallenge

▸ **verifyChallenge**(`acknowledgement`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [`HalfKey`](halfkey.md) |

#### Returns

`boolean`

#### Defined in

[types/unacknowledgedTicket.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L29)

___

### verifySignature

▸ **verifySignature**(): `boolean`

#### Returns

`boolean`

#### Defined in

[types/unacknowledgedTicket.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L33)

___

### SIZE

▸ `Static` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/unacknowledgedTicket.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L45)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`UnacknowledgedTicket`](unacknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`UnacknowledgedTicket`](unacknowledgedticket.md)

#### Defined in

[types/unacknowledgedTicket.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/unacknowledgedTicket.ts#L11)
