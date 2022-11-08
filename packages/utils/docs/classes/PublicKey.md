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
- [toCompressedPubKeyHex](PublicKey.md#tocompressedpubkeyhex)
- [toPeerId](PublicKey.md#topeerid)
- [toString](PublicKey.md#tostring)
- [toUncompressedPubKeyHex](PublicKey.md#touncompressedpubkeyhex)
- [createMock](PublicKey.md#createmock)
- [deserialize](PublicKey.md#deserialize)
- [deserializeArray](PublicKey.md#deserializearray)
- [fromPeerId](PublicKey.md#frompeerid)
- [fromPeerIdString](PublicKey.md#frompeeridstring)
- [fromPrivKey](PublicKey.md#fromprivkey)
- [fromPrivKeyString](PublicKey.md#fromprivkeystring)
- [fromSignature](PublicKey.md#fromsignature)
- [fromSignatureString](PublicKey.md#fromsignaturestring)
- [fromString](PublicKey.md#fromstring)
- [serializeArray](PublicKey.md#serializearray)

## Constructors

### constructor

• `Private` **new PublicKey**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

## Properties

### \_address

• `Private` **\_address**: [`Address`](Address.md)

#### Defined in

[types/publicKey.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L16)

## Accessors

### isCompressed

• `get` **isCompressed**(): `boolean`

#### Returns

`boolean`

___

### SIZE\_COMPRESSED

• `Static` `get` **SIZE_COMPRESSED**(): `number`

#### Returns

`number`

___

### SIZE\_UNCOMPRESSED

• `Static` `get` **SIZE_UNCOMPRESSED**(): `number`

#### Returns

`number`

## Methods

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`PublicKey`](PublicKey.md) |

#### Returns

`boolean`

___

### serializeCompressed

▸ **serializeCompressed**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### serializeUncompressed

▸ **serializeUncompressed**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

___

### toCompressedPubKeyHex

▸ **toCompressedPubKeyHex**(): `string`

#### Returns

`string`

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

___

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): `string`

#### Returns

`string`

___

### createMock

▸ `Static` **createMock**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`PublicKey`](PublicKey.md)

___

### deserializeArray

▸ `Static` **deserializeArray**(`arr`): [`PublicKey`](PublicKey.md)[]

Deserializes a Uint8Array containing serialized publicKeys

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `arr` | `Uint8Array` | u8a containing serialized pubkeys |

#### Returns

[`PublicKey`](PublicKey.md)[]

an array of deserialized publicKeys

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | `PeerId` |

#### Returns

[`PublicKey`](PublicKey.md)

___

### fromPeerIdString

▸ `Static` **fromPeerIdString**(`peerIdString`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerIdString` | `string` |

#### Returns

[`PublicKey`](PublicKey.md)

___

### fromPrivKey

▸ `Static` **fromPrivKey**(`privKey`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | `Uint8Array` |

#### Returns

[`PublicKey`](PublicKey.md)

___

### fromPrivKeyString

▸ `Static` **fromPrivKeyString**(`privKey`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | `string` |

#### Returns

[`PublicKey`](PublicKey.md)

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

___

### fromString

▸ `Static` **fromString**(`str`): [`PublicKey`](PublicKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`PublicKey`](PublicKey.md)

___

### serializeArray

▸ `Static` **serializeArray**(`pKeys`): `Uint8Array`

Serializes an array of publicKeys

#### Parameters

| Name | Type |
| :------ | :------ |
| `pKeys` | [`PublicKey`](PublicKey.md)[] |

#### Returns

`Uint8Array`

a Uint8Array containing the given publicKeys
