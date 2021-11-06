[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / DetailedERC20\_\_factory

# Class: DetailedERC20\_\_factory

## Table of contents

### Constructors

- [constructor](DetailedERC20__factory.md#constructor)

### Properties

- [abi](DetailedERC20__factory.md#abi)

### Methods

- [connect](DetailedERC20__factory.md#connect)
- [createInterface](DetailedERC20__factory.md#createinterface)

## Constructors

### constructor

• **new DetailedERC20__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = false; `inputs`: { `name`: `string` = "\_spender"; `type`: `string` = "address" }[] ; `name`: `string` = "approve"; `outputs`: { `name`: `string` = ""; `type`: `string` = "bool" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" } \| { `anonymous`: `undefined` = false; `constant`: `undefined` = true; `inputs`: { `name`: `string` = "\_name"; `type`: `string` = "string" }[] ; `name`: `undefined` = "allowance"; `outputs`: `undefined` ; `payable`: `boolean` = false; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "constructor" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "Approval"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[] = `_abi`

#### Defined in

packages/ethereum/src/types/factories/DetailedERC20__factory.ts:247

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`DetailedERC20`](DetailedERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`DetailedERC20`](DetailedERC20.md)

#### Defined in

packages/ethereum/src/types/factories/DetailedERC20__factory.ts:251

___

### createInterface

▸ `Static` **createInterface**(): `DetailedERC20Interface`

#### Returns

`DetailedERC20Interface`

#### Defined in

packages/ethereum/src/types/factories/DetailedERC20__factory.ts:248
