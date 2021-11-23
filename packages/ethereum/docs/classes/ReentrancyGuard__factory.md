[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ReentrancyGuard\_\_factory

# Class: ReentrancyGuard\_\_factory

## Table of contents

### Constructors

- [constructor](ReentrancyGuard__factory.md#constructor)

### Properties

- [abi](ReentrancyGuard__factory.md#abi)

### Methods

- [connect](ReentrancyGuard__factory.md#connect)
- [createInterface](ReentrancyGuard__factory.md#createinterface)

## Constructors

### constructor

• **new ReentrancyGuard__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: { `inputs`: `any`[] = []; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "constructor" }[] = `_abi`

#### Defined in

packages/ethereum/src/types/factories/ReentrancyGuard__factory.ts:21

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Defined in

packages/ethereum/src/types/factories/ReentrancyGuard__factory.ts:25

___

### createInterface

▸ `Static` **createInterface**(): `ReentrancyGuardInterface`

#### Returns

`ReentrancyGuardInterface`

#### Defined in

packages/ethereum/src/types/factories/ReentrancyGuard__factory.ts:22
