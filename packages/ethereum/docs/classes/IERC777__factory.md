[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC777\_\_factory

# Class: IERC777\_\_factory

## Table of contents

### Constructors

- [constructor](IERC777__factory.md#constructor)

### Properties

- [abi](IERC777__factory.md#abi)

### Methods

- [connect](IERC777__factory.md#connect)
- [createInterface](IERC777__factory.md#createinterface)

## Constructors

### constructor

• **new IERC777__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `boolean` = false; `inputs`: { `indexed`: `boolean` = true; `internalType`: `string` = "address"; `name`: `string` = "operator"; `type`: `string` = "address" }[] ; `name`: `string` = "AuthorizedOperator"; `outputs`: `undefined` ; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" } \| { `anonymous`: `undefined` = false; `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "balanceOf"; `outputs`: { `internalType`: `string` = "uint256"; `name`: `string` = ""; `type`: `string` = "uint256" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" })[] = `_abi`

#### Defined in

packages/ethereum/types/factories/IERC777__factory.ts:404

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`IERC777`](IERC777.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`IERC777`](IERC777.md)

#### Defined in

packages/ethereum/types/factories/IERC777__factory.ts:408

___

### createInterface

▸ `Static` **createInterface**(): `IERC777Interface`

#### Returns

`IERC777Interface`

#### Defined in

packages/ethereum/types/factories/IERC777__factory.ts:405
