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
- [toUncompressedPubKeyHex](publickey.md#touncompressedpubkeyhex)
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

[types/primitives.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L55)

## Methods

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [PublicKey](publickey.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L67)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:59](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L59)

___

### toAddress

▸ **toAddress**(): [Address](address.md)

#### Returns

[Address](address.md)

#### Defined in

[types/primitives.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L38)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:63](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L63)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[types/primitives.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L47)

___

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L42)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [PublicKey](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[PublicKey](publickey.md)

#### Defined in

[types/primitives.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L34)

___

### fromPrivKey

▸ `Static` **fromPrivKey**(`privKey`): [PublicKey](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | `Uint8Array` |

#### Returns

[PublicKey](publickey.md)

#### Defined in

[types/primitives.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L18)

___

### fromString

▸ `Static` **fromString**(`str`): [PublicKey](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[PublicKey](publickey.md)

#### Defined in

[types/primitives.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L51)

___

### fromUncompressedPubKey

▸ `Static` **fromUncompressedPubKey**(`arr`): [PublicKey](publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[PublicKey](publickey.md)

#### Defined in

[types/primitives.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L26)
