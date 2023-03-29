[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / CurvePoint

# Class: CurvePoint

## Hierarchy

- **`CurvePoint`**

  ↳ [`Challenge`](Challenge.md)

## Table of contents

### Constructors

- [constructor](CurvePoint.md#constructor)

### Properties

- [arr](CurvePoint.md#arr)

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

#### Defined in

[utils/src/types/curvePoint.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L12)

## Properties

### arr

• `Private` **arr**: `Uint8Array`

#### Defined in

[utils/src/types/curvePoint.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L12)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[utils/src/types/curvePoint.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L55)

## Methods

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`CurvePoint`](CurvePoint.md) |

#### Returns

`boolean`

#### Defined in

[utils/src/types/curvePoint.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L67)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[utils/src/types/curvePoint.ts:59](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L59)

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[utils/src/types/curvePoint.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L38)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[utils/src/types/curvePoint.ts:63](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L63)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[utils/src/types/curvePoint.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L47)

___

### toUncompressedCurvePoint

▸ **toUncompressedCurvePoint**(): `string`

#### Returns

`string`

#### Defined in

[utils/src/types/curvePoint.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L42)

___

### fromExponent

▸ `Static` **fromExponent**(`exponent`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[`CurvePoint`](CurvePoint.md)

#### Defined in

[utils/src/types/curvePoint.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L18)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`CurvePoint`](CurvePoint.md)

#### Defined in

[utils/src/types/curvePoint.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L34)

___

### fromString

▸ `Static` **fromString**(`str`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`CurvePoint`](CurvePoint.md)

#### Defined in

[utils/src/types/curvePoint.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L51)

___

### fromUncompressedUncompressedCurvePoint

▸ `Static` **fromUncompressedUncompressedCurvePoint**(`arr`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`CurvePoint`](CurvePoint.md)

#### Defined in

[utils/src/types/curvePoint.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L26)
