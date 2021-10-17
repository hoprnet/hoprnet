[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / LegacyERC20\_\_factory

# Class: LegacyERC20\_\_factory

## Table of contents

### Constructors

- [constructor](LegacyERC20__factory.md#constructor)

### Properties

- [abi](LegacyERC20__factory.md#abi)

### Methods

- [connect](LegacyERC20__factory.md#connect)
- [createInterface](LegacyERC20__factory.md#createinterface)

## Constructors

### constructor

• **new LegacyERC20__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: { `constant`: `boolean` = false; `inputs`: { `name`: `string` = "\_owner"; `type`: `string` = "address" }[] ; `name`: `string` = "transferFrom"; `outputs`: `any`[] = []; `payable`: `boolean` = false; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" }[]

#### Defined in

packages/ethereum/types/factories/LegacyERC20__factory.ts:53

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`LegacyERC20`](LegacyERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`LegacyERC20`](LegacyERC20.md)

#### Defined in

packages/ethereum/types/factories/LegacyERC20__factory.ts:57

___

### createInterface

▸ `Static` **createInterface**(): `LegacyERC20Interface`

#### Returns

`LegacyERC20Interface`

#### Defined in

packages/ethereum/types/factories/LegacyERC20__factory.ts:54
