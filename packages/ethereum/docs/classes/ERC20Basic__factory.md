[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC20Basic\_\_factory

# Class: ERC20Basic\_\_factory

## Table of contents

### Constructors

- [constructor](ERC20Basic__factory.md#constructor)

### Properties

- [abi](ERC20Basic__factory.md#abi)

### Methods

- [connect](ERC20Basic__factory.md#connect)
- [createInterface](ERC20Basic__factory.md#createinterface)

## Constructors

### constructor

• **new ERC20Basic__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = true; `inputs`: { `name`: `string` = "\_who"; `type`: `string` = "address" }[] ; `name`: `string` = "balanceOf"; `outputs`: { `name`: `string` = ""; `type`: `string` = "uint256" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "view"; `type`: `string` = "function" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "from"; `type`: `string` = "address" }[] ; `name`: `string` = "Transfer"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[]

#### Defined in

packages/ethereum/types/factories/ERC20Basic__factory.ts:91

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ERC20Basic`](ERC20Basic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ERC20Basic`](ERC20Basic.md)

#### Defined in

packages/ethereum/types/factories/ERC20Basic__factory.ts:95

___

### createInterface

▸ `Static` **createInterface**(): `ERC20BasicInterface`

#### Returns

`ERC20BasicInterface`

#### Defined in

packages/ethereum/types/factories/ERC20Basic__factory.ts:92
