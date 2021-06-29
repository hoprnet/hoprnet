[@hoprnet/hopr-ethereum](README.md) / Exports

# @hoprnet/hopr-ethereum

## Table of contents

### Classes

- [AccessControl](classes/accesscontrol.md)
- [AccessControl\_\_factory](classes/accesscontrol__factory.md)
- [BasicToken](classes/basictoken.md)
- [BasicToken\_\_factory](classes/basictoken__factory.md)
- [BurnableToken](classes/burnabletoken.md)
- [BurnableToken\_\_factory](classes/burnabletoken__factory.md)
- [ChannelsMock](classes/channelsmock.md)
- [ChannelsMock\_\_factory](classes/channelsmock__factory.md)
- [Context](classes/context.md)
- [Context\_\_factory](classes/context__factory.md)
- [DetailedERC20](classes/detailederc20.md)
- [DetailedERC20\_\_factory](classes/detailederc20__factory.md)
- [ERC1820Implementer](classes/erc1820implementer.md)
- [ERC1820Implementer\_\_factory](classes/erc1820implementer__factory.md)
- [ERC20](classes/erc20.md)
- [ERC20Basic](classes/erc20basic.md)
- [ERC20Basic\_\_factory](classes/erc20basic__factory.md)
- [ERC20\_\_factory](classes/erc20__factory.md)
- [ERC677](classes/erc677.md)
- [ERC677BridgeToken](classes/erc677bridgetoken.md)
- [ERC677BridgeToken\_\_factory](classes/erc677bridgetoken__factory.md)
- [ERC677\_\_factory](classes/erc677__factory.md)
- [ERC777](classes/erc777.md)
- [ERC777Mock](classes/erc777mock.md)
- [ERC777Mock\_\_factory](classes/erc777mock__factory.md)
- [ERC777SenderRecipientMock](classes/erc777senderrecipientmock.md)
- [ERC777SenderRecipientMock\_\_factory](classes/erc777senderrecipientmock__factory.md)
- [ERC777Snapshot](classes/erc777snapshot.md)
- [ERC777SnapshotMock](classes/erc777snapshotmock.md)
- [ERC777SnapshotMock\_\_factory](classes/erc777snapshotmock__factory.md)
- [ERC777Snapshot\_\_factory](classes/erc777snapshot__factory.md)
- [ERC777\_\_factory](classes/erc777__factory.md)
- [HoprChannels](classes/hoprchannels.md)
- [HoprChannels\_\_factory](classes/hoprchannels__factory.md)
- [HoprDistributor](classes/hoprdistributor.md)
- [HoprDistributor\_\_factory](classes/hoprdistributor__factory.md)
- [HoprForwarder](classes/hoprforwarder.md)
- [HoprForwarder\_\_factory](classes/hoprforwarder__factory.md)
- [HoprToken](classes/hoprtoken.md)
- [HoprToken\_\_factory](classes/hoprtoken__factory.md)
- [HoprWrapper](classes/hoprwrapper.md)
- [HoprWrapper\_\_factory](classes/hoprwrapper__factory.md)
- [IBurnableMintableERC677Token](classes/iburnablemintableerc677token.md)
- [IBurnableMintableERC677Token\_\_factory](classes/iburnablemintableerc677token__factory.md)
- [IERC1820Implementer](classes/ierc1820implementer.md)
- [IERC1820Implementer\_\_factory](classes/ierc1820implementer__factory.md)
- [IERC1820Registry](classes/ierc1820registry.md)
- [IERC1820Registry\_\_factory](classes/ierc1820registry__factory.md)
- [IERC20](classes/ierc20.md)
- [IERC20\_\_factory](classes/ierc20__factory.md)
- [IERC777](classes/ierc777.md)
- [IERC777Recipient](classes/ierc777recipient.md)
- [IERC777Recipient\_\_factory](classes/ierc777recipient__factory.md)
- [IERC777Sender](classes/ierc777sender.md)
- [IERC777Sender\_\_factory](classes/ierc777sender__factory.md)
- [IERC777\_\_factory](classes/ierc777__factory.md)
- [LegacyERC20](classes/legacyerc20.md)
- [LegacyERC20\_\_factory](classes/legacyerc20__factory.md)
- [MintableToken](classes/mintabletoken.md)
- [MintableToken\_\_factory](classes/mintabletoken__factory.md)
- [Ownable](classes/ownable.md)
- [Ownable\_\_factory](classes/ownable__factory.md)
- [PermittableToken](classes/permittabletoken.md)
- [PermittableToken\_\_factory](classes/permittabletoken__factory.md)
- [ReentrancyGuard](classes/reentrancyguard.md)
- [ReentrancyGuard\_\_factory](classes/reentrancyguard__factory.md)
- [Sacrifice](classes/sacrifice.md)
- [Sacrifice\_\_factory](classes/sacrifice__factory.md)
- [StandardToken](classes/standardtoken.md)
- [StandardToken\_\_factory](classes/standardtoken__factory.md)

