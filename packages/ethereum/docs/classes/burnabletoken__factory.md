[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / BurnableToken__factory

# Class: BurnableToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`BurnableToken__factory`**

## Table of contents

### Constructors

- [constructor](burnabletoken__factory.md#constructor)

### Properties

- [bytecode](burnabletoken__factory.md#bytecode)
- [interface](burnabletoken__factory.md#interface)
- [signer](burnabletoken__factory.md#signer)

### Methods

- [attach](burnabletoken__factory.md#attach)
- [connect](burnabletoken__factory.md#connect)
- [deploy](burnabletoken__factory.md#deploy)
- [getDeployTransaction](burnabletoken__factory.md#getdeploytransaction)
- [connect](burnabletoken__factory.md#connect)
- [fromSolidity](burnabletoken__factory.md#fromsolidity)
- [getContract](burnabletoken__factory.md#getcontract)
- [getContractAddress](burnabletoken__factory.md#getcontractaddress)
- [getInterface](burnabletoken__factory.md#getinterface)

## Constructors

### constructor

• **new BurnableToken__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:10

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

▸ **attach**(`address`): [`BurnableToken`](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`BurnableToken`](burnabletoken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:25

___

### connect

▸ **connect**(`signer`): [`BurnableToken__factory`](burnabletoken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`BurnableToken__factory`](burnabletoken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:28

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`BurnableToken`](burnabletoken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`BurnableToken`](burnabletoken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:15

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

packages/ethereum/types/factories/BurnableToken__factory.ts:20

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`BurnableToken`](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`BurnableToken`](burnabletoken.md)

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:31

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
