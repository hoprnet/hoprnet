[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/primitives](../modules/types_primitives.md) / Signature

# Class: Signature

[types/primitives](../modules/types_primitives.md).Signature

## Table of contents

### Constructors

- [constructor](types_primitives.signature.md#constructor)

### Properties

- [recovery](types_primitives.signature.md#recovery)
- [signature](types_primitives.signature.md#signature)
- [SIZE](types_primitives.signature.md#size)

### Methods

- [serialize](types_primitives.signature.md#serialize)
- [verify](types_primitives.signature.md#verify)
- [create](types_primitives.signature.md#create)
- [deserialize](types_primitives.signature.md#deserialize)

## Constructors

### constructor

\+ **new Signature**(`signature`: _Uint8Array_, `recovery`: _number_): [_Signature_](types_primitives.signature.md)

#### Parameters

| Name        | Type         |
| :---------- | :----------- |
| `signature` | _Uint8Array_ |
| `recovery`  | _number_     |

**Returns:** [_Signature_](types_primitives.signature.md)

Defined in: [types/primitives.ts:161](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L161)

## Properties

### recovery

• `Readonly` **recovery**: _number_

---

### signature

• `Readonly` **signature**: _Uint8Array_

---

### SIZE

▪ `Static` **SIZE**: _number_

Defined in: [types/primitives.ts:189](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L189)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/primitives.ts:178](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L178)

---

### verify

▸ **verify**(`msg`: _Uint8Array_, `pubKey`: [_PublicKey_](types_primitives.publickey.md)): _boolean_

#### Parameters

| Name     | Type                                         |
| :------- | :------------------------------------------- |
| `msg`    | _Uint8Array_                                 |
| `pubKey` | [_PublicKey_](types_primitives.publickey.md) |

**Returns:** _boolean_

Defined in: [types/primitives.ts:185](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L185)

---

### create

▸ `Static` **create**(`msg`: _Uint8Array_, `privKey`: _Uint8Array_): [_Signature_](types_primitives.signature.md)

#### Parameters

| Name      | Type         |
| :-------- | :----------- |
| `msg`     | _Uint8Array_ |
| `privKey` | _Uint8Array_ |

**Returns:** [_Signature_](types_primitives.signature.md)

Defined in: [types/primitives.ts:173](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L173)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_Signature_](types_primitives.signature.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_Signature_](types_primitives.signature.md)

Defined in: [types/primitives.ts:168](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L168)
