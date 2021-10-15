[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IBurnableMintableERC677Token\_\_factory

# Class: IBurnableMintableERC677Token\_\_factory

## Table of contents

### Constructors

- [constructor](IBurnableMintableERC677Token__factory.md#constructor)

### Properties

- [abi](IBurnableMintableERC677Token__factory.md#abi)

### Methods

- [connect](IBurnableMintableERC677Token__factory.md#connect)
- [createInterface](IBurnableMintableERC677Token__factory.md#createinterface)

## Constructors

### constructor

• **new IBurnableMintableERC677Token__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = false; `inputs`: { `name`: `string` = "\_spender"; `type`: `string` = "address" }[] ; `name`: `string` = "approve"; `outputs`: { `name`: `string` = ""; `type`: `string` = "bool" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "from"; `type`: `string` = "address" }[] ; `name`: `string` = "Transfer"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[]

#### Defined in

packages/ethereum/types/factories/IBurnableMintableERC677Token__factory.ts:344

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`IBurnableMintableERC677Token`](IBurnableMintableERC677Token.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`IBurnableMintableERC677Token`](IBurnableMintableERC677Token.md)

#### Defined in

packages/ethereum/types/factories/IBurnableMintableERC677Token__factory.ts:348

___

### createInterface

▸ `Static` **createInterface**(): `IBurnableMintableERC677TokenInterface`

#### Returns

`IBurnableMintableERC677TokenInterface`

#### Defined in

packages/ethereum/types/factories/IBurnableMintableERC677Token__factory.ts:345
