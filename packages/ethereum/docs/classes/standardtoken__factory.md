[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / StandardToken__factory

# Class: StandardToken\_\_factory

## Hierarchy

- *ContractFactory*

  ↳ **StandardToken__factory**

## Table of contents

### Constructors

- [constructor](standardtoken__factory.md#constructor)

### Properties

- [bytecode](standardtoken__factory.md#bytecode)
- [interface](standardtoken__factory.md#interface)
- [signer](standardtoken__factory.md#signer)

### Methods

- [attach](standardtoken__factory.md#attach)
- [connect](standardtoken__factory.md#connect)
- [deploy](standardtoken__factory.md#deploy)
- [getDeployTransaction](standardtoken__factory.md#getdeploytransaction)
- [connect](standardtoken__factory.md#connect)
- [fromSolidity](standardtoken__factory.md#fromsolidity)
- [getContract](standardtoken__factory.md#getcontract)
- [getContractAddress](standardtoken__factory.md#getcontractaddress)
- [getInterface](standardtoken__factory.md#getinterface)

## Constructors

### constructor

\+ **new StandardToken__factory**(`signer?`: *Signer*): [*StandardToken\_\_factory*](standardtoken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | *Signer* |

**Returns:** [*StandardToken\_\_factory*](standardtoken__factory.md)

Overrides: ContractFactory.constructor

Defined in: packages/ethereum/types/factories/StandardToken__factory.ts:10

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

▸ **attach**(`address`: *string*): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *string* |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: ContractFactory.attach

Defined in: packages/ethereum/types/factories/StandardToken__factory.ts:25

___

### connect

▸ **connect**(`signer`: *Signer*): [*StandardToken\_\_factory*](standardtoken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | *Signer* |

**Returns:** [*StandardToken\_\_factory*](standardtoken__factory.md)

Overrides: ContractFactory.connect

Defined in: packages/ethereum/types/factories/StandardToken__factory.ts:28

___

### deploy

▸ **deploy**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<[*StandardToken*](standardtoken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<[*StandardToken*](standardtoken.md)\>

Overrides: ContractFactory.deploy

Defined in: packages/ethereum/types/factories/StandardToken__factory.ts:15

___

### getDeployTransaction

▸ **getDeployTransaction**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): TransactionRequest

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** TransactionRequest

Overrides: ContractFactory.getDeployTransaction

Defined in: packages/ethereum/types/factories/StandardToken__factory.ts:20

___

### connect

▸ `Static` **connect**(`address`: *string*, `signerOrProvider`: *Signer* \| *Provider*): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *string* |
| `signerOrProvider` | *Signer* \| *Provider* |

**Returns:** [*StandardToken*](standardtoken.md)

Defined in: packages/ethereum/types/factories/StandardToken__factory.ts:31

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
