[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / Sacrifice__factory

# Class: Sacrifice\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`Sacrifice__factory`**

## Table of contents

### Constructors

- [constructor](Sacrifice__factory.md#constructor)

### Properties

- [bytecode](Sacrifice__factory.md#bytecode)
- [interface](Sacrifice__factory.md#interface)
- [signer](Sacrifice__factory.md#signer)
- [abi](Sacrifice__factory.md#abi)
- [bytecode](Sacrifice__factory.md#bytecode)

### Methods

- [attach](Sacrifice__factory.md#attach)
- [connect](Sacrifice__factory.md#connect)
- [deploy](Sacrifice__factory.md#deploy)
- [getDeployTransaction](Sacrifice__factory.md#getdeploytransaction)
- [connect](Sacrifice__factory.md#connect)
- [createInterface](Sacrifice__factory.md#createinterface)
- [fromSolidity](Sacrifice__factory.md#fromsolidity)
- [getContract](Sacrifice__factory.md#getcontract)
- [getContractAddress](Sacrifice__factory.md#getcontractaddress)
- [getInterface](Sacrifice__factory.md#getinterface)

## Constructors

### constructor

• **new Sacrifice__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:33

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

▪ `Static` `Readonly` **abi**: { `inputs`: { `name`: `string` = "\_recipient"; `type`: `string` = "address" }[] ; `payable`: `boolean` = true; `stateMutability`: `string` = "payable"; `type`: `string` = "constructor" }[]

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:56

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405260405160208060218339810160405251600160a060020a038116ff00"``

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:55

## Methods

### attach

▸ **attach**(`address`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:49

___

### connect

▸ **connect**(`signer`): [`Sacrifice__factory`](Sacrifice__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`Sacrifice__factory`](Sacrifice__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:52

___

### deploy

▸ **deploy**(`_recipient`, `overrides?`): `Promise`<[`Sacrifice`](Sacrifice.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_recipient` | `string` |
| `overrides?` | `PayableOverrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`Sacrifice`](Sacrifice.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:37

___

### getDeployTransaction

▸ **getDeployTransaction**(`_recipient`, `overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `_recipient` | `string` |
| `overrides?` | `PayableOverrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:43

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:60

___

### createInterface

▸ `Static` **createInterface**(): `SacrificeInterface`

#### Returns

`SacrificeInterface`

#### Defined in

packages/ethereum/types/factories/Sacrifice__factory.ts:57

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
