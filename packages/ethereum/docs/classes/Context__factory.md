[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / Context__factory

# Class: Context\_\_factory

## Table of contents

### Constructors

- [constructor](Context__factory.md#constructor)

### Properties

- [abi](Context__factory.md#abi)

### Methods

- [connect](Context__factory.md#connect)
- [createInterface](Context__factory.md#createinterface)

## Constructors

### constructor

• **new Context__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: { `inputs`: `any`[] = []; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "constructor" }[]

#### Defined in

packages/ethereum/types/factories/Context__factory.ts:18

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`Context`](Context.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`Context`](Context.md)

#### Defined in

packages/ethereum/types/factories/Context__factory.ts:22

___

### createInterface

▸ `Static` **createInterface**(): `ContextInterface`

#### Returns

`ContextInterface`

#### Defined in

packages/ethereum/types/factories/Context__factory.ts:19
