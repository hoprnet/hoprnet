[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PublicKey

# Class: PublicKey

## Table of contents

### Constructors

- [constructor](publickey.md#constructor)

### Accessors

- [SIZE](publickey.md#size)

### Methods

- [eq](publickey.md#eq)
- [serialize](publickey.md#serialize)
- [toAddress](publickey.md#toaddress)
- [toHex](publickey.md#tohex)
- [toPeerId](publickey.md#topeerid)
- [toString](publickey.md#tostring)
- [toUncompressedPubKeyHex](publickey.md#touncompressedpubkeyhex)
- [createMock](publickey.md#createmock)
- [deserialize](publickey.md#deserialize)
- [fromPeerId](publickey.md#frompeerid)
- [fromPrivKey](publickey.md#fromprivkey)
- [fromString](publickey.md#fromstring)
- [fromUncompressedPubKey](publickey.md#fromuncompressedpubkey)

## Constructors

### constructor

• **new PublicKey**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/primitives.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L10)

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
| `b` | [`PublicKey`](publickey.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L74)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:62](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L62)

___

### toAddress

▸ **toAddress**(): [`Address`](address.md)

#### Returns

[`Address`](address.md)

#### Defined in

[types/primitives.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L38)

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

▸ `Static` **createMock**(): [`PublicKey`](publickey.md)

#### Returns

[`PublicKey`](publickey.md)

#### Defined in

[types/primitives.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L82)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`PublicKey`](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`PublicKey`](publickey.md)

#### Defined in

[types/primitives.ts:78](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L78)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`PublicKey`](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`PublicKey`](publickey.md)

#### Defined in

[types/primitives.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L34)

___

### fromPrivKey

▸ `Static` **fromPrivKey**(`privKey`): [`PublicKey`](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | `Uint8Array` |

#### Returns

[`PublicKey`](publickey.md)

#### Defined in

[types/primitives.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L18)

___

### fromString

▸ `Static` **fromString**(`str`): [`PublicKey`](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`PublicKey`](publickey.md)

#### Defined in

[types/primitives.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L51)

___

### fromUncompressedPubKey

▸ `Static` **fromUncompressedPubKey**(`arr`): [`PublicKey`](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`PublicKey`](publickey.md)

#### Defined in

[types/primitives.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L26)
