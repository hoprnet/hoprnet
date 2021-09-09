[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC1820Implementer__factory

# Class: ERC1820Implementer\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`ERC1820Implementer__factory`**

## Table of contents

### Constructors

- [constructor](ERC1820Implementer__factory.md#constructor)

### Properties

- [bytecode](ERC1820Implementer__factory.md#bytecode)
- [interface](ERC1820Implementer__factory.md#interface)
- [signer](ERC1820Implementer__factory.md#signer)
- [abi](ERC1820Implementer__factory.md#abi)
- [bytecode](ERC1820Implementer__factory.md#bytecode)

### Methods

- [attach](ERC1820Implementer__factory.md#attach)
- [connect](ERC1820Implementer__factory.md#connect)
- [deploy](ERC1820Implementer__factory.md#deploy)
- [getDeployTransaction](ERC1820Implementer__factory.md#getdeploytransaction)
- [connect](ERC1820Implementer__factory.md#connect)
- [createInterface](ERC1820Implementer__factory.md#createinterface)
- [fromSolidity](ERC1820Implementer__factory.md#fromsolidity)
- [getContract](ERC1820Implementer__factory.md#getcontract)
- [getContractAddress](ERC1820Implementer__factory.md#getcontractaddress)
- [getInterface](ERC1820Implementer__factory.md#getinterface)

## Constructors

### constructor

• **new ERC1820Implementer__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:43

## Properties

### bytecode

• `Readonly` **bytecode**: `string`

#### Inherited from

ContractFactory.bytecode

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:135

___

### interface

• `Readonly` **interface**: `Interface`

#### Inherited from

ContractFactory.interface

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:134

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

ContractFactory.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:136

___

### abi

▪ `Static` `Readonly` **abi**: { `inputs`: { `internalType`: `string` = "bytes32"; `name`: `string` = "interfaceHash"; `type`: `string` = "bytes32" }[] ; `name`: `string` = "canImplementInterfaceForAddress"; `outputs`: { `internalType`: `string` = "bytes32"; `name`: `string` = ""; `type`: `string` = "bytes32" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" }[]

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:64

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405234801561001057600080fd5b50610112806100206000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063249cb3fa14602d575b600080fd5b603c603836600460a4565b604e565b60405190815260200160405180910390f35b6000828152602081815260408083206001600160a01b038516845290915281205460ff16607b576000609d565b7fa2ef4600d742022d532d4747cb3547474667d6f13804902513b2ec01c848f4b45b9392505050565b6000806040838503121560b5578182fd5b8235915060208301356001600160a01b038116811460d1578182fd5b80915050925092905056fea264697066735822122083955dced04baee5a5fe89f6ff23af051fe56c66fa086e20144b2a9a4782137d64736f6c63430008030033"``

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:63

## Methods

### attach

▸ **attach**(`address`): [`ERC1820Implementer`](ERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`ERC1820Implementer`](ERC1820Implementer.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:57

___

### connect

▸ **connect**(`signer`): [`ERC1820Implementer__factory`](ERC1820Implementer__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`ERC1820Implementer__factory`](ERC1820Implementer__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:60

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`ERC1820Implementer`](ERC1820Implementer.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`ERC1820Implementer`](ERC1820Implementer.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:47

___

### getDeployTransaction

▸ **getDeployTransaction**(`overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:52

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ERC1820Implementer`](ERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ERC1820Implementer`](ERC1820Implementer.md)

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:68

___

### createInterface

▸ `Static` **createInterface**(): `ERC1820ImplementerInterface`

#### Returns

`ERC1820ImplementerInterface`

#### Defined in

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:65

___

### fromSolidity

▸ `Static` **fromSolidity**(`compilerOutput`, `signer?`): `ContractFactory`

#### Parameters

| Name | Type |
| :------ | :------ |
| `compilerOutput` | `any` |
| `signer?` | `Signer` |

#### Returns

`ContractFactory`

#### Inherited from

ContractFactory.fromSolidity

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:144

___

### getContract

▸ `Static` **getContract**(`address`, `contractInterface`, `signer?`): `Contract`

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `contractInterface` | `ContractInterface` |
| `signer?` | `Signer` |

#### Returns

`Contract`

#### Inherited from

ContractFactory.getContract

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:150

___

### getContractAddress

▸ `Static` **getContractAddress**(`tx`): `string`

#### Parameters

| Name | Type |
| :------ | :------ |
| `tx` | `Object` |
| `tx.from` | `string` |
| `tx.nonce` | `number` \| `BigNumber` \| `BytesLike` |

#### Returns

`string`

#### Inherited from

ContractFactory.getContractAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:146

___

### getInterface

▸ `Static` **getInterface**(`contractInterface`): `Interface`

#### Parameters

| Name | Type |
| :------ | :------ |
| `contractInterface` | `ContractInterface` |

#### Returns

`Interface`

#### Inherited from

ContractFactory.getInterface

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:145
