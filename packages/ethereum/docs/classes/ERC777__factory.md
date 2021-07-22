[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC777__factory

# Class: ERC777\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`ERC777__factory`**

## Table of contents

### Constructors

- [constructor](ERC777__factory.md#constructor)

### Properties

- [bytecode](ERC777__factory.md#bytecode)
- [interface](ERC777__factory.md#interface)
- [signer](ERC777__factory.md#signer)

### Methods

- [attach](ERC777__factory.md#attach)
- [connect](ERC777__factory.md#connect)
- [deploy](ERC777__factory.md#deploy)
- [getDeployTransaction](ERC777__factory.md#getdeploytransaction)
- [connect](ERC777__factory.md#connect)
- [fromSolidity](ERC777__factory.md#fromsolidity)
- [getContract](ERC777__factory.md#getcontract)
- [getContractAddress](ERC777__factory.md#getcontractaddress)
- [getInterface](ERC777__factory.md#getinterface)

## Constructors

### constructor

• **new ERC777__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/ERC777__factory.ts:11

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

▸ **attach**(`address`): [`ERC777`](ERC777.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`ERC777`](ERC777.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/ERC777__factory.ts:41

___

### connect

▸ **connect**(`signer`): [`ERC777__factory`](ERC777__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`ERC777__factory`](ERC777__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/ERC777__factory.ts:44

___

### deploy

▸ **deploy**(`name_`, `symbol_`, `defaultOperators_`, `overrides?`): `Promise`<[`ERC777`](ERC777.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `name_` | `string` |
| `symbol_` | `string` |
| `defaultOperators_` | `string`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`ERC777`](ERC777.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/ERC777__factory.ts:15

___

### getDeployTransaction

▸ **getDeployTransaction**(`name_`, `symbol_`, `defaultOperators_`, `overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `name_` | `string` |
| `symbol_` | `string` |
| `defaultOperators_` | `string`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/ERC777__factory.ts:28

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ERC777`](ERC777.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ERC777`](ERC777.md)

#### Defined in

packages/ethereum/types/factories/ERC777__factory.ts:47

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
