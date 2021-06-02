[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HalfKeyChallenge

# Class: HalfKeyChallenge

## Hierarchy

- [CurvePoint](curvepoint.md)

  ↳ **HalfKeyChallenge**

## Table of contents

### Constructors

- [constructor](halfkeychallenge.md#constructor)

### Accessors

- [SIZE](halfkeychallenge.md#size)

### Methods

- [eq](halfkeychallenge.md#eq)
- [serialize](halfkeychallenge.md#serialize)
- [toAddress](halfkeychallenge.md#toaddress)
- [toHex](halfkeychallenge.md#tohex)
- [toPeerId](halfkeychallenge.md#topeerid)
- [toUncompressedCurvePoint](halfkeychallenge.md#touncompressedcurvepoint)
- [fromExponent](halfkeychallenge.md#fromexponent)
- [fromPeerId](halfkeychallenge.md#frompeerid)
- [fromString](halfkeychallenge.md#fromstring)
- [fromUncompressedUncompressedCurvePoint](halfkeychallenge.md#fromuncompresseduncompressedcurvepoint)

## Constructors

### constructor

• **new HalfKeyChallenge**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Inherited from

[CurvePoint](curvepoint.md).[constructor](curvepoint.md#constructor)

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
| `b` | [CurvePoint](curvepoint.md) |

#### Returns

`boolean`

#### Inherited from

[CurvePoint](curvepoint.md).[eq](curvepoint.md#eq)

#### Defined in

[types/curvePoint.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L66)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Inherited from

[CurvePoint](curvepoint.md).[serialize](curvepoint.md#serialize)

#### Defined in

[types/curvePoint.ts:58](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L58)

___

### toAddress

▸ **toAddress**(): [Address](address.md)

#### Returns

[Address](address.md)

#### Inherited from

[CurvePoint](curvepoint.md).[toAddress](curvepoint.md#toaddress)

#### Defined in

[types/curvePoint.ts:37](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L37)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Inherited from

[CurvePoint](curvepoint.md).[toHex](curvepoint.md#tohex)

#### Defined in

[types/curvePoint.ts:62](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L62)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Inherited from

[CurvePoint](curvepoint.md).[toPeerId](curvepoint.md#topeerid)

#### Defined in

[types/curvePoint.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L46)

___

### toUncompressedCurvePoint

▸ **toUncompressedCurvePoint**(): `string`

#### Returns

`string`

#### Inherited from

[CurvePoint](curvepoint.md).[toUncompressedCurvePoint](curvepoint.md#touncompressedcurvepoint)

#### Defined in

[types/curvePoint.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L41)

___

### fromExponent

▸ `Static` **fromExponent**(`exponent`): [CurvePoint](curvepoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[CurvePoint](curvepoint.md)

#### Inherited from

[CurvePoint](curvepoint.md).[fromExponent](curvepoint.md#fromexponent)

#### Defined in

[types/curvePoint.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L17)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [CurvePoint](curvepoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[CurvePoint](curvepoint.md)

#### Inherited from

[CurvePoint](curvepoint.md).[fromPeerId](curvepoint.md#frompeerid)

#### Defined in

[types/curvePoint.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L33)

___

### fromString

▸ `Static` **fromString**(`str`): [CurvePoint](curvepoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[CurvePoint](curvepoint.md)

#### Inherited from

[CurvePoint](curvepoint.md).[fromString](curvepoint.md#fromstring)

#### Defined in

[types/curvePoint.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L50)

___

### fromUncompressedUncompressedCurvePoint

▸ `Static` **fromUncompressedUncompressedCurvePoint**(`arr`): [CurvePoint](curvepoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[CurvePoint](curvepoint.md)

#### Inherited from

[CurvePoint](curvepoint.md).[fromUncompressedUncompressedCurvePoint](curvepoint.md#fromuncompresseduncompressedcurvepoint)

#### Defined in

[types/curvePoint.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L25)
