[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / BasicToken__factory

# Class: BasicToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`BasicToken__factory`**

## Table of contents

### Constructors

- [constructor](basictoken__factory.md#constructor)

### Properties

- [bytecode](basictoken__factory.md#bytecode)
- [interface](basictoken__factory.md#interface)
- [signer](basictoken__factory.md#signer)

### Methods

- [attach](basictoken__factory.md#attach)
- [connect](basictoken__factory.md#connect)
- [deploy](basictoken__factory.md#deploy)
- [getDeployTransaction](basictoken__factory.md#getdeploytransaction)
- [connect](basictoken__factory.md#connect)
- [fromSolidity](basictoken__factory.md#fromsolidity)
- [getContract](basictoken__factory.md#getcontract)
- [getContractAddress](basictoken__factory.md#getcontractaddress)
- [getInterface](basictoken__factory.md#getinterface)

## Constructors

### constructor

• **new BasicToken__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/BasicToken__factory.ts:10

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

▸ **attach**(`address`): [`BasicToken`](basictoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`BasicToken`](basictoken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/BasicToken__factory.ts:25

___

### connect

▸ **connect**(`signer`): [`BasicToken__factory`](basictoken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`BasicToken__factory`](basictoken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/BasicToken__factory.ts:28

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`BasicToken`](basictoken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`BasicToken`](basictoken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/BasicToken__factory.ts:15

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

packages/ethereum/types/factories/BasicToken__factory.ts:20

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`BasicToken`](basictoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`BasicToken`](basictoken.md)

#### Defined in

packages/ethereum/types/factories/BasicToken__factory.ts:31

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
