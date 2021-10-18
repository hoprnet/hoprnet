[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC1820Registry\_\_factory

# Class: IERC1820Registry\_\_factory

## Table of contents

### Constructors

- [constructor](IERC1820Registry__factory.md#constructor)

### Properties

- [abi](IERC1820Registry__factory.md#abi)

### Methods

- [connect](IERC1820Registry__factory.md#connect)
- [createInterface](IERC1820Registry__factory.md#createinterface)

## Constructors

### constructor

• **new IERC1820Registry__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `boolean` = false; `inputs`: { `indexed`: `boolean` = true; `internalType`: `string` = "address"; `name`: `string` = "account"; `type`: `string` = "address" }[] ; `name`: `string` = "InterfaceImplementerSet"; `outputs`: `undefined` ; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" } \| { `anonymous`: `undefined` = false; `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "account"; `type`: `string` = "address" }[] ; `name`: `string` = "getInterfaceImplementer"; `outputs`: { `internalType`: `string` = "address"; `name`: `string` = ""; `type`: `string` = "address" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" })[] = `_abi`

#### Defined in

packages/ethereum/types/factories/IERC1820Registry__factory.ts:229

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`IERC1820Registry`](IERC1820Registry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Defined in

packages/ethereum/types/factories/IERC1820Registry__factory.ts:233

___

### createInterface

▸ `Static` **createInterface**(): `IERC1820RegistryInterface`

#### Returns

`IERC1820RegistryInterface`

#### Defined in

packages/ethereum/types/factories/IERC1820Registry__factory.ts:230
