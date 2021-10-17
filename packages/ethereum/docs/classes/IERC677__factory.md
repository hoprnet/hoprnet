[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC677\_\_factory

# Class: IERC677\_\_factory

## Table of contents

### Constructors

- [constructor](IERC677__factory.md#constructor)

### Properties

- [abi](IERC677__factory.md#abi)

### Methods

- [connect](IERC677__factory.md#connect)
- [createInterface](IERC677__factory.md#createinterface)

## Constructors

### constructor

• **new IERC677__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `boolean` = false; `inputs`: { `indexed`: `boolean` = true; `internalType`: `string` = "address"; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "Approval"; `outputs`: `undefined` ; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" } \| { `anonymous`: `undefined` = false; `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "allowance"; `outputs`: { `internalType`: `string` = "uint256"; `name`: `string` = ""; `type`: `string` = "uint256" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" })[]

#### Defined in

packages/ethereum/types/factories/IERC677__factory.ts:304

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`IERC677`](IERC677.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`IERC677`](IERC677.md)

#### Defined in

packages/ethereum/types/factories/IERC677__factory.ts:308

___

### createInterface

▸ `Static` **createInterface**(): `IERC677Interface`

#### Returns

`IERC677Interface`

#### Defined in

packages/ethereum/types/factories/IERC677__factory.ts:305
