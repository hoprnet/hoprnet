[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Response

# Class: Response

## Table of contents

### Constructors

- [constructor](response.md#constructor)

### Properties

- [SIZE](response.md#size)

### Methods

- [serialize](response.md#serialize)
- [toChallenge](response.md#tochallenge)
- [toHex](response.md#tohex)
- [createMock](response.md#createmock)
- [deserialize](response.md#deserialize)
- [fromHalfKeys](response.md#fromhalfkeys)

## Constructors

### constructor

• **new Response**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/response.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L10)

## Properties

### SIZE

▪ `Static` **SIZE**: `number`

#### Defined in

[types/response.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L45)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/response.ts:37](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L37)

___

### toChallenge

▸ **toChallenge**(): [`Challenge`](challenge.md)

#### Returns

[`Challenge`](challenge.md)

#### Defined in

[types/response.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L41)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/response.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L33)

___

### createMock

▸ `Static` **createMock**(): [`Response`](response.md)

#### Returns

[`Response`](response.md)

#### Defined in

[types/response.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L47)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Response`](response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Response`](response.md)

#### Defined in

[types/response.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L29)

___

### fromHalfKeys

▸ `Static` **fromHalfKeys**(`firstHalfKey`, `secondHalfKey`): [`Response`](response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `firstHalfKey` | [`HalfKey`](halfkey.md) |
| `secondHalfKey` | [`HalfKey`](halfkey.md) |

#### Returns

[`Response`](response.md)

#### Defined in

[types/response.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L25)
