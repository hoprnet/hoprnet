[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / ChannelCommitmentInfo

# Class: ChannelCommitmentInfo

Simple class encapsulating channel information
used to generate the initial channel commitment.

## Table of contents

### Constructors

- [constructor](ChannelCommitmentInfo.md#constructor)

### Properties

- [chainId](ChannelCommitmentInfo.md#chainid)
- [channelEpoch](ChannelCommitmentInfo.md#channelepoch)
- [channelId](ChannelCommitmentInfo.md#channelid)
- [contractAddress](ChannelCommitmentInfo.md#contractaddress)

### Methods

- [createInitialCommitmentSeed](ChannelCommitmentInfo.md#createinitialcommitmentseed)

## Constructors

### constructor

• **new ChannelCommitmentInfo**(`chainId`, `contractAddress`, `channelId`, `channelEpoch`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `chainId` | `number` |
| `contractAddress` | `string` |
| `channelId` | `Hash` |
| `channelEpoch` | `UINT256` |

#### Defined in

[packages/core-ethereum/src/commitment.ts:69](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L69)

## Properties

### chainId

• `Readonly` **chainId**: `number`

___

### channelEpoch

• `Readonly` **channelEpoch**: `UINT256`

___

### channelId

• `Readonly` **channelId**: `Hash`

___

### contractAddress

• `Readonly` **contractAddress**: `string`

## Methods

### createInitialCommitmentSeed

▸ **createInitialCommitmentSeed**(`peerId`): `Uint8Array`

Generate the initial commitment seed using this channel information and the given
private node key.
All members need to be specified (non-null).

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peerId` | `PeerId` | Local node ID. |

#### Returns

`Uint8Array`

#### Defined in

[packages/core-ethereum/src/commitment.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/commitment.ts#L82)
