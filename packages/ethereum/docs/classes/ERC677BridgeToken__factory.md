[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC677BridgeToken__factory

# Class: ERC677BridgeToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`ERC677BridgeToken__factory`**

## Table of contents

### Constructors

- [constructor](ERC677BridgeToken__factory.md#constructor)

### Properties

- [bytecode](ERC677BridgeToken__factory.md#bytecode)
- [interface](ERC677BridgeToken__factory.md#interface)
- [signer](ERC677BridgeToken__factory.md#signer)

### Methods

- [attach](ERC677BridgeToken__factory.md#attach)
- [connect](ERC677BridgeToken__factory.md#connect)
- [deploy](ERC677BridgeToken__factory.md#deploy)
- [getDeployTransaction](ERC677BridgeToken__factory.md#getdeploytransaction)
- [connect](ERC677BridgeToken__factory.md#connect)
- [fromSolidity](ERC677BridgeToken__factory.md#fromsolidity)
- [getContract](ERC677BridgeToken__factory.md#getcontract)
- [getContractAddress](ERC677BridgeToken__factory.md#getcontractaddress)
- [getInterface](ERC677BridgeToken__factory.md#getinterface)

## Constructors

### constructor

• **new ERC677BridgeToken__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/ERC677BridgeToken__factory.ts:17

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

▸ **attach**(`address`): [`ERC677BridgeToken`](ERC677BridgeToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`ERC677BridgeToken`](ERC677BridgeToken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/ERC677BridgeToken__factory.ts:47

___

### connect

▸ **connect**(`signer`): [`ERC677BridgeToken__factory`](ERC677BridgeToken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`ERC677BridgeToken__factory`](ERC677BridgeToken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/ERC677BridgeToken__factory.ts:50

___

### deploy

▸ **deploy**(`_name`, `_symbol`, `_decimals`, `overrides?`): `Promise`<[`ERC677BridgeToken`](ERC677BridgeToken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_name` | `string` |
| `_symbol` | `string` |
| `_decimals` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`ERC677BridgeToken`](ERC677BridgeToken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/ERC677BridgeToken__factory.ts:21

___

### getDeployTransaction

▸ **getDeployTransaction**(`_name`, `_symbol`, `_decimals`, `overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `_name` | `string` |
| `_symbol` | `string` |
| `_decimals` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/ERC677BridgeToken__factory.ts:34

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ERC677BridgeToken`](ERC677BridgeToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ERC677BridgeToken`](ERC677BridgeToken.md)

#### Defined in

packages/ethereum/types/factories/ERC677BridgeToken__factory.ts:53

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
