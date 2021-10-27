[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / Ownable\_\_factory

# Class: Ownable\_\_factory

## Table of contents

### Constructors

- [constructor](Ownable__factory.md#constructor)

### Properties

- [abi](Ownable__factory.md#abi)

### Methods

- [connect](Ownable__factory.md#connect)
- [createInterface](Ownable__factory.md#createinterface)

## Constructors

### constructor

• **new Ownable__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `boolean` = false; `inputs`: { `indexed`: `boolean` = true; `internalType`: `string` = "address"; `name`: `string` = "previousOwner"; `type`: `string` = "address" }[] ; `name`: `string` = "OwnershipTransferred"; `outputs`: `undefined` ; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" } \| { `anonymous`: `undefined` = false; `inputs`: `any`[] = []; `name`: `string` = "owner"; `outputs`: { `internalType`: `string` = "address"; `name`: `string` = ""; `type`: `string` = "address" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" } \| { `anonymous`: `undefined` = false; `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "newOwner"; `type`: `string` = "address" }[] ; `name`: `string` = "transferOwnership"; `outputs`: `any`[] = []; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" })[] = `_abi`

#### Defined in

packages/ethereum/types/factories/Ownable__factory.ts:65

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`Ownable`](Ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`Ownable`](Ownable.md)

#### Defined in

packages/ethereum/types/factories/Ownable__factory.ts:69

___

### createInterface

▸ `Static` **createInterface**(): `OwnableInterface`

#### Returns

`OwnableInterface`

#### Defined in

packages/ethereum/types/factories/Ownable__factory.ts:66
