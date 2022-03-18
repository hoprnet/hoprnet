[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprChannels

# Namespace: HoprChannels

## Table of contents

### Type aliases

- [ChannelStruct](HoprChannels.md#channelstruct)
- [ChannelStructOutput](HoprChannels.md#channelstructoutput)

## Type aliases

### ChannelStruct

Ƭ **ChannelStruct**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `balance` | `BigNumberish` |
| `channelEpoch` | `BigNumberish` |
| `closureTime` | `BigNumberish` |
| `commitment` | `BytesLike` |
| `status` | `BigNumberish` |
| `ticketEpoch` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |

#### Defined in

packages/ethereum/src/types/HoprChannels.ts:21

___

### ChannelStructOutput

Ƭ **ChannelStructOutput**: [`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }

#### Defined in

packages/ethereum/src/types/HoprChannels.ts:31
