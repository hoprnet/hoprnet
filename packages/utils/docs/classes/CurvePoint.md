[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / CurvePoint

# Class: CurvePoint

## Hierarchy

- **`CurvePoint`**

  ↳ [`Challenge`](Challenge.md)

## Table of contents

### Constructors

- [constructor](CurvePoint.md#constructor)

### Accessors

- [SIZE](CurvePoint.md#size)

### Methods

- [eq](CurvePoint.md#eq)
- [serialize](CurvePoint.md#serialize)
- [toAddress](CurvePoint.md#toaddress)
- [toHex](CurvePoint.md#tohex)
- [toPeerId](CurvePoint.md#topeerid)
- [toUncompressedCurvePoint](CurvePoint.md#touncompressedcurvepoint)
- [fromExponent](CurvePoint.md#fromexponent)
- [fromPeerId](CurvePoint.md#frompeerid)
- [fromString](CurvePoint.md#fromstring)
- [fromUncompressedUncompressedCurvePoint](CurvePoint.md#fromuncompresseduncompressedcurvepoint)

## Constructors

### constructor

• **new CurvePoint**(`arr`)

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

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`CurvePoint`](CurvePoint.md) |

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

### fromExponent

▸ `Static` **fromExponent**(`exponent`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[`CurvePoint`](CurvePoint.md)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`CurvePoint`](CurvePoint.md)

___

### fromString

▸ `Static` **fromString**(`str`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`CurvePoint`](CurvePoint.md)

___

### fromUncompressedUncompressedCurvePoint

▸ `Static` **fromUncompressedUncompressedCurvePoint**(`arr`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`CurvePoint`](CurvePoint.md)
