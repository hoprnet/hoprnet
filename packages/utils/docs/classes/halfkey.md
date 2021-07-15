[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HalfKey

# Class: HalfKey

## Table of contents

### Constructors

- [constructor](halfkey.md#constructor)

### Properties

- [SIZE](halfkey.md#size)

### Methods

- [clone](halfkey.md#clone)
- [eq](halfkey.md#eq)
- [serialize](halfkey.md#serialize)
- [toChallenge](halfkey.md#tochallenge)
- [toHex](halfkey.md#tohex)
- [deserialize](halfkey.md#deserialize)

## Constructors

### constructor

• **new HalfKey**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/halfKey.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L5)

## Properties

### SIZE

▪ `Static` **SIZE**: `number` = `32`

#### Defined in

[types/halfKey.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L41)

## Methods

### clone

▸ **clone**(): [`HalfKey`](halfkey.md)

#### Returns

[`HalfKey`](halfkey.md)

#### Defined in

[types/halfKey.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L36)

___

### eq

▸ **eq**(`halfKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKey` | [`HalfKey`](halfkey.md) |

#### Returns

`boolean`

#### Defined in

[types/halfKey.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L28)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/halfKey.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L20)

___

### toChallenge

▸ **toChallenge**(): [`HalfKeyChallenge`](halfkeychallenge.md)

#### Returns

[`HalfKeyChallenge`](halfkeychallenge.md)

#### Defined in

[types/halfKey.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L16)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/halfKey.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L24)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`HalfKey`](halfkey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`HalfKey`](halfkey.md)

#### Defined in

[types/halfKey.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L32)
