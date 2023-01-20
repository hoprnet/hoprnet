[@hoprnet/hopr-ethereum](README.md) / Exports

# @hoprnet/hopr-ethereum

## Table of contents

### Namespaces

- [HoprChannels](modules/HoprChannels.md)

### Interfaces

- [HoprBoost](interfaces/HoprBoost.md)
- [HoprChannels](interfaces/HoprChannels-1.md)
- [HoprDistributor](interfaces/HoprDistributor.md)
- [HoprNetworkRegistry](interfaces/HoprNetworkRegistry.md)
- [HoprStake](interfaces/HoprStake.md)
- [HoprStake2](interfaces/HoprStake2.md)
- [HoprStakeSeason3](interfaces/HoprStakeSeason3.md)
- [HoprStakeSeason4](interfaces/HoprStakeSeason4.md)
- [HoprStakeSeason5](interfaces/HoprStakeSeason5.md)
- [HoprToken](interfaces/HoprToken.md)
- [HoprWhitehat](interfaces/HoprWhitehat.md)
- [TypedEvent](interfaces/TypedEvent.md)
- [TypedEventFilter](interfaces/TypedEventFilter.md)
- [xHoprToken](interfaces/xHoprToken.md)

### Type Aliases

- [ContractData](modules.md#contractdata)
- [ContractNames](modules.md#contractnames)
- [DeploymentTypes](modules.md#deploymenttypes)
- [NetworkTag](modules.md#networktag)
- [Networks](modules.md#networks)
- [PublicNetworks](modules.md#publicnetworks)

### Functions

- [getContractData](modules.md#getcontractdata)

## Type Aliases

### ContractData

Ƭ **ContractData**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `abi` | `any` |
| `address` | `string` |
| `blockNumber` | `number` |
| `transactionHash` | `string` |

#### Defined in

[packages/ethereum/src/index.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/index.ts#L34)

___

### ContractNames

Ƭ **ContractNames**: ``"HoprToken"`` \| ``"HoprChannels"`` \| ``"HoprDistributor"`` \| ``"HoprNetworkRegistry"`` \| ``"HoprBoost"`` \| ``"HoprStake"`` \| ``"HoprStake2"`` \| ``"HoprStakeSeason3"`` \| ``"HoprStakeSeason4"`` \| ``"HoprWhitehat"``

#### Defined in

[packages/ethereum/src/index.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/index.ts#L22)

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

▸ **getContractData**(`network`, `environmentId`, `contract`): [`ContractData`](modules.md#contractdata) \| `Deployment`

#### Parameters

| Name | Type |
| :------ | :------ |
| `network` | `string` |
| `environmentId` | `string` |
| `contract` | [`ContractNames`](modules.md#contractnames) |

#### Returns

[`ContractData`](modules.md#contractdata) \| `Deployment`

#### Defined in

[packages/ethereum/src/index.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/src/index.ts#L41)
