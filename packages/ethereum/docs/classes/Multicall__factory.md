[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / Multicall__factory

# Class: Multicall\_\_factory

## Table of contents

### Constructors

- [constructor](Multicall__factory.md#constructor)

### Properties

- [abi](Multicall__factory.md#abi)

### Methods

- [connect](Multicall__factory.md#connect)
- [createInterface](Multicall__factory.md#createinterface)

## Constructors

### constructor

• **new Multicall__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: { `inputs`: { `internalType`: `string` = "bytes[]"; `name`: `string` = "data"; `type`: `string` = "bytes[]" }[] ; `name`: `string` = "multicall"; `outputs`: { `internalType`: `string` = "bytes[]"; `name`: `string` = "results"; `type`: `string` = "bytes[]" }[] ; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" }[]

#### Defined in

packages/ethereum/types/factories/Multicall__factory.ts:32

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`Multicall`](Multicall.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`Multicall`](Multicall.md)

#### Defined in

packages/ethereum/types/factories/Multicall__factory.ts:36

___

### createInterface

▸ `Static` **createInterface**(): `MulticallInterface`

#### Returns

`MulticallInterface`

#### Defined in

packages/ethereum/types/factories/Multicall__factory.ts:33
