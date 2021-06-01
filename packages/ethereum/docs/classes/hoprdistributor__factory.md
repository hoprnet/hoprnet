[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprDistributor__factory

# Class: HoprDistributor\_\_factory

## Hierarchy

- *ContractFactory*

  ↳ **HoprDistributor__factory**

## Table of contents

### Constructors

- [constructor](hoprdistributor__factory.md#constructor)

### Properties

- [bytecode](hoprdistributor__factory.md#bytecode)
- [interface](hoprdistributor__factory.md#interface)
- [signer](hoprdistributor__factory.md#signer)

### Methods

- [attach](hoprdistributor__factory.md#attach)
- [connect](hoprdistributor__factory.md#connect)
- [deploy](hoprdistributor__factory.md#deploy)
- [getDeployTransaction](hoprdistributor__factory.md#getdeploytransaction)
- [connect](hoprdistributor__factory.md#connect)
- [fromSolidity](hoprdistributor__factory.md#fromsolidity)
- [getContract](hoprdistributor__factory.md#getcontract)
- [getContractAddress](hoprdistributor__factory.md#getcontractaddress)
- [getInterface](hoprdistributor__factory.md#getinterface)

## Constructors

### constructor

\+ **new HoprDistributor__factory**(`signer?`: *Signer*): [*HoprDistributor\_\_factory*](hoprdistributor__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | *Signer* |

**Returns:** [*HoprDistributor\_\_factory*](hoprdistributor__factory.md)

Overrides: ContractFactory.constructor

Defined in: packages/ethereum/types/factories/HoprDistributor__factory.ts:16

## Properties

### bytecode

• `Readonly` **bytecode**: *string*

Inherited from: ContractFactory.bytecode

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:131

___

### interface

• `Readonly` **interface**: *Interface*

Inherited from: ContractFactory.interface

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:130

___

### signer

• `Readonly` **signer**: *Signer*

Inherited from: ContractFactory.signer

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:132

## Methods

### attach

▸ **attach**(`address`: *string*): [*HoprDistributor*](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *string* |

**Returns:** [*HoprDistributor*](hoprdistributor.md)

Overrides: ContractFactory.attach

Defined in: packages/ethereum/types/factories/HoprDistributor__factory.ts:47

___

### connect

▸ **connect**(`signer`: *Signer*): [*HoprDistributor\_\_factory*](hoprdistributor__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | *Signer* |

**Returns:** [*HoprDistributor\_\_factory*](hoprdistributor__factory.md)

Overrides: ContractFactory.connect

Defined in: packages/ethereum/types/factories/HoprDistributor__factory.ts:50

___

### deploy

▸ **deploy**(`_token`: *string*, `_startTime`: BigNumberish, `_maxMintAmount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<[*HoprDistributor*](hoprdistributor.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_token` | *string* |
| `_startTime` | BigNumberish |
| `_maxMintAmount` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<[*HoprDistributor*](hoprdistributor.md)\>

Overrides: ContractFactory.deploy

Defined in: packages/ethereum/types/factories/HoprDistributor__factory.ts:21

___

### getDeployTransaction

▸ **getDeployTransaction**(`_token`: *string*, `_startTime`: BigNumberish, `_maxMintAmount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): TransactionRequest

#### Parameters

| Name | Type |
| :------ | :------ |
| `_token` | *string* |
| `_startTime` | BigNumberish |
| `_maxMintAmount` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** TransactionRequest

Overrides: ContractFactory.getDeployTransaction

Defined in: packages/ethereum/types/factories/HoprDistributor__factory.ts:34

___

### connect

▸ `Static` **connect**(`address`: *string*, `signerOrProvider`: *Signer* \| *Provider*): [*HoprDistributor*](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *string* |
| `signerOrProvider` | *Signer* \| *Provider* |

**Returns:** [*HoprDistributor*](hoprdistributor.md)

Defined in: packages/ethereum/types/factories/HoprDistributor__factory.ts:53

___

### fromSolidity

▸ `Static` **fromSolidity**(`compilerOutput`: *any*, `signer?`: *Signer*): *ContractFactory*

#### Parameters

| Name | Type |
| :------ | :------ |
| `compilerOutput` | *any* |
| `signer?` | *Signer* |

**Returns:** *ContractFactory*

Inherited from: ContractFactory.fromSolidity

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:140

___

### getContract

▸ `Static` **getContract**(`address`: *string*, `contractInterface`: ContractInterface, `signer?`: *Signer*): *Contract*

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *string* |
| `contractInterface` | ContractInterface |
| `signer?` | *Signer* |

**Returns:** *Contract*

Inherited from: ContractFactory.getContract

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:146

___

### getContractAddress

▸ `Static` **getContractAddress**(`tx`: { `from`: *string* ; `nonce`: *number* \| *BigNumber* \| BytesLike  }): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `tx` | *object* |
| `tx.from` | *string* |
| `tx.nonce` | *number* \| *BigNumber* \| BytesLike |

**Returns:** *string*

Inherited from: ContractFactory.getContractAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:142

___

### getInterface

▸ `Static` **getInterface**(`contractInterface`: ContractInterface): *Interface*

#### Parameters

| Name | Type |
| :------ | :------ |
| `contractInterface` | ContractInterface |

**Returns:** *Interface*

Inherited from: ContractFactory.getInterface

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:141
