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

#### Defined in

[types/curvePoint.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L11)

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
| `b` | [`CurvePoint`](CurvePoint.md) |

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

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

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

▸ `Static` **fromExponent**(`exponent`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[`CurvePoint`](CurvePoint.md)

#### Defined in

[types/curvePoint.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L17)

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

[types/curvePoint.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L33)

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

[types/curvePoint.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L50)

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

[types/curvePoint.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L25)
