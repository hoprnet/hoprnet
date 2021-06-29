[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Address

# Class: Address

## Table of contents

### Constructors

- [constructor](address.md#constructor)

### Accessors

- [SIZE](address.md#size)

### Methods

- [compare](address.md#compare)
- [eq](address.md#eq)
- [lt](address.md#lt)
- [serialize](address.md#serialize)
- [sortPair](address.md#sortpair)
- [toHex](address.md#tohex)
- [createMock](address.md#createmock)
- [deserialize](address.md#deserialize)
- [fromString](address.md#fromstring)

## Constructors

### constructor

• **new Address**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/primitives.ts:87](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L87)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L96)

## Methods

### compare

▸ **compare**(`b`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](address.md) |

#### Returns

`number`

#### Defined in

[types/primitives.ts:120](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L120)

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](address.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L116)

___

### lt

▸ **lt**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](address.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:124](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L124)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:108](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L108)

___

### sortPair

▸ **sortPair**(`b`): [[`Address`](address.md), [`Address`](address.md)]

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](address.md) |

#### Returns

[[`Address`](address.md), [`Address`](address.md)]

#### Defined in

[types/primitives.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L128)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L112)

___

### createMock

▸ `Static` **createMock**(): [`Address`](address.md)

#### Returns

[`Address`](address.md)

#### Defined in

[types/primitives.ts:132](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L132)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Address`](address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Address`](address.md)

#### Defined in

[types/primitives.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L104)

___

### fromString

▸ `Static` **fromString**(`str`): [`Address`](address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`Address`](address.md)

#### Defined in

[types/primitives.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L100)
