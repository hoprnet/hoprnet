[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / UINT256

# Class: UINT256

## Table of contents

### Constructors

- [constructor](UINT256.md#constructor)

### Accessors

- [DUMMY\_INVERSE\_PROBABILITY](UINT256.md#dummy_inverse_probability)
- [SIZE](UINT256.md#size)

### Methods

- [cmp](UINT256.md#cmp)
- [eq](UINT256.md#eq)
- [serialize](UINT256.md#serialize)
- [toBN](UINT256.md#tobn)
- [toHex](UINT256.md#tohex)
- [deserialize](UINT256.md#deserialize)
- [fromInverseProbability](UINT256.md#frominverseprobability)
- [fromString](UINT256.md#fromstring)

## Constructors

### constructor

• **new UINT256**(`bn`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | `BN` |

## Accessors

### DUMMY\_INVERSE\_PROBABILITY

• `Static` `get` **DUMMY_INVERSE_PROBABILITY**(): [`UINT256`](UINT256.md)

#### Returns

[`UINT256`](UINT256.md)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

## Methods

### cmp

▸ **cmp**(`b`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`UINT256`](UINT256.md) |

#### Returns

`number`

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`UINT256`](UINT256.md) |

#### Returns

`boolean`

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`UINT256`](UINT256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`UINT256`](UINT256.md)

___

### fromInverseProbability

▸ `Static` **fromInverseProbability**(`inverseProb`): [`UINT256`](UINT256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `inverseProb` | `BN` |

#### Returns

[`UINT256`](UINT256.md)

___

### fromString

▸ `Static` **fromString**(`str`): [`UINT256`](UINT256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`UINT256`](UINT256.md)
