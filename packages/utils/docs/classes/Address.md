[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Address

# Class: Address

## Table of contents

### Constructors

- [constructor](Address.md#constructor)

### Accessors

- [SIZE](Address.md#size)

### Methods

- [compare](Address.md#compare)
- [eq](Address.md#eq)
- [lt](Address.md#lt)
- [serialize](Address.md#serialize)
- [sortPair](Address.md#sortpair)
- [toHex](Address.md#tohex)
- [createMock](Address.md#createmock)
- [deserialize](Address.md#deserialize)
- [fromString](Address.md#fromstring)

## Constructors

### constructor

• **new Address**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/primitives.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L92)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L100)

## Methods

### compare

▸ **compare**(`b`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`number`

#### Defined in

[types/primitives.ts:124](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L124)

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:120](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L120)

___

### lt

▸ **lt**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L128)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L112)

___

### sortPair

▸ **sortPair**(`b`): [[`Address`](Address.md), [`Address`](Address.md)]

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

[[`Address`](Address.md), [`Address`](Address.md)]

#### Defined in

[types/primitives.ts:132](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L132)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L116)

___

### createMock

▸ `Static` **createMock**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[types/primitives.ts:136](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L136)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Address`](Address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Address`](Address.md)

#### Defined in

[types/primitives.ts:108](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L108)

___

### fromString

▸ `Static` **fromString**(`str`): [`Address`](Address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`Address`](Address.md)

#### Defined in

[types/primitives.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L104)
