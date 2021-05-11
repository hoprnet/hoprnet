[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / AccountEntry

# Class: AccountEntry

## Table of contents

### Constructors

- [constructor](accountentry.md#constructor)

### Properties

- [address](accountentry.md#address)
- [multiAddr](accountentry.md#multiaddr)
- [updatedBlock](accountentry.md#updatedblock)

### Accessors

- [SIZE](accountentry.md#size)

### Methods

- [containsRouting](accountentry.md#containsrouting)
- [getPeerId](accountentry.md#getpeerid)
- [getPublicKey](accountentry.md#getpublickey)
- [hasAnnounced](accountentry.md#hasannounced)
- [serialize](accountentry.md#serialize)
- [deserialize](accountentry.md#deserialize)

## Constructors

### constructor

\+ **new AccountEntry**(`address`: [*Address*](address.md), `multiAddr`: *Multiaddr*, `updatedBlock`: *BN*): [*AccountEntry*](accountentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [*Address*](address.md) |
| `multiAddr` | *Multiaddr* |
| `updatedBlock` | *BN* |

**Returns:** [*AccountEntry*](accountentry.md)

Defined in: [types/accountEntry.ts:8](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L8)

## Properties

### address

• `Readonly` **address**: [*Address*](address.md)

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

Defined in: [types/accountEntry.ts:15](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L15)

## Methods

### containsRouting

▸ **containsRouting**(): *boolean*

**Returns:** *boolean*

Defined in: [types/accountEntry.ts:51](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L51)

___

### getPeerId

▸ **getPeerId**(): *PeerId*

**Returns:** *PeerId*

Defined in: [types/accountEntry.ts:43](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L43)

___

### getPublicKey

▸ **getPublicKey**(): [*PublicKey*](publickey.md)

**Returns:** [*PublicKey*](publickey.md)

Defined in: [types/accountEntry.ts:47](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L47)

___

### hasAnnounced

▸ **hasAnnounced**(): *boolean*

**Returns:** *boolean*

Defined in: [types/accountEntry.ts:56](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L56)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/accountEntry.ts:30](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L30)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*AccountEntry*](accountentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*AccountEntry*](accountentry.md)

Defined in: [types/accountEntry.ts:19](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L19)
