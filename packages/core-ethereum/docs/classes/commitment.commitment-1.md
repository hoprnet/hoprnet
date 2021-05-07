[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [commitment](../modules/commitment.md) / Commitment

# Class: Commitment

[commitment](../modules/commitment.md).Commitment

## Table of contents

### Constructors

- [constructor](commitment.commitment-1.md#constructor)

### Properties

- [initialized](commitment.commitment-1.md#initialized)

### Methods

- [bumpCommitment](commitment.commitment-1.md#bumpcommitment)
- [createCommitmentChain](commitment.commitment-1.md#createcommitmentchain)
- [findPreImage](commitment.commitment-1.md#findpreimage)
- [getCurrentCommitment](commitment.commitment-1.md#getcurrentcommitment)
- [hasDBSecret](commitment.commitment-1.md#hasdbsecret)
- [initialize](commitment.commitment-1.md#initialize)
- [searchDBFor](commitment.commitment-1.md#searchdbfor)

## Constructors

### constructor

\+ **new Commitment**(`setChainCommitment`: _any_, `getChainCommitment`: _any_, `db`: _HoprDB_, `channelId`: _Hash_): [_Commitment_](commitment.commitment-1.md)

#### Parameters

| Name                 | Type     |
| :------------------- | :------- |
| `setChainCommitment` | _any_    |
| `getChainCommitment` | _any_    |
| `db`                 | _HoprDB_ |
| `channelId`          | _Hash_   |

**Returns:** [_Commitment_](commitment.commitment-1.md)

Defined in: [packages/core-ethereum/src/commitment.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L15)

## Properties

### initialized

• `Private` **initialized**: _boolean_= false

Defined in: [packages/core-ethereum/src/commitment.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L15)

## Methods

### bumpCommitment

▸ **bumpCommitment**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core-ethereum/src/commitment.ts:31](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L31)

---

### createCommitmentChain

▸ `Private` **createCommitmentChain**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core-ethereum/src/commitment.ts:75](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L75)

---

### findPreImage

▸ `Private` **findPreImage**(`hash`: _Hash_): _Promise_<Hash\>

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `hash` | _Hash_ |

**Returns:** _Promise_<Hash\>

Defined in: [packages/core-ethereum/src/commitment.ts:41](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L41)

---

### getCurrentCommitment

▸ **getCurrentCommitment**(): _Promise_<Hash\>

**Returns:** _Promise_<Hash\>

Defined in: [packages/core-ethereum/src/commitment.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L24)

---

### hasDBSecret

▸ `Private` **hasDBSecret**(): _Promise_<boolean\>

**Returns:** _Promise_<boolean\>

Defined in: [packages/core-ethereum/src/commitment.ts:85](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L85)

---

### initialize

▸ `Private` **initialize**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core-ethereum/src/commitment.ts:56](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L56)

---

### searchDBFor

▸ `Private` **searchDBFor**(`iteration`: _number_): _Promise_<Uint8Array\>

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `iteration` | _number_ |

**Returns:** _Promise_<Uint8Array\>

Defined in: [packages/core-ethereum/src/commitment.ts:89](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L89)
