[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HalfKeyChallenge

# Class: HalfKeyChallenge

## Table of contents

### Constructors

- [constructor](HalfKeyChallenge.md#constructor)

### Properties

- [arr](HalfKeyChallenge.md#arr)

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

#### Defined in

[utils/src/types/halfKeyChallenge.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L13)

## Properties

### arr

• `Private` **arr**: `Uint8Array`

#### Defined in

[utils/src/types/halfKeyChallenge.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L13)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[utils/src/types/halfKeyChallenge.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L56)

## Methods

### clone

▸ **clone**(): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Defined in

[utils/src/types/halfKeyChallenge.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L72)

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

[utils/src/types/halfKeyChallenge.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L76)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[utils/src/types/halfKeyChallenge.ts:64](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L64)

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[utils/src/types/halfKeyChallenge.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L39)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[utils/src/types/halfKeyChallenge.ts:68](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L68)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[utils/src/types/halfKeyChallenge.ts:48](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L48)

___

### toUncompressedCurvePoint

▸ **toUncompressedCurvePoint**(): `string`

#### Returns

`string`

#### Defined in

[utils/src/types/halfKeyChallenge.ts:43](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L43)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Defined in

[utils/src/types/halfKeyChallenge.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L60)

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

[utils/src/types/halfKeyChallenge.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L19)

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

[utils/src/types/halfKeyChallenge.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L35)

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

[utils/src/types/halfKeyChallenge.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L52)

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

[utils/src/types/halfKeyChallenge.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKeyChallenge.ts#L27)
