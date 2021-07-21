[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprToken__factory

# Class: HoprToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`HoprToken__factory`**

## Table of contents

### Constructors

- [constructor](HoprToken__factory.md#constructor)

### Properties

- [bytecode](HoprToken__factory.md#bytecode)
- [interface](HoprToken__factory.md#interface)
- [signer](HoprToken__factory.md#signer)

### Methods

- [attach](HoprToken__factory.md#attach)
- [connect](HoprToken__factory.md#connect)
- [deploy](HoprToken__factory.md#deploy)
- [getDeployTransaction](HoprToken__factory.md#getdeploytransaction)
- [connect](HoprToken__factory.md#connect)
- [fromSolidity](HoprToken__factory.md#fromsolidity)
- [getContract](HoprToken__factory.md#getcontract)
- [getContractAddress](HoprToken__factory.md#getcontractaddress)
- [getInterface](HoprToken__factory.md#getinterface)

## Constructors

### constructor

• **new HoprToken__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/HoprToken__factory.ts:11

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

▸ **attach**(`address`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/HoprToken__factory.ts:25

___

### connect

▸ **connect**(`signer`): [`HoprToken__factory`](HoprToken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`HoprToken__factory`](HoprToken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/HoprToken__factory.ts:28

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`HoprToken`](HoprToken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`HoprToken`](HoprToken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/HoprToken__factory.ts:15

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

packages/ethereum/types/factories/HoprToken__factory.ts:20

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Defined in

packages/ethereum/types/factories/HoprToken__factory.ts:31

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