### Interfaces

- [TypedEvent](interfaces/typedevent.md)
- [TypedEventFilter](interfaces/typedeventfilter.md)

### Type aliases

- [ContractData](modules.md#contractdata)
- [ContractNames](modules.md#contractnames)
- [DeploymentTypes](modules.md#deploymenttypes)
- [NetworkTag](modules.md#networktag)
- [Networks](modules.md#networks)
- [PublicNetworks](modules.md#publicnetworks)
- [TypedListener](modules.md#typedlistener)

### Variables

- [networks](modules.md#networks)

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

[packages/ethereum/index.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/index.ts#L10)

___

### ContractNames

Ƭ **ContractNames**: ``"HoprToken"`` \| ``"HoprChannels"`` \| ``"HoprDistributor"``

#### Defined in

[packages/ethereum/index.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/index.ts#L8)

___

### DeploymentTypes

Ƭ **DeploymentTypes**: ``"testing"`` \| ``"development"`` \| ``"staging"`` \| ``"production"``

#### Defined in

[packages/ethereum/constants.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L5)

___

### NetworkTag

Ƭ **NetworkTag**: [`DeploymentTypes`](modules.md#deploymenttypes) \| ``"etherscan"``

#### Defined in

[packages/ethereum/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L6)

___

### Networks

Ƭ **Networks**: ``"hardhat"`` \| ``"localhost"`` \| [`PublicNetworks`](modules.md#publicnetworks)

#### Defined in

[packages/ethereum/constants.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L4)

___

### PublicNetworks

Ƭ **PublicNetworks**: ``"xdai"`` \| ``"goerli"``

#### Defined in

[packages/ethereum/constants.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L3)

___

### TypedListener

Ƭ **TypedListener**<`EventArgsArray`, `EventArgsObject`\>: (...`listenerArg`: [...EventArgsArray, [`TypedEvent`](interfaces/typedevent.md)<`EventArgsArray` & `EventArgsObject`\>]) => `void`

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Type declaration

▸ (...`listenerArg`): `void`

##### Parameters

| Name | Type |
| :------ | :------ |
| `...listenerArg` | [...EventArgsArray, [`TypedEvent`](interfaces/typedevent.md)<`EventArgsArray` & `EventArgsObject`\>] |

##### Returns

`void`

#### Defined in

packages/ethereum/types/commons.ts:15

## Variables

### networks

• `Const` **networks**: { [network in PublicNetworks]: object}

#### Defined in

[packages/ethereum/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/constants.ts#L8)

## Functions

### getContractData

▸ `Const` **getContractData**(`network`, `contract`): [`ContractData`](modules.md#contractdata)

#### Parameters

| Name | Type |
| :------ | :------ |
| `network` | [`Networks`](modules.md#networks) |
| `contract` | [`ContractNames`](modules.md#contractnames) |

#### Returns

[`ContractData`](modules.md#contractdata)

#### Defined in

[packages/ethereum/index.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/ethereum/index.ts#L16)
