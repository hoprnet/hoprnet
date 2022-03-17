[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PublicKey

# Class: PublicKey

## Table of contents

### Constructors

- [constructor](PublicKey.md#constructor)

### Properties

- [\_address](PublicKey.md#_address)

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
- [fromPeerIdString](PublicKey.md#frompeeridstring)
- [fromPrivKey](PublicKey.md#fromprivkey)
- [fromSignature](PublicKey.md#fromsignature)
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

[types/primitives.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L14)

## Properties

### \_address

• `Private` **\_address**: [`Address`](Address.md)

#### Defined in

[types/primitives.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L12)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L76)

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

[types/primitives.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L96)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:80](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L80)

___

### toAddress

▸ **toAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[types/primitives.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L50)

___

### toB58String

▸ **toB58String**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L92)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:84](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L84)

___

### toPeerId

▸ **toPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[types/primitives.ts:65](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L65)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L88)

___

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L60)

___

### createMock

▸ `Static` **createMock**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

#### Defined in

[types/primitives.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L104)

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

[types/primitives.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L100)

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

[types/primitives.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L36)

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

[types/primitives.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L40)

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

[types/primitives.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L20)

___

### fromSignature

▸ `Static` **fromSignature**(`hash`, `r`, `s`, `v`): [`PublicKey`](PublicKey.md)

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

[types/primitives.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L44)

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

[types/primitives.ts:69](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L69)

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

[types/primitives.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L28)
