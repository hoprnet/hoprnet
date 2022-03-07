[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Signature

# Class: Signature

Class used to represent an ECDSA signature.

The methods serialize()/deserialize() are used to convert the signature
to/from 64-byte compressed representation as given by EIP-2098 (https://eips.ethereum.org/EIPS/eip-2098).
This compressed signature format is supported by OpenZeppelin.

Internally this class still maintains representation using `(r,s)` tuple and `v` parity component separate
as this makes interop with the underlying ECDSA library simpler.

## Table of contents

### Constructors

- [constructor](Signature.md#constructor)

### Properties

- [recovery](Signature.md#recovery)
- [signature](Signature.md#signature)
- [SIZE](Signature.md#size)

### Methods

- [serialize](Signature.md#serialize)
- [toHex](Signature.md#tohex)
- [verify](Signature.md#verify)
- [create](Signature.md#create)
- [deserialize](Signature.md#deserialize)

## Constructors

### constructor

• **new Signature**(`signature`, `recovery`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signature` | `Uint8Array` |
| `recovery` | `number` |

#### Defined in

[types/primitives.ts:221](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L221)

## Properties

### recovery

• `Readonly` **recovery**: `number`

___

### signature

• `Readonly` **signature**: `Uint8Array`

___

### SIZE

▪ `Static` **SIZE**: `number` = `SIGNATURE_LENGTH`

#### Defined in

[types/primitives.ts:264](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L264)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:249](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L249)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:260](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L260)

___

### verify

▸ **verify**(`msg`, `pubKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | `Uint8Array` |
| `pubKey` | [`PublicKey`](PublicKey.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:256](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L256)

___

### create

▸ `Static` **create**(`msg`, `privKey`): [`Signature`](Signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | `Uint8Array` |
| `privKey` | `Uint8Array` |

#### Returns

[`Signature`](Signature.md)

#### Defined in

[types/primitives.ts:244](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L244)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Signature`](Signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Signature`](Signature.md)

#### Defined in

[types/primitives.ts:230](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L230)
