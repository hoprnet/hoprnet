[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC777Snapshot\_\_factory

# Class: ERC777Snapshot\_\_factory

## Table of contents

### Constructors

- [constructor](ERC777Snapshot__factory.md#constructor)

### Properties

- [abi](ERC777Snapshot__factory.md#abi)

### Methods

- [connect](ERC777Snapshot__factory.md#connect)
- [createInterface](ERC777Snapshot__factory.md#createinterface)

## Constructors

### constructor

• **new ERC777Snapshot__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `boolean` = false; `inputs`: { `indexed`: `boolean` = true; `internalType`: `string` = "address"; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "Approval"; `outputs`: `undefined` ; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" } \| { `anonymous`: `undefined` = false; `inputs`: { `internalType`: `string` = "address"; `name`: `string` = ""; `type`: `string` = "address" }[] ; `name`: `string` = "accountSnapshots"; `outputs`: { `internalType`: `string` = "uint128"; `name`: `string` = "fromBlock"; `type`: `string` = "uint128" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" })[]

#### Defined in

packages/ethereum/types/factories/ERC777Snapshot__factory.ts:667

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`ERC777Snapshot`](ERC777Snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`ERC777Snapshot`](ERC777Snapshot.md)

#### Defined in

packages/ethereum/types/factories/ERC777Snapshot__factory.ts:671

___

### createInterface

▸ `Static` **createInterface**(): `ERC777SnapshotInterface`

#### Returns

`ERC777SnapshotInterface`

#### Defined in

packages/ethereum/types/factories/ERC777Snapshot__factory.ts:668
