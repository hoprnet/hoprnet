[@hoprnet/hopr-ethereum](README.md) / Exports

# @hoprnet/hopr-ethereum

## Table of contents

### Interfaces

- [HoprChannels](interfaces/HoprChannels.md)
- [HoprToken](interfaces/HoprToken.md)
- [TypedEvent](interfaces/TypedEvent.md)
- [TypedEventFilter](interfaces/TypedEventFilter.md)

### Type aliases

- [ContractData](modules.md#contractdata)
- [ContractNames](modules.md#contractnames)
- [DeploymentTypes](modules.md#deploymenttypes)
- [NetworkTag](modules.md#networktag)
- [Networks](modules.md#networks)
- [PublicNetworks](modules.md#publicnetworks)

### Functions

- [getContractData](modules.md#getcontractdata)

## Type aliases

### ContractData

Ƭ **ContractData**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `abi` | `any` |
| `address` | `string` |
| `transactionHash` | `string` |

#### Defined in

[packages/ethereum/src/index.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/index.ts#L9)

___

### ContractNames

Ƭ **ContractNames**: ``"HoprToken"`` \| ``"HoprChannels"`` \| ``"HoprDistributor"``

#### Defined in

[packages/ethereum/src/index.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/index.ts#L7)

___

### DeploymentTypes

Ƭ **DeploymentTypes**: ``"testing"`` \| ``"development"`` \| ``"staging"`` \| ``"production"``

testing = for ganache / hardhat powered chains which do not auto mine
development = chains which automine - may or may not be public chains
staging = chain should be treated as production chain
production = our current production chain

#### Defined in

[packages/ethereum/src/constants.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/constants.ts#L10)

___

### NetworkTag

Ƭ **NetworkTag**: [`DeploymentTypes`](modules.md#deploymenttypes) \| ``"etherscan"``

#### Defined in

[packages/ethereum/src/constants.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/constants.ts#L11)

___

### Networks

Ƭ **Networks**: ``"hardhat"`` \| [`PublicNetworks`](modules.md#publicnetworks)

#### Defined in

[packages/ethereum/src/constants.ts:2](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/constants.ts#L2)

___

### PublicNetworks

Ƭ **PublicNetworks**: ``"xdai"`` \| ``"goerli"``

#### Defined in

[packages/ethereum/src/constants.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/constants.ts#L1)

## Functions

### getContractData

▸ `Const` **getContractData**(`network`, `environmentId`, `contract`): [`ContractData`](modules.md#contractdata)

#### Parameters

| Name | Type |
| :------ | :------ |
| `network` | `string` |
| `environmentId` | `string` |
| `contract` | [`ContractNames`](modules.md#contractnames) |

#### Returns

[`ContractData`](modules.md#contractdata)

#### Defined in

[packages/ethereum/src/index.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/index.ts#L15)
