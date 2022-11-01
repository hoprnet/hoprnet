[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Response

# Class: Response

## Table of contents

### Constructors

- [constructor](Response.md#constructor)

### Properties

- [SIZE](Response.md#size)

### Methods

- [serialize](Response.md#serialize)
- [toChallenge](Response.md#tochallenge)
- [toHex](Response.md#tohex)
- [createMock](Response.md#createmock)
- [deserialize](Response.md#deserialize)
- [fromHalfKeys](Response.md#fromhalfkeys)

## Constructors

### constructor

• **new Response**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

## Properties

### SIZE

▪ `Static` **SIZE**: `number` = `SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH`

#### Defined in

[src/types/response.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L45)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toChallenge

▸ **toChallenge**(): [`Challenge`](Challenge.md)

#### Returns

[`Challenge`](Challenge.md)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

___

### createMock

▸ `Static` **createMock**(): [`Response`](Response.md)

#### Returns

[`Response`](Response.md)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Response`](Response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Response`](Response.md)

___

### fromHalfKeys

▸ `Static` **fromHalfKeys**(`firstHalfKey`, `secondHalfKey`): [`Response`](Response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `firstHalfKey` | [`HalfKey`](HalfKey.md) |
| `secondHalfKey` | [`HalfKey`](HalfKey.md) |

#### Returns

[`Response`](Response.md)
