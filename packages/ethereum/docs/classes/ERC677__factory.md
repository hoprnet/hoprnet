[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC677\_\_factory

# Class: ERC677\_\_factory

## Table of contents

### Constructors

- [constructor](ERC677__factory.md#constructor)

### Properties

- [abi](ERC677__factory.md#abi)

### Methods

- [connect](ERC677__factory.md#connect)
- [createInterface](ERC677__factory.md#createinterface)

## Constructors

### constructor

• **new ERC677__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = false; `inputs`: { `name`: `string` = "\_spender"; `type`: `string` = "address" }[] ; `name`: `string` = "approve"; `outputs`: { `name`: `string` = ""; `type`: `string` = "bool" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "from"; `type`: `string` = "address" }[] ; `name`: `string` = "Transfer"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[] = `_abi`

#### Defined in

packages/ethereum/types/factories/ERC677__factory.ts:286

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ERC677`](ERC677.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ERC677`](ERC677.md)

#### Defined in

packages/ethereum/types/factories/ERC677__factory.ts:290

___

### createInterface

▸ `Static` **createInterface**(): `ERC677Interface`

#### Returns

`ERC677Interface`

#### Defined in

packages/ethereum/types/factories/ERC677__factory.ts:287
