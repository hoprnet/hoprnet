[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PublicKey

# Class: PublicKey

## Table of contents

### Constructors

- [constructor](PublicKey.md#constructor)

### Properties

- [\_address](PublicKey.md#_address)

### Accessors

- [isCompressed](PublicKey.md#iscompressed)
- [SIZE\_COMPRESSED](PublicKey.md#size_compressed)
- [SIZE\_UNCOMPRESSED](PublicKey.md#size_uncompressed)

### Methods

- [eq](PublicKey.md#eq)
- [serializeCompressed](PublicKey.md#serializecompressed)
- [serializeUncompressed](PublicKey.md#serializeuncompressed)
- [toAddress](PublicKey.md#toaddress)
- [toB58String](PublicKey.md#tob58string)
- [toCompressedPubKeyHex](PublicKey.md#tocompressedpubkeyhex)
- [toPeerId](PublicKey.md#topeerid)
- [toString](PublicKey.md#tostring)
- [toUncompressedPubKeyHex](PublicKey.md#touncompressedpubkeyhex)
- [createMock](PublicKey.md#createmock)
- [deserialize](PublicKey.md#deserialize)
- [fromPeerId](PublicKey.md#frompeerid)
- [fromPeerIdString](PublicKey.md#frompeeridstring)
- [fromPrivKey](PublicKey.md#fromprivkey)
- [fromPrivKeyString](PublicKey.md#fromprivkeystring)
- [fromSignature](PublicKey.md#fromsignature)
- [fromSignatureString](PublicKey.md#fromsignaturestring)
- [fromString](PublicKey.md#fromstring)

## Constructors

### constructor

• `Private` **new PublicKey**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/publicKey.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L11)

## Properties

### \_address

• `Private` **\_address**: [`Address`](Address.md)

#### Defined in

[types/publicKey.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L9)

## Accessors

### isCompressed

• `get` **isCompressed**(): `boolean`

#### Returns

`boolean`

#### Defined in

[types/publicKey.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L75)

___

### SIZE\_COMPRESSED

• `Static` `get` **SIZE_COMPRESSED**(): `number`

#### Returns

`number`

#### Defined in

[types/publicKey.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L67)

___

### SIZE\_UNCOMPRESSED

• `Static` `get` **SIZE_UNCOMPRESSED**(): `number`

#### Returns

`number`

#### Defined in

[types/publicKey.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L71)

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

[types/publicKey.ts:131](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L131)

___

### serializeCompressed

▸ **serializeCompressed**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/publicKey.ts:106](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L106)

___

### serializeUncompressed

▸ **serializeUncompressed**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/publicKey.ts:114](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L114)

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[types/publicKey.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L79)

___

### toB58String

▸ **toB58String**(): `string`

#### Returns

`string`

#### Defined in

[types/publicKey.ts:127](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L127)

___

### toCompressedPubKeyHex

▸ **toCompressedPubKeyHex**(): `string`

#### Returns

`string`

#### Defined in

[types/publicKey.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L98)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[types/publicKey.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L102)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[types/publicKey.ts:123](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L123)

___

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): `string`

#### Returns

`string`

#### Defined in

[types/publicKey.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L94)

___

### createMock

▸ `Static` **createMock**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/publicKey.ts:141](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L141)

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

[types/publicKey.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L25)

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

[types/publicKey.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L44)

___

### fromPeerIdString

▸ `Static` **fromPeerIdString**(`peerIdString`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerIdString` | `string` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/publicKey.ts:48](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L48)

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

[types/publicKey.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L17)

___

### fromPrivKeyString

▸ `Static` **fromPrivKeyString**(`privKey`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | `string` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/publicKey.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L13)

___

### fromSignature

▸ `Static` **fromSignature**(`hash`, `signature`, `v`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `hash` | `Uint8Array` |
| `signature` | `Uint8Array` |
| `v` | `number` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/publicKey.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L52)

___

### fromSignatureString

▸ `Static` **fromSignatureString**(`hash`, `r`, `s`, `v`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `hash` | `string` |
| `r` | `string` |
| `s` | `string` |
| `v` | `number` |

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/publicKey.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L56)

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

[types/publicKey.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L60)
