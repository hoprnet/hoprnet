[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC20\_\_factory

# Class: IERC20\_\_factory

## Table of contents

### Constructors

- [constructor](IERC20__factory.md#constructor)

### Properties

- [abi](IERC20__factory.md#abi)

### Methods

- [connect](IERC20__factory.md#connect)
- [createInterface](IERC20__factory.md#createinterface)

## Constructors

### constructor

• **new IERC20__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `boolean` = false; `inputs`: { `indexed`: `boolean` = true; `internalType`: `string` = "address"; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "Approval"; `outputs`: `undefined` ; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" } \| { `anonymous`: `undefined` = false; `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "allowance"; `outputs`: { `internalType`: `string` = "uint256"; `name`: `string` = ""; `type`: `string` = "uint256" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" })[] = `_abi`

#### Defined in

packages/ethereum/src/types/factories/IERC20__factory.ts:196

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`IERC20`](IERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`IERC20`](IERC20.md)

#### Defined in

packages/ethereum/src/types/factories/IERC20__factory.ts:200

___

### createInterface

▸ `Static` **createInterface**(): `IERC20Interface`

#### Returns

`IERC20Interface`

#### Defined in

packages/ethereum/src/types/factories/IERC20__factory.ts:197
