[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / PermittableToken__factory

# Class: PermittableToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`PermittableToken__factory`**

## Table of contents

### Constructors

- [constructor](PermittableToken__factory.md#constructor)

### Properties

- [bytecode](PermittableToken__factory.md#bytecode)
- [interface](PermittableToken__factory.md#interface)
- [signer](PermittableToken__factory.md#signer)

### Methods

- [attach](PermittableToken__factory.md#attach)
- [connect](PermittableToken__factory.md#connect)
- [deploy](PermittableToken__factory.md#deploy)
- [getDeployTransaction](PermittableToken__factory.md#getdeploytransaction)
- [connect](PermittableToken__factory.md#connect)
- [fromSolidity](PermittableToken__factory.md#fromsolidity)
- [getContract](PermittableToken__factory.md#getcontract)
- [getContractAddress](PermittableToken__factory.md#getcontractaddress)
- [getInterface](PermittableToken__factory.md#getinterface)

## Constructors

### constructor

• **new PermittableToken__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/PermittableToken__factory.ts:17

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

▸ **attach**(`address`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/PermittableToken__factory.ts:51

___

### connect

▸ **connect**(`signer`): [`PermittableToken__factory`](PermittableToken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`PermittableToken__factory`](PermittableToken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/PermittableToken__factory.ts:54

___

### deploy

▸ **deploy**(`_name`, `_symbol`, `_decimals`, `_chainId`, `overrides?`): `Promise`<[`PermittableToken`](PermittableToken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_name` | `string` |
| `_symbol` | `string` |
| `_decimals` | `BigNumberish` |
| `_chainId` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`PermittableToken`](PermittableToken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/PermittableToken__factory.ts:21

___

### getDeployTransaction

▸ **getDeployTransaction**(`_name`, `_symbol`, `_decimals`, `_chainId`, `overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `_name` | `string` |
| `_symbol` | `string` |
| `_decimals` | `BigNumberish` |
| `_chainId` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/PermittableToken__factory.ts:36

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Defined in

packages/ethereum/types/factories/PermittableToken__factory.ts:57

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
