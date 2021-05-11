[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Signature

# Class: Signature

## Table of contents

### Constructors

- [constructor](signature.md#constructor)

### Properties

- [recovery](signature.md#recovery)
- [signature](signature.md#signature)
- [SIZE](signature.md#size)

### Methods

- [serialize](signature.md#serialize)
- [verify](signature.md#verify)
- [create](signature.md#create)
- [deserialize](signature.md#deserialize)

## Constructors

### constructor

\+ **new Signature**(`signature`: _Uint8Array_, `recovery`: _number_): [_Signature_](signature.md)

#### Parameters

| Name        | Type         |
| :---------- | :----------- |
| `signature` | _Uint8Array_ |
| `recovery`  | _number_     |

**Returns:** [_Signature_](signature.md)

Defined in: [types/primitives.ts:161](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L161)

## Properties

### recovery

• `Readonly` **recovery**: _number_

---

### signature

• `Readonly` **signature**: _Uint8Array_

---

### SIZE

▪ `Static` **SIZE**: _number_

Defined in: [types/primitives.ts:189](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L189)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/primitives.ts:178](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L178)

---

### verify

▸ **verify**(`msg`: _Uint8Array_, `pubKey`: [_PublicKey_](publickey.md)): _boolean_

#### Parameters

| Name     | Type                        |
| :------- | :-------------------------- |
| `msg`    | _Uint8Array_                |
| `pubKey` | [_PublicKey_](publickey.md) |

**Returns:** _boolean_

Defined in: [types/primitives.ts:185](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L185)

---

### create

▸ `Static` **create**(`msg`: _Uint8Array_, `privKey`: _Uint8Array_): [_Signature_](signature.md)

#### Parameters

| Name      | Type         |
| :-------- | :----------- |
| `msg`     | _Uint8Array_ |
| `privKey` | _Uint8Array_ |

**Returns:** [_Signature_](signature.md)

Defined in: [types/primitives.ts:173](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L173)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_Signature_](signature.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_Signature_](signature.md)

Defined in: [types/primitives.ts:168](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L168)
