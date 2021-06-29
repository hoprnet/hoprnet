[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HalfKeyChallenge

# Class: HalfKeyChallenge

## Table of contents

### Constructors

- [constructor](halfkeychallenge.md#constructor)

### Accessors

- [SIZE](halfkeychallenge.md#size)

### Methods

- [clone](halfkeychallenge.md#clone)
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

#### Defined in

[types/halfKeyChallenge.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L10)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/halfKeyChallenge.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L55)

## Methods

### clone

▸ **clone**(): [`HalfKeyChallenge`](halfkeychallenge.md)

#### Returns

[`HalfKeyChallenge`](halfkeychallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L67)

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`HalfKeyChallenge`](halfkeychallenge.md) |

#### Returns

`boolean`

#### Defined in

[types/halfKeyChallenge.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L71)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/halfKeyChallenge.ts:59](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L59)

___

### toAddress

▸ **toAddress**(): [`Address`](address.md)

#### Returns

[`Address`](address.md)

#### Defined in

[types/halfKeyChallenge.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L38)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/halfKeyChallenge.ts:63](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L63)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[types/halfKeyChallenge.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L47)

___

### toUncompressedCurvePoint

▸ **toUncompressedCurvePoint**(): `string`

#### Returns

`string`

#### Defined in

[types/halfKeyChallenge.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L42)

___

### fromExponent

▸ `Static` **fromExponent**(`exponent`): [`HalfKeyChallenge`](halfkeychallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[`HalfKeyChallenge`](halfkeychallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L18)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`HalfKeyChallenge`](halfkeychallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`HalfKeyChallenge`](halfkeychallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L34)

___

### fromString

▸ `Static` **fromString**(`str`): [`HalfKeyChallenge`](halfkeychallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`HalfKeyChallenge`](halfkeychallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L51)

___

### fromUncompressedUncompressedCurvePoint

▸ `Static` **fromUncompressedUncompressedCurvePoint**(`arr`): [`HalfKeyChallenge`](halfkeychallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`HalfKeyChallenge`](halfkeychallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L26)
