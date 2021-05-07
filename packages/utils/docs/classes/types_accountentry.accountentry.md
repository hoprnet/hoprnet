[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/accountEntry](../modules/types_accountentry.md) / AccountEntry

# Class: AccountEntry

[types/accountEntry](../modules/types_accountentry.md).AccountEntry

## Table of contents

### Constructors

- [constructor](types_accountentry.accountentry.md#constructor)

### Properties

- [address](types_accountentry.accountentry.md#address)
- [multiAddr](types_accountentry.accountentry.md#multiaddr)
- [updatedBlock](types_accountentry.accountentry.md#updatedblock)

### Accessors

- [SIZE](types_accountentry.accountentry.md#size)

### Methods

- [containsRouting](types_accountentry.accountentry.md#containsrouting)
- [getPeerId](types_accountentry.accountentry.md#getpeerid)
- [getPublicKey](types_accountentry.accountentry.md#getpublickey)
- [hasAnnounced](types_accountentry.accountentry.md#hasannounced)
- [serialize](types_accountentry.accountentry.md#serialize)
- [deserialize](types_accountentry.accountentry.md#deserialize)

## Constructors

### constructor

\+ **new AccountEntry**(`address`: [*Address*](types_primitives.address.md), `multiAddr`: *Multiaddr*, `updatedBlock`: *BN*): [*AccountEntry*](types_accountentry.accountentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [*Address*](types_primitives.address.md) |
| `multiAddr` | *Multiaddr* |
| `updatedBlock` | *BN* |

**Returns:** [*AccountEntry*](types_accountentry.accountentry.md)

Defined in: [types/accountEntry.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/accountEntry.ts#L8)

## Properties

### address

• `Readonly` **address**: [*Address*](types_primitives.address.md)

___

### multiAddr

• `Readonly` **multiAddr**: *Multiaddr*

___

### updatedBlock

• `Readonly` **updatedBlock**: *BN*

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/accountEntry.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/accountEntry.ts#L15)

## Methods

### containsRouting

▸ **containsRouting**(): *boolean*

**Returns:** *boolean*

Defined in: [types/accountEntry.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/accountEntry.ts#L51)

___

### getPeerId

▸ **getPeerId**(): *PeerId*

**Returns:** *PeerId*

Defined in: [types/accountEntry.ts:43](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/accountEntry.ts#L43)

___

### getPublicKey

▸ **getPublicKey**(): [*PublicKey*](types_primitives.publickey.md)

**Returns:** [*PublicKey*](types_primitives.publickey.md)

Defined in: [types/accountEntry.ts:47](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/accountEntry.ts#L47)

___

### hasAnnounced

▸ **hasAnnounced**(): *boolean*

**Returns:** *boolean*

Defined in: [types/accountEntry.ts:56](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/accountEntry.ts#L56)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/accountEntry.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/accountEntry.ts#L30)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*AccountEntry*](types_accountentry.accountentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*AccountEntry*](types_accountentry.accountentry.md)

Defined in: [types/accountEntry.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/accountEntry.ts#L19)
