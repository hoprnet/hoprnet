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

\+ **new AccountEntry**(`address`: [_Address_](address.md), `multiAddr`: _Multiaddr_, `updatedBlock`: _BN_): [_AccountEntry_](accountentry.md)

#### Parameters

| Name           | Type                    |
| :------------- | :---------------------- |
| `address`      | [_Address_](address.md) |
| `multiAddr`    | _Multiaddr_             |
| `updatedBlock` | _BN_                    |

**Returns:** [_AccountEntry_](accountentry.md)

Defined in: [types/accountEntry.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L8)

## Properties

### address

• `Readonly` **address**: [_Address_](address.md)

---

### multiAddr

• `Readonly` **multiAddr**: _Multiaddr_

---

### updatedBlock

• `Readonly` **updatedBlock**: _BN_

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/accountEntry.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L15)

## Methods

### containsRouting

▸ **containsRouting**(): _boolean_

**Returns:** _boolean_

Defined in: [types/accountEntry.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L51)

---

### getPeerId

▸ **getPeerId**(): _PeerId_

**Returns:** _PeerId_

Defined in: [types/accountEntry.ts:43](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L43)

---

### getPublicKey

▸ **getPublicKey**(): [_PublicKey_](publickey.md)

**Returns:** [_PublicKey_](publickey.md)

Defined in: [types/accountEntry.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L47)

---

### hasAnnounced

▸ **hasAnnounced**(): _boolean_

**Returns:** _boolean_

Defined in: [types/accountEntry.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L56)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/accountEntry.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L30)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_AccountEntry_](accountentry.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_AccountEntry_](accountentry.md)

Defined in: [types/accountEntry.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L19)
