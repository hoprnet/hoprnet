[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / utils

# Module: utils

## Table of contents

### Functions

- [getNetworkGasPrice](utils.md#getnetworkgasprice)
- [getNetworkName](utils.md#getnetworkname)
- [getSignatureParameters](utils.md#getsignatureparameters)

## Functions

### getNetworkGasPrice

▸ **getNetworkGasPrice**(`network`: Networks): *number* \| *undefined*

Get current network's name.

#### Parameters

| Name | Type |
| :------ | :------ |
| `network` | Networks |

**Returns:** *number* \| *undefined*

the network's name

Defined in: [packages/core-ethereum/src/utils.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/utils.ts#L23)

___

### getNetworkName

▸ **getNetworkName**(`chainId`: *number*): Networks

Get current network's name.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `chainId` | *number* | a chain id |

**Returns:** Networks

the network's name

Defined in: [packages/core-ethereum/src/utils.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/utils.ts#L10)

___

### getSignatureParameters

▸ **getSignatureParameters**(`signature`: Signature): *object*

Get r,s,v values of a signature

#### Parameters

| Name | Type |
| :------ | :------ |
| `signature` | Signature |

**Returns:** *object*

| Name | Type |
| :------ | :------ |
| `r` | Hash |
| `s` | Hash |
| `v` | *number* |

Defined in: [packages/core-ethereum/src/utils.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/utils.ts#L33)
