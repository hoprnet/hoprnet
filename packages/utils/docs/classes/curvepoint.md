[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / CurvePoint

# Class: CurvePoint

## Hierarchy

- **`CurvePoint`**

  ↳ [`Challenge`](challenge.md)

## Table of contents

### Constructors

- [constructor](curvepoint.md#constructor)

### Accessors

- [SIZE](curvepoint.md#size)

### Methods

- [eq](curvepoint.md#eq)
- [serialize](curvepoint.md#serialize)
- [toAddress](curvepoint.md#toaddress)
- [toHex](curvepoint.md#tohex)
- [toPeerId](curvepoint.md#topeerid)
- [toUncompressedCurvePoint](curvepoint.md#touncompressedcurvepoint)
- [fromExponent](curvepoint.md#fromexponent)
- [fromPeerId](curvepoint.md#frompeerid)
- [fromString](curvepoint.md#fromstring)
- [fromUncompressedUncompressedCurvePoint](curvepoint.md#fromuncompresseduncompressedcurvepoint)

## Constructors

### constructor

• **new CurvePoint**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/curvePoint.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L9)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/curvePoint.ts:54](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L54)

## Methods

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`CurvePoint`](curvepoint.md) |

#### Returns

`boolean`

#### Defined in

[types/curvePoint.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L66)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/curvePoint.ts:58](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L58)

___

### toAddress

▸ **toAddress**(): [`Address`](address.md)

#### Returns

[`Address`](address.md)

#### Defined in

[types/curvePoint.ts:37](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L37)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/curvePoint.ts:62](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L62)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[types/curvePoint.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L46)

___

### toUncompressedCurvePoint

▸ **toUncompressedCurvePoint**(): `string`

#### Returns

`string`

#### Defined in

[types/curvePoint.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L41)

___

### fromExponent

▸ `Static` **fromExponent**(`exponent`): [`CurvePoint`](curvepoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[`CurvePoint`](curvepoint.md)

#### Defined in

[types/curvePoint.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L17)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`CurvePoint`](curvepoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`CurvePoint`](curvepoint.md)

#### Defined in

[types/curvePoint.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L33)

___

### fromString

▸ `Static` **fromString**(`str`): [`CurvePoint`](curvepoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`CurvePoint`](curvepoint.md)

#### Defined in

[types/curvePoint.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L50)

___

### fromUncompressedUncompressedCurvePoint

▸ `Static` **fromUncompressedUncompressedCurvePoint**(`arr`): [`CurvePoint`](curvepoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`CurvePoint`](curvepoint.md)

#### Defined in

[types/curvePoint.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L25)
