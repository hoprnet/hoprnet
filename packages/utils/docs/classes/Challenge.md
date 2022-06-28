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

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Inherited from

CurvePoint.SIZE

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

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Inherited from

[CurvePoint](CurvePoint.md).[serialize](CurvePoint.md#serialize)

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Inherited from

[CurvePoint](CurvePoint.md).[toAddress](CurvePoint.md#toaddress)

___

### toEthereumChallenge

▸ **toEthereumChallenge**(): [`EthereumChallenge`](EthereumChallenge.md)

#### Returns

[`EthereumChallenge`](EthereumChallenge.md)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Inherited from

[CurvePoint](CurvePoint.md).[toHex](CurvePoint.md#tohex)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Inherited from

[CurvePoint](CurvePoint.md).[toPeerId](CurvePoint.md#topeerid)

___

### toUncompressedCurvePoint

▸ **toUncompressedCurvePoint**(): `string`

#### Returns

`string`

#### Inherited from

[CurvePoint](CurvePoint.md).[toUncompressedCurvePoint](CurvePoint.md#touncompressedcurvepoint)

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
