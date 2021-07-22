[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprWrapper__factory

# Class: HoprWrapper\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`HoprWrapper__factory`**

## Table of contents

### Constructors

- [constructor](HoprWrapper__factory.md#constructor)

### Properties

- [bytecode](HoprWrapper__factory.md#bytecode)
- [interface](HoprWrapper__factory.md#interface)
- [signer](HoprWrapper__factory.md#signer)

### Methods

- [attach](HoprWrapper__factory.md#attach)
- [connect](HoprWrapper__factory.md#connect)
- [deploy](HoprWrapper__factory.md#deploy)
- [getDeployTransaction](HoprWrapper__factory.md#getdeploytransaction)
- [connect](HoprWrapper__factory.md#connect)
- [fromSolidity](HoprWrapper__factory.md#fromsolidity)
- [getContract](HoprWrapper__factory.md#getcontract)
- [getContractAddress](HoprWrapper__factory.md#getcontractaddress)
- [getInterface](HoprWrapper__factory.md#getinterface)

## Constructors

### constructor

• **new HoprWrapper__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/HoprWrapper__factory.ts:11

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

▸ **attach**(`address`): [`HoprWrapper`](HoprWrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`HoprWrapper`](HoprWrapper.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/HoprWrapper__factory.ts:33

___

### connect

▸ **connect**(`signer`): [`HoprWrapper__factory`](HoprWrapper__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`HoprWrapper__factory`](HoprWrapper__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/HoprWrapper__factory.ts:36

___

### deploy

▸ **deploy**(`_xHOPR`, `_wxHOPR`, `overrides?`): `Promise`<[`HoprWrapper`](HoprWrapper.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_xHOPR` | `string` |
| `_wxHOPR` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`HoprWrapper`](HoprWrapper.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/HoprWrapper__factory.ts:15

___

### getDeployTransaction

▸ **getDeployTransaction**(`_xHOPR`, `_wxHOPR`, `overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `_xHOPR` | `string` |
| `_wxHOPR` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/HoprWrapper__factory.ts:26

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`HoprWrapper`](HoprWrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`HoprWrapper`](HoprWrapper.md)

#### Defined in

packages/ethereum/types/factories/HoprWrapper__factory.ts:39

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
