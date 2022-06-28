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
- [toBytes32](Address.md#tobytes32)
- [toHex](Address.md#tohex)
- [toString](Address.md#tostring)
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

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

## Methods

### compare

▸ **compare**(`b`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`number`

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`boolean`

___

### lt

▸ **lt**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`boolean`

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### sortPair

▸ **sortPair**(`b`): [[`Address`](Address.md), [`Address`](Address.md)]

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

[[`Address`](Address.md), [`Address`](Address.md)]

___

### toBytes32

▸ **toBytes32**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

___

### createMock

▸ `Static` **createMock**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Address`](Address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Address`](Address.md)

___

### fromString

▸ `Static` **fromString**(`str`): [`Address`](Address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`Address`](Address.md)
