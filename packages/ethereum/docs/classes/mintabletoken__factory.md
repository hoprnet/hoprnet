[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / MintableToken__factory

# Class: MintableToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`MintableToken__factory`**

## Table of contents

### Constructors

- [constructor](mintabletoken__factory.md#constructor)

### Properties

- [bytecode](mintabletoken__factory.md#bytecode)
- [interface](mintabletoken__factory.md#interface)
- [signer](mintabletoken__factory.md#signer)

### Methods

- [attach](mintabletoken__factory.md#attach)
- [connect](mintabletoken__factory.md#connect)
- [deploy](mintabletoken__factory.md#deploy)
- [getDeployTransaction](mintabletoken__factory.md#getdeploytransaction)
- [connect](mintabletoken__factory.md#connect)
- [fromSolidity](mintabletoken__factory.md#fromsolidity)
- [getContract](mintabletoken__factory.md#getcontract)
- [getContractAddress](mintabletoken__factory.md#getcontractaddress)
- [getInterface](mintabletoken__factory.md#getinterface)

## Constructors

### constructor

• **new MintableToken__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:10

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

▸ **attach**(`address`): [`MintableToken`](mintabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`MintableToken`](mintabletoken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:25

___

### connect

▸ **connect**(`signer`): [`MintableToken__factory`](mintabletoken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`MintableToken__factory`](mintabletoken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:28

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`MintableToken`](mintabletoken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`MintableToken`](mintabletoken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:15

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

packages/ethereum/types/factories/MintableToken__factory.ts:20

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`MintableToken`](mintabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`MintableToken`](mintabletoken.md)

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:31

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
