[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC20\_\_factory

# Class: ERC20\_\_factory

## Table of contents

### Constructors

- [constructor](ERC20__factory.md#constructor)

### Properties

- [abi](ERC20__factory.md#abi)

### Methods

- [connect](ERC20__factory.md#connect)
- [createInterface](ERC20__factory.md#createinterface)

## Constructors

### constructor

• **new ERC20__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = false; `inputs`: { `name`: `string` = "\_spender"; `type`: `string` = "address" }[] ; `name`: `string` = "approve"; `outputs`: { `name`: `string` = ""; `type`: `string` = "bool" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "Approval"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[]

#### Defined in

packages/ethereum/types/factories/ERC20__factory.ts:186

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ERC20`](ERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ERC20`](ERC20.md)

#### Defined in

packages/ethereum/types/factories/ERC20__factory.ts:190

___

### createInterface

▸ `Static` **createInterface**(): `ERC20Interface`

#### Returns

`ERC20Interface`

#### Defined in

packages/ethereum/types/factories/ERC20__factory.ts:187
