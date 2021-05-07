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

\+ **new PublicKey**(`arr`: _Uint8Array_): [_PublicKey_](types_primitives.publickey.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_PublicKey_](types_primitives.publickey.md)

Defined in: [types/primitives.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L10)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/primitives.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L55)

## Methods

### eq

▸ **eq**(`b`: [_PublicKey_](types_primitives.publickey.md)): _boolean_

#### Parameters

| Name | Type                                         |
| :--- | :------------------------------------------- |
| `b`  | [_PublicKey_](types_primitives.publickey.md) |

**Returns:** _boolean_

Defined in: [types/primitives.ts:67](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L67)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/primitives.ts:59](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L59)

---

### toAddress

▸ **toAddress**(): [_Address_](types_primitives.address.md)

**Returns:** [_Address_](types_primitives.address.md)

Defined in: [types/primitives.ts:38](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L38)

---

### toHex

▸ **toHex**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:63](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L63)

---

### toPeerId

▸ **toPeerId**(): _PeerId_

**Returns:** _PeerId_

Defined in: [types/primitives.ts:47](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L47)

---

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:42](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L42)

---

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`: _PeerId_): [_PublicKey_](types_primitives.publickey.md)

#### Parameters

| Name     | Type     |
| :------- | :------- |
| `peerId` | _PeerId_ |

**Returns:** [_PublicKey_](types_primitives.publickey.md)

Defined in: [types/primitives.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L34)

---

### fromPrivKey

▸ `Static` **fromPrivKey**(`privKey`: _Uint8Array_): [_PublicKey_](types_primitives.publickey.md)

#### Parameters

| Name      | Type         |
| :-------- | :----------- |
| `privKey` | _Uint8Array_ |

**Returns:** [_PublicKey_](types_primitives.publickey.md)

Defined in: [types/primitives.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L18)

---

### fromString

▸ `Static` **fromString**(`str`: _string_): [_PublicKey_](types_primitives.publickey.md)

#### Parameters

| Name  | Type     |
| :---- | :------- |
| `str` | _string_ |

**Returns:** [_PublicKey_](types_primitives.publickey.md)

Defined in: [types/primitives.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L51)

---

### fromUncompressedPubKey

▸ `Static` **fromUncompressedPubKey**(`arr`: _Uint8Array_): [_PublicKey_](types_primitives.publickey.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_PublicKey_](types_primitives.publickey.md)

Defined in: [types/primitives.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L26)
