[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / utils

# Module: utils

## Table of contents

### Functions

- [getNetworkGasPrice](utils.md#getnetworkgasprice)
- [getNetworkName](utils.md#getnetworkname)
- [getSignatureParameters](utils.md#getsignatureparameters)

## Functions

### getNetworkGasPrice

▸ **getNetworkGasPrice**(`network`: Networks): _number_ \| _undefined_

Get current network's name.

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `network` | Networks |

**Returns:** _number_ \| _undefined_

the network's name

Defined in: [packages/core-ethereum/src/utils.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/utils.ts#L23)

---

### getNetworkName

▸ **getNetworkName**(`chainId`: _number_): Networks

Get current network's name.

#### Parameters

| Name      | Type     | Description |
| :-------- | :------- | :---------- |
| `chainId` | _number_ | a chain id  |

**Returns:** Networks

the network's name

Defined in: [packages/core-ethereum/src/utils.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/utils.ts#L10)

---

### getSignatureParameters

▸ **getSignatureParameters**(`signature`: Signature): _object_

Get r,s,v values of a signature

#### Parameters

| Name        | Type      |
| :---------- | :-------- |
| `signature` | Signature |

**Returns:** _object_

| Name | Type     |
| :--- | :------- |
| `r`  | Hash     |
| `s`  | Hash     |
| `v`  | _number_ |

Defined in: [packages/core-ethereum/src/utils.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/utils.ts#L33)
