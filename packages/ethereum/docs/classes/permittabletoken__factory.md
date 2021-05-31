[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / PermittableToken__factory

# Class: PermittableToken\_\_factory

## Hierarchy

- *ContractFactory*

  ↳ **PermittableToken__factory**

## Table of contents

### Constructors

- [constructor](permittabletoken__factory.md#constructor)

### Properties

- [bytecode](permittabletoken__factory.md#bytecode)
- [interface](permittabletoken__factory.md#interface)
- [signer](permittabletoken__factory.md#signer)

### Methods

- [attach](permittabletoken__factory.md#attach)
- [connect](permittabletoken__factory.md#connect)
- [deploy](permittabletoken__factory.md#deploy)
- [getDeployTransaction](permittabletoken__factory.md#getdeploytransaction)
- [connect](permittabletoken__factory.md#connect)
- [fromSolidity](permittabletoken__factory.md#fromsolidity)
- [getContract](permittabletoken__factory.md#getcontract)
- [getContractAddress](permittabletoken__factory.md#getcontractaddress)
- [getInterface](permittabletoken__factory.md#getinterface)

## Constructors

### constructor

\+ **new PermittableToken__factory**(`signer?`: *Signer*): [*PermittableToken\_\_factory*](permittabletoken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | *Signer* |

**Returns:** [*PermittableToken\_\_factory*](permittabletoken__factory.md)

Overrides: ContractFactory.constructor

Defined in: packages/ethereum/types/factories/PermittableToken__factory.ts:16

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

▸ **attach**(`address`: *string*): [*PermittableToken*](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *string* |

**Returns:** [*PermittableToken*](permittabletoken.md)

Overrides: ContractFactory.attach

Defined in: packages/ethereum/types/factories/PermittableToken__factory.ts:51

___

### connect

▸ **connect**(`signer`: *Signer*): [*PermittableToken\_\_factory*](permittabletoken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | *Signer* |

**Returns:** [*PermittableToken\_\_factory*](permittabletoken__factory.md)

Overrides: ContractFactory.connect

Defined in: packages/ethereum/types/factories/PermittableToken__factory.ts:54

___

### deploy

▸ **deploy**(`_name`: *string*, `_symbol`: *string*, `_decimals`: BigNumberish, `_chainId`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<[*PermittableToken*](permittabletoken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_name` | *string* |
| `_symbol` | *string* |
| `_decimals` | BigNumberish |
| `_chainId` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<[*PermittableToken*](permittabletoken.md)\>

Overrides: ContractFactory.deploy

Defined in: packages/ethereum/types/factories/PermittableToken__factory.ts:21

___

### getDeployTransaction

▸ **getDeployTransaction**(`_name`: *string*, `_symbol`: *string*, `_decimals`: BigNumberish, `_chainId`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): TransactionRequest

#### Parameters

| Name | Type |
| :------ | :------ |
| `_name` | *string* |
| `_symbol` | *string* |
| `_decimals` | BigNumberish |
| `_chainId` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** TransactionRequest

Overrides: ContractFactory.getDeployTransaction

Defined in: packages/ethereum/types/factories/PermittableToken__factory.ts:36

___

### connect

▸ `Static` **connect**(`address`: *string*, `signerOrProvider`: *Signer* \| *Provider*): [*PermittableToken*](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *string* |
| `signerOrProvider` | *Signer* \| *Provider* |

**Returns:** [*PermittableToken*](permittabletoken.md)

Defined in: packages/ethereum/types/factories/PermittableToken__factory.ts:57

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
