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

#### Defined in

[types/halfKeyChallenge.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L12)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/halfKeyChallenge.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L55)

## Methods

### clone

▸ **clone**(): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L67)

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |

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

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

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

▸ `Static` **fromExponent**(`exponent`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L18)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L34)

___

### fromString

▸ `Static` **fromString**(`str`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L51)

___

### fromUncompressedUncompressedCurvePoint

▸ `Static` **fromUncompressedUncompressedCurvePoint**(`arr`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Defined in

[types/halfKeyChallenge.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L26)
