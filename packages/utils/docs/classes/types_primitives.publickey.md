[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/primitives](../modules/types_primitives.md) / PublicKey

# Class: PublicKey

[types/primitives](../modules/types_primitives.md).PublicKey

## Table of contents

### Constructors

- [constructor](types_primitives.publickey.md#constructor)

### Accessors

- [SIZE](types_primitives.publickey.md#size)

### Methods

- [eq](types_primitives.publickey.md#eq)
- [serialize](types_primitives.publickey.md#serialize)
- [toAddress](types_primitives.publickey.md#toaddress)
- [toHex](types_primitives.publickey.md#tohex)
- [toPeerId](types_primitives.publickey.md#topeerid)
- [toUncompressedPubKeyHex](types_primitives.publickey.md#touncompressedpubkeyhex)
- [fromPeerId](types_primitives.publickey.md#frompeerid)
- [fromPrivKey](types_primitives.publickey.md#fromprivkey)
- [fromString](types_primitives.publickey.md#fromstring)
- [fromUncompressedPubKey](types_primitives.publickey.md#fromuncompressedpubkey)

## Constructors

### constructor

\+ **new PublicKey**(`arr`: *Uint8Array*): [*PublicKey*](types_primitives.publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*PublicKey*](types_primitives.publickey.md)

Defined in: [types/primitives.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L10)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/primitives.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L55)

## Methods

### eq

▸ **eq**(`b`: [*PublicKey*](types_primitives.publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [*PublicKey*](types_primitives.publickey.md) |

**Returns:** *boolean*

Defined in: [types/primitives.ts:67](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L67)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/primitives.ts:59](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L59)

___

### toAddress

▸ **toAddress**(): [*Address*](types_primitives.address.md)

**Returns:** [*Address*](types_primitives.address.md)

Defined in: [types/primitives.ts:38](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L38)

___

### toHex

▸ **toHex**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:63](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L63)

___

### toPeerId

▸ **toPeerId**(): *PeerId*

**Returns:** *PeerId*

Defined in: [types/primitives.ts:47](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L47)

___

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:42](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L42)

___

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`: *PeerId*): [*PublicKey*](types_primitives.publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `peerId` | *PeerId* |

**Returns:** [*PublicKey*](types_primitives.publickey.md)

Defined in: [types/primitives.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L34)

___

### fromPrivKey

▸ `Static` **fromPrivKey**(`privKey`: *Uint8Array*): [*PublicKey*](types_primitives.publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | *Uint8Array* |

**Returns:** [*PublicKey*](types_primitives.publickey.md)

Defined in: [types/primitives.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L18)

___

### fromString

▸ `Static` **fromString**(`str`: *string*): [*PublicKey*](types_primitives.publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | *string* |

**Returns:** [*PublicKey*](types_primitives.publickey.md)

Defined in: [types/primitives.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L51)

___

### fromUncompressedPubKey

▸ `Static` **fromUncompressedPubKey**(`arr`: *Uint8Array*): [*PublicKey*](types_primitives.publickey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*PublicKey*](types_primitives.publickey.md)

Defined in: [types/primitives.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L26)
