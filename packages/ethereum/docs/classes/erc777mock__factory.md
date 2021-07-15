[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC777Mock__factory

# Class: ERC777Mock\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`ERC777Mock__factory`**

## Table of contents

### Constructors

- [constructor](erc777mock__factory.md#constructor)

### Properties

- [bytecode](erc777mock__factory.md#bytecode)
- [interface](erc777mock__factory.md#interface)
- [signer](erc777mock__factory.md#signer)

### Methods

- [attach](erc777mock__factory.md#attach)
- [connect](erc777mock__factory.md#connect)
- [deploy](erc777mock__factory.md#deploy)
- [getDeployTransaction](erc777mock__factory.md#getdeploytransaction)
- [connect](erc777mock__factory.md#connect)
- [fromSolidity](erc777mock__factory.md#fromsolidity)
- [getContract](erc777mock__factory.md#getcontract)
- [getContractAddress](erc777mock__factory.md#getcontractaddress)
- [getInterface](erc777mock__factory.md#getinterface)

## Constructors

### constructor

• **new ERC777Mock__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/ERC777Mock__factory.ts:16

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

▸ **attach**(`address`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/ERC777Mock__factory.ts:55

___

### connect

▸ **connect**(`signer`): [`ERC777Mock__factory`](erc777mock__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`ERC777Mock__factory`](erc777mock__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/ERC777Mock__factory.ts:58

___

### deploy

▸ **deploy**(`initialHolder`, `initialBalance`, `name`, `symbol`, `defaultOperators`, `overrides?`): `Promise`<[`ERC777Mock`](erc777mock.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `initialHolder` | `string` |
| `initialBalance` | `BigNumberish` |
| `name` | `string` |
| `symbol` | `string` |
| `defaultOperators` | `string`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`ERC777Mock`](erc777mock.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/ERC777Mock__factory.ts:21

___

### getDeployTransaction

▸ **getDeployTransaction**(`initialHolder`, `initialBalance`, `name`, `symbol`, `defaultOperators`, `overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `initialHolder` | `string` |
| `initialBalance` | `BigNumberish` |
| `name` | `string` |
| `symbol` | `string` |
| `defaultOperators` | `string`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/ERC777Mock__factory.ts:38

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Defined in

packages/ethereum/types/factories/ERC777Mock__factory.ts:61

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
