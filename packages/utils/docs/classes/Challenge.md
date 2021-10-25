[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Challenge

# Class: Challenge

## Hierarchy

- [`CurvePoint`](CurvePoint.md)

  ↳ **`Challenge`**

## Table of contents

### Constructors

- [constructor](Challenge.md#constructor)

### Accessors

- [SIZE](Challenge.md#size)

### Methods

- [eq](Challenge.md#eq)
- [serialize](Challenge.md#serialize)
- [toAddress](Challenge.md#toaddress)
- [toEthereumChallenge](Challenge.md#toethereumchallenge)
- [toHex](Challenge.md#tohex)
- [toPeerId](Challenge.md#topeerid)
- [toUncompressedCurvePoint](Challenge.md#touncompressedcurvepoint)
- [fromExponent](Challenge.md#fromexponent)
- [fromHintAndShare](Challenge.md#fromhintandshare)
- [fromOwnShareAndHalfKey](Challenge.md#fromownshareandhalfkey)
- [fromPeerId](Challenge.md#frompeerid)
- [fromString](Challenge.md#fromstring)
- [fromUncompressedUncompressedCurvePoint](Challenge.md#fromuncompresseduncompressedcurvepoint)

## Constructors

### constructor

• **new Challenge**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Inherited from

[CurvePoint](CurvePoint.md).[constructor](CurvePoint.md#constructor)

#### Defined in

[types/curvePoint.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L11)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Inherited from

CurvePoint.SIZE

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

#### Inherited from

[CurvePoint](CurvePoint.md).[eq](CurvePoint.md#eq)

#### Defined in

[types/curvePoint.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L66)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Inherited from

[CurvePoint](CurvePoint.md).[serialize](CurvePoint.md#serialize)

#### Defined in

[types/curvePoint.ts:58](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L58)

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Inherited from

[CurvePoint](CurvePoint.md).[toAddress](CurvePoint.md#toaddress)

#### Defined in

[types/curvePoint.ts:37](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L37)

___

### toEthereumChallenge

▸ **toEthereumChallenge**(): [`EthereumChallenge`](EthereumChallenge.md)

#### Returns

[`EthereumChallenge`](EthereumChallenge.md)

#### Defined in

[types/challenge.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/challenge.ts#L20)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Inherited from

[CurvePoint](CurvePoint.md).[toHex](CurvePoint.md#tohex)

#### Defined in

[types/curvePoint.ts:62](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L62)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Inherited from

[CurvePoint](CurvePoint.md).[toPeerId](CurvePoint.md#topeerid)

#### Defined in

[types/curvePoint.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L46)

___

### toUncompressedCurvePoint

▸ **toUncompressedCurvePoint**(): `string`

#### Returns

`string`

#### Inherited from

[CurvePoint](CurvePoint.md).[toUncompressedCurvePoint](CurvePoint.md#touncompressedcurvepoint)

#### Defined in

[types/curvePoint.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L41)

___

### fromExponent

▸ `Static` **fromExponent**(`exponent`): [`Challenge`](Challenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `exponent` | `Uint8Array` |

#### Returns

[`Challenge`](Challenge.md)

#### Overrides

[CurvePoint](CurvePoint.md).[fromExponent](CurvePoint.md#fromexponent)

#### Defined in

[types/challenge.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/challenge.ts#L8)

___

### fromHintAndShare

▸ `Static` **fromHintAndShare**(`ownShare`, `hint`): [`Challenge`](Challenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ownShare` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |
| `hint` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |

#### Returns

[`Challenge`](Challenge.md)

#### Defined in

[types/challenge.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/challenge.ts#L12)

___

### fromOwnShareAndHalfKey

▸ `Static` **fromOwnShareAndHalfKey**(`ownShare`, `halfKey`): [`Challenge`](Challenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ownShare` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |
| `halfKey` | [`HalfKey`](HalfKey.md) |

#### Returns

[`Challenge`](Challenge.md)

#### Defined in

[types/challenge.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/challenge.ts#L16)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`CurvePoint`](CurvePoint.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`CurvePoint`](CurvePoint.md)

#### Inherited from

[CurvePoint](CurvePoint.md).[fromPeerId](CurvePoint.md#frompeerid)

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

#### Inherited from

[CurvePoint](CurvePoint.md).[fromString](CurvePoint.md#fromstring)

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

#### Inherited from

[CurvePoint](CurvePoint.md).[fromUncompressedUncompressedCurvePoint](CurvePoint.md#fromuncompresseduncompressedcurvepoint)

#### Defined in

[types/curvePoint.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/curvePoint.ts#L25)
