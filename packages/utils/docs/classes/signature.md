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

\+ **new Signature**(`signature`: *Uint8Array*, `recovery`: *number*): [*Signature*](signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signature` | *Uint8Array* |
| `recovery` | *number* |

**Returns:** [*Signature*](signature.md)

Defined in: [types/primitives.ts:165](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L165)

## Properties

### recovery

• `Readonly` **recovery**: *number*

___

### signature

• `Readonly` **signature**: *Uint8Array*

___

### SIZE

▪ `Static` **SIZE**: *number*

Defined in: [types/primitives.ts:193](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L193)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/primitives.ts:182](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L182)

___

### verify

▸ **verify**(`msg`: *Uint8Array*, `pubKey`: [*PublicKey*](publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `pubKey` | [*PublicKey*](publickey.md) |

**Returns:** *boolean*

Defined in: [types/primitives.ts:189](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L189)

___

### create

▸ `Static` **create**(`msg`: *Uint8Array*, `privKey`: *Uint8Array*): [*Signature*](signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `privKey` | *Uint8Array* |

**Returns:** [*Signature*](signature.md)

Defined in: [types/primitives.ts:177](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L177)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Signature*](signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Signature*](signature.md)

Defined in: [types/primitives.ts:172](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L172)
