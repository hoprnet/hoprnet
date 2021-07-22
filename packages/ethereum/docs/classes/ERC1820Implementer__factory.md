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

### Methods

- [attach](ERC1820Implementer__factory.md#attach)
- [connect](ERC1820Implementer__factory.md#connect)
- [deploy](ERC1820Implementer__factory.md#deploy)
- [getDeployTransaction](ERC1820Implementer__factory.md#getdeploytransaction)
- [connect](ERC1820Implementer__factory.md#connect)
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

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:11

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

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:25

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

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:28

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

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:15

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

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:20

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

packages/ethereum/types/factories/ERC1820Implementer__factory.ts:31

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
