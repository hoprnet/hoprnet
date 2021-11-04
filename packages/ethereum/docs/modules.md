[@hoprnet/hopr-ethereum](README.md) / Exports

# @hoprnet/hopr-ethereum

## Table of contents

### Classes

- [AccessControl](classes/AccessControl.md)
- [AccessControl\_\_factory](classes/AccessControl__factory.md)
- [BasicToken](classes/BasicToken.md)
- [BasicToken\_\_factory](classes/BasicToken__factory.md)
- [BurnableToken](classes/BurnableToken.md)
- [BurnableToken\_\_factory](classes/BurnableToken__factory.md)
- [ChannelsMock](classes/ChannelsMock.md)
- [ChannelsMock\_\_factory](classes/ChannelsMock__factory.md)
- [DetailedERC20](classes/DetailedERC20.md)
- [DetailedERC20\_\_factory](classes/DetailedERC20__factory.md)
- [ERC1820Implementer](classes/ERC1820Implementer.md)
- [ERC1820Implementer\_\_factory](classes/ERC1820Implementer__factory.md)
- [ERC20](classes/ERC20.md)
- [ERC20Basic](classes/ERC20Basic.md)
- [ERC20Basic\_\_factory](classes/ERC20Basic__factory.md)
- [ERC20\_\_factory](classes/ERC20__factory.md)
- [ERC677](classes/ERC677.md)
- [ERC677BridgeToken](classes/ERC677BridgeToken.md)
- [ERC677BridgeToken\_\_factory](classes/ERC677BridgeToken__factory.md)
- [ERC677\_\_factory](classes/ERC677__factory.md)
- [ERC777](classes/ERC777.md)
- [ERC777Mock](classes/ERC777Mock.md)
- [ERC777Mock\_\_factory](classes/ERC777Mock__factory.md)
- [ERC777SenderRecipientMock](classes/ERC777SenderRecipientMock.md)
- [ERC777SenderRecipientMock\_\_factory](classes/ERC777SenderRecipientMock__factory.md)
- [ERC777Snapshot](classes/ERC777Snapshot.md)
- [ERC777SnapshotMock](classes/ERC777SnapshotMock.md)
- [ERC777SnapshotMock\_\_factory](classes/ERC777SnapshotMock__factory.md)
- [ERC777Snapshot\_\_factory](classes/ERC777Snapshot__factory.md)
- [ERC777\_\_factory](classes/ERC777__factory.md)
- [HoprChannels](classes/HoprChannels.md)
- [HoprChannels\_\_factory](classes/HoprChannels__factory.md)
- [HoprDistributor](classes/HoprDistributor.md)
- [HoprDistributor\_\_factory](classes/HoprDistributor__factory.md)
- [HoprForwarder](classes/HoprForwarder.md)
- [HoprForwarder\_\_factory](classes/HoprForwarder__factory.md)
- [HoprToken](classes/HoprToken.md)
- [HoprToken\_\_factory](classes/HoprToken__factory.md)
- [HoprWrapper](classes/HoprWrapper.md)
- [HoprWrapperProxy](classes/HoprWrapperProxy.md)
- [HoprWrapperProxy\_\_factory](classes/HoprWrapperProxy__factory.md)
- [HoprWrapper\_\_factory](classes/HoprWrapper__factory.md)
- [IBurnableMintableERC677Token](classes/IBurnableMintableERC677Token.md)
- [IBurnableMintableERC677Token\_\_factory](classes/IBurnableMintableERC677Token__factory.md)
- [IERC1820Implementer](classes/IERC1820Implementer.md)
- [IERC1820Implementer\_\_factory](classes/IERC1820Implementer__factory.md)
- [IERC1820Registry](classes/IERC1820Registry.md)
- [IERC1820Registry\_\_factory](classes/IERC1820Registry__factory.md)
- [IERC20](classes/IERC20.md)
- [IERC20\_\_factory](classes/IERC20__factory.md)
- [IERC677](classes/IERC677.md)
- [IERC677\_\_factory](classes/IERC677__factory.md)
- [IERC777](classes/IERC777.md)
- [IERC777Recipient](classes/IERC777Recipient.md)
- [IERC777Recipient\_\_factory](classes/IERC777Recipient__factory.md)
- [IERC777Sender](classes/IERC777Sender.md)
- [IERC777Sender\_\_factory](classes/IERC777Sender__factory.md)
- [IERC777\_\_factory](classes/IERC777__factory.md)
- [LegacyERC20](classes/LegacyERC20.md)
- [LegacyERC20\_\_factory](classes/LegacyERC20__factory.md)
- [MintableToken](classes/MintableToken.md)
- [MintableToken\_\_factory](classes/MintableToken__factory.md)
- [Multicall](classes/Multicall.md)
- [Multicall\_\_factory](classes/Multicall__factory.md)
- [Ownable](classes/Ownable.md)
- [Ownable\_\_factory](classes/Ownable__factory.md)
- [PermittableToken](classes/PermittableToken.md)
- [PermittableToken\_\_factory](classes/PermittableToken__factory.md)
- [ReentrancyGuard](classes/ReentrancyGuard.md)
- [ReentrancyGuard\_\_factory](classes/ReentrancyGuard__factory.md)
- [Sacrifice](classes/Sacrifice.md)
- [Sacrifice\_\_factory](classes/Sacrifice__factory.md)
- [StandardToken](classes/StandardToken.md)
- [StandardToken\_\_factory](classes/StandardToken__factory.md)

### Interfaces

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

[packages/ethereum/index.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/index.ts#L9)

___

### ContractNames

Ƭ **ContractNames**: ``"HoprToken"`` \| ``"HoprChannels"`` \| ``"HoprDistributor"``

#### Defined in

[packages/ethereum/index.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/index.ts#L7)

___

### DeploymentTypes

Ƭ **DeploymentTypes**: ``"testing"`` \| ``"development"`` \| ``"staging"`` \| ``"production"``

testing = for ganache / hardhat powered chains which do not auto mine
development = chains which automine - may or may not be public chains
staging = chain should be treated as production chain
production = our current production chain

#### Defined in

[packages/ethereum/constants.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L10)

___

### NetworkTag

Ƭ **NetworkTag**: [`DeploymentTypes`](modules.md#deploymenttypes) \| ``"etherscan"``

#### Defined in

[packages/ethereum/constants.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L11)

___

### Networks

Ƭ **Networks**: ``"hardhat"`` \| [`PublicNetworks`](modules.md#publicnetworks)

#### Defined in

[packages/ethereum/constants.ts:2](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L2)

___

### PublicNetworks

Ƭ **PublicNetworks**: ``"xdai"`` \| ``"goerli"``

#### Defined in

[packages/ethereum/constants.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L1)

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

[packages/ethereum/index.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/index.ts#L15)
