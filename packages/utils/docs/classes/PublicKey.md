[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PublicKey

# Class: PublicKey

## Table of contents

### Constructors

- [constructor](PublicKey.md#constructor)

### Properties

- [\_address](PublicKey.md#_address)
- [arr](PublicKey.md#arr)

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

#### Defined in

[src/types/publicKey.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L18)

## Properties

### \_address

• `Private` **\_address**: [`Address`](Address.md)

#### Defined in

[src/types/publicKey.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L16)

___

### arr

• `Private` **arr**: `Uint8Array`

#### Defined in

[src/types/publicKey.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L18)

## Accessors

### isCompressed

• `get` **isCompressed**(): `boolean`

#### Returns

`boolean`

#### Defined in

[src/types/publicKey.ts:129](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L129)

___

### SIZE\_COMPRESSED

• `Static` `get` **SIZE_COMPRESSED**(): `number`

#### Returns

`number`

#### Defined in

[src/types/publicKey.ts:121](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L121)

___

### SIZE\_UNCOMPRESSED

• `Static` `get` **SIZE_UNCOMPRESSED**(): `number`

#### Returns

`number`

#### Defined in

[src/types/publicKey.ts:125](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L125)

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

[src/types/publicKey.ts:181](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L181)

___

### serializeCompressed

▸ **serializeCompressed**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[src/types/publicKey.ts:160](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L160)

___

### serializeUncompressed

▸ **serializeUncompressed**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[src/types/publicKey.ts:168](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L168)

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[src/types/publicKey.ts:133](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L133)

___

### toCompressedPubKeyHex

▸ **toCompressedPubKeyHex**(): `string`

#### Returns

`string`

#### Defined in

[src/types/publicKey.ts:152](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L152)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[src/types/publicKey.ts:156](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L156)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[src/types/publicKey.ts:177](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L177)

___

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): `string`

#### Returns

`string`

#### Defined in

[src/types/publicKey.ts:148](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L148)

___

### createMock

▸ `Static` **createMock**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[src/types/publicKey.ts:191](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L191)

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

[src/types/publicKey.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L32)

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

#### Defined in

[src/types/publicKey.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L56)

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

[src/types/publicKey.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L95)

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

[src/types/publicKey.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L102)

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

[src/types/publicKey.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L24)

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

[src/types/publicKey.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L20)

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

[src/types/publicKey.ts:106](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L106)

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

[src/types/publicKey.ts:110](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L110)

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

[src/types/publicKey.ts:114](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L114)

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

#### Defined in

[src/types/publicKey.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/publicKey.ts#L91)
