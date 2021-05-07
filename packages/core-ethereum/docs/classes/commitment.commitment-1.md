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

\+ **new Commitment**(`setChainCommitment`: *any*, `getChainCommitment`: *any*, `db`: *HoprDB*, `channelId`: *Hash*): [*Commitment*](commitment.commitment-1.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `setChainCommitment` | *any* |
| `getChainCommitment` | *any* |
| `db` | *HoprDB* |
| `channelId` | *Hash* |

**Returns:** [*Commitment*](commitment.commitment-1.md)

Defined in: [packages/core-ethereum/src/commitment.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L15)

## Properties

### initialized

• `Private` **initialized**: *boolean*= false

Defined in: [packages/core-ethereum/src/commitment.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L15)

## Methods

### bumpCommitment

▸ **bumpCommitment**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/commitment.ts:31](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L31)

___

### createCommitmentChain

▸ `Private` **createCommitmentChain**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/commitment.ts:75](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L75)

___

### findPreImage

▸ `Private` **findPreImage**(`hash`: *Hash*): *Promise*<Hash\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `hash` | *Hash* |

**Returns:** *Promise*<Hash\>

Defined in: [packages/core-ethereum/src/commitment.ts:41](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L41)

___

### getCurrentCommitment

▸ **getCurrentCommitment**(): *Promise*<Hash\>

**Returns:** *Promise*<Hash\>

Defined in: [packages/core-ethereum/src/commitment.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L24)

___

### hasDBSecret

▸ `Private` **hasDBSecret**(): *Promise*<boolean\>

**Returns:** *Promise*<boolean\>

Defined in: [packages/core-ethereum/src/commitment.ts:85](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L85)

___

### initialize

▸ `Private` **initialize**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/commitment.ts:56](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L56)

___

### searchDBFor

▸ `Private` **searchDBFor**(`iteration`: *number*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `iteration` | *number* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [packages/core-ethereum/src/commitment.ts:89](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/commitment.ts#L89)
