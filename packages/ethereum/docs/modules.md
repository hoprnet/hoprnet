[@hoprnet/hopr-ethereum](README.md) / Exports

# @hoprnet/hopr-ethereum

## Table of contents

### Type aliases

- [ContractData](modules.md#contractdata)
- [ContractNames](modules.md#contractnames)
- [DeploymentTypes](modules.md#deploymenttypes)
- [NetworkTag](modules.md#networktag)
- [Networks](modules.md#networks)
- [PublicNetworks](modules.md#publicnetworks)

### Variables

- [abis](modules.md#abis)
- [networks](modules.md#networks)

### Functions

- [getContracts](modules.md#getcontracts)

## Type aliases

### ContractData

Ƭ **ContractData**: _object_

#### Type declaration

| Name          | Type     |
| :------------ | :------- |
| `address`     | _string_ |
| `deployedAt?` | _number_ |

Defined in: [chain/index.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/index.ts#L7)

---

### ContractNames

Ƭ **ContractNames**: `"HoprToken"` \| `"HoprChannels"` \| `"HoprDistributor"`

Defined in: [chain/index.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/index.ts#L6)

---

### DeploymentTypes

Ƭ **DeploymentTypes**: `"testing"` \| `"development"` \| `"staging"` \| `"production"`

Defined in: [chain/networks.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/networks.ts#L5)

---

### NetworkTag

Ƭ **NetworkTag**: [_DeploymentTypes_](modules.md#deploymenttypes) \| `"etherscan"`

Defined in: [chain/networks.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/networks.ts#L6)

---

### Networks

Ƭ **Networks**: `"hardhat"` \| `"localhost"` \| [_PublicNetworks_](modules.md#publicnetworks)

Defined in: [chain/networks.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/networks.ts#L4)

---

### PublicNetworks

Ƭ **PublicNetworks**: `"xdai"` \| `"goerli"`

Defined in: [chain/networks.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/networks.ts#L3)

## Variables

### abis

• `Const` **abis**: { [name in ContractNames]: any[]}

Defined in: [chain/index.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/index.ts#L20)

---

### networks

• `Const` **networks**: { [network in PublicNetworks]: object}

Defined in: [chain/networks.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/networks.ts#L8)

## Functions

### getContracts

▸ `Const` **getContracts**(): _object_

**Returns:** _object_

| Name        | Type |
| :---------- | :--- |
| `goerli`    |      |
| `hardhat`   |      |
| `localhost` |      |
| `xdai`      |      |

Defined in: [chain/index.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/chain/index.ts#L14)
