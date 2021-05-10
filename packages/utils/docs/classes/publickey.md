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

\+ **new PublicKey**(`arr`: _Uint8Array_): [_PublicKey_](publickey.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_PublicKey_](publickey.md)

Defined in: [types/primitives.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L10)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/primitives.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L55)

## Methods

### eq

▸ **eq**(`b`: [_PublicKey_](publickey.md)): _boolean_

#### Parameters

| Name | Type                        |
| :--- | :-------------------------- |
| `b`  | [_PublicKey_](publickey.md) |

**Returns:** _boolean_

Defined in: [types/primitives.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L67)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/primitives.ts:59](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L59)

---

### toAddress

▸ **toAddress**(): [_Address_](address.md)

**Returns:** [_Address_](address.md)

Defined in: [types/primitives.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L38)

---

### toHex

▸ **toHex**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:63](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L63)

---

### toPeerId

▸ **toPeerId**(): _PeerId_

**Returns:** _PeerId_

Defined in: [types/primitives.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L47)

---

### toUncompressedPubKeyHex

▸ **toUncompressedPubKeyHex**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L42)

---

### fromPeerId

▸ `Static` **fromPeerId**(`peerId`: _PeerId_): [_PublicKey_](publickey.md)

#### Parameters

| Name     | Type     |
| :------- | :------- |
| `peerId` | _PeerId_ |

**Returns:** [_PublicKey_](publickey.md)

Defined in: [types/primitives.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L34)

---

### fromPrivKey

▸ `Static` **fromPrivKey**(`privKey`: _Uint8Array_): [_PublicKey_](publickey.md)

#### Parameters

| Name      | Type         |
| :-------- | :----------- |
| `privKey` | _Uint8Array_ |

**Returns:** [_PublicKey_](publickey.md)

Defined in: [types/primitives.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L18)

---

### fromString

▸ `Static` **fromString**(`str`: _string_): [_PublicKey_](publickey.md)

#### Parameters

| Name  | Type     |
| :---- | :------- |
| `str` | _string_ |

**Returns:** [_PublicKey_](publickey.md)

Defined in: [types/primitives.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L51)

---

### fromUncompressedPubKey

▸ `Static` **fromUncompressedPubKey**(`arr`: _Uint8Array_): [_PublicKey_](publickey.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_PublicKey_](publickey.md)

Defined in: [types/primitives.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L26)
