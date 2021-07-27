[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PublicKey

# Class: PublicKey

## Table of contents

### Constructors

- [constructor](PublicKey.md#constructor)

### Accessors

- [SIZE](PublicKey.md#size)

### Methods

- [eq](PublicKey.md#eq)
- [serialize](PublicKey.md#serialize)
- [toAddress](PublicKey.md#toaddress)
- [toB58String](PublicKey.md#tob58string)
- [toHex](PublicKey.md#tohex)
- [toPeerId](PublicKey.md#topeerid)
- [toString](PublicKey.md#tostring)
- [toUncompressedPubKeyHex](PublicKey.md#touncompressedpubkeyhex)
- [createMock](PublicKey.md#createmock)
- [deserialize](PublicKey.md#deserialize)
- [fromPeerId](PublicKey.md#frompeerid)
- [fromPrivKey](PublicKey.md#fromprivkey)
- [fromString](PublicKey.md#fromstring)
- [fromUncompressedPubKey](PublicKey.md#fromuncompressedpubkey)

## Constructors

### constructor

• **new PublicKey**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/primitives.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L12)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:58](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L58)

## Methods

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`PublicKey`](PublicKey.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:78](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L78)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:62](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L62)

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[types/primitives.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L38)

___

### toB58String

▸ **toB58String**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L74)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L66)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[types/primitives.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L47)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:70](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L70)

___

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L42)

___

### createMock

▸ `Static` **createMock**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/primitives.ts:86](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L86)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/primitives.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L82)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/primitives.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L34)

___

### fromPrivKey

▸ `Static` **fromPrivKey**(`privKey`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | `Uint8Array` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/primitives.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L18)

___

### fromString

▸ `Static` **fromString**(`str`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/primitives.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L51)

___

### fromUncompressedPubKey

▸ `Static` **fromUncompressedPubKey**(`arr`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/primitives.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L26)
