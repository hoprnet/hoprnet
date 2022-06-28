[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HalfKeyChallenge

# Class: HalfKeyChallenge

## Table of contents

### Constructors

- [constructor](HalfKeyChallenge.md#constructor)

### Accessors

- [SIZE](HalfKeyChallenge.md#size)

### Methods

- [clone](HalfKeyChallenge.md#clone)
- [eq](HalfKeyChallenge.md#eq)
- [serialize](HalfKeyChallenge.md#serialize)
- [toAddress](HalfKeyChallenge.md#toaddress)
- [toHex](HalfKeyChallenge.md#tohex)
- [toPeerId](HalfKeyChallenge.md#topeerid)
- [toUncompressedCurvePoint](HalfKeyChallenge.md#touncompressedcurvepoint)
- [deserialize](HalfKeyChallenge.md#deserialize)
- [fromExponent](HalfKeyChallenge.md#fromexponent)
- [fromPeerId](HalfKeyChallenge.md#frompeerid)
- [fromString](HalfKeyChallenge.md#fromstring)
- [fromUncompressedUncompressedCurvePoint](HalfKeyChallenge.md#fromuncompresseduncompressedcurvepoint)

## Constructors

### constructor

• **new HalfKeyChallenge**(`arr`)

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

### clone

▸ **clone**(): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |

#### Returns

`boolean`

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

___

### toUncompressedCurvePoint

▸ **toUncompressedCurvePoint**(): `string`

#### Returns

`string`

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

___

### fromExponent

▸ `Static` **fromExponent**(`exponent`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

___

### fromString

▸ `Static` **fromString**(`str`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

___

### fromUncompressedUncompressedCurvePoint

▸ `Static` **fromUncompressedUncompressedCurvePoint**(`arr`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)
