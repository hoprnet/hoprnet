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

[types/response.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L8)

## Properties

### SIZE

▪ `Static` **SIZE**: `number`

#### Defined in

[types/response.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L39)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/response.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L31)

___

### toChallenge

▸ **toChallenge**(): [Challenge](challenge.md)

#### Returns

[Challenge](challenge.md)

#### Defined in

[types/response.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L35)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/response.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L27)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [Response](response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[Response](response.md)

#### Defined in

[types/response.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L23)

___

### fromHalfKeys

▸ `Static` **fromHalfKeys**(`firstHalfKey`, `secondHalfKey`): [Response](response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `firstHalfKey` | [HalfKey](halfkey.md) |
| `secondHalfKey` | [HalfKey](halfkey.md) |

#### Returns

[Response](response.md)

#### Defined in

[types/response.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L19)
