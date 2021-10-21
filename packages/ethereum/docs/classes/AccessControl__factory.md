[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / AccessControl\_\_factory

# Class: AccessControl\_\_factory

## Table of contents

### Constructors

- [constructor](AccessControl__factory.md#constructor)

### Properties

- [abi](AccessControl__factory.md#abi)

### Methods

- [connect](AccessControl__factory.md#connect)
- [createInterface](AccessControl__factory.md#createinterface)

## Constructors

### constructor

• **new AccessControl__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `boolean` = false; `inputs`: { `indexed`: `boolean` = true; `internalType`: `string` = "bytes32"; `name`: `string` = "role"; `type`: `string` = "bytes32" }[] ; `name`: `string` = "RoleAdminChanged"; `outputs`: `undefined` ; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" } \| { `anonymous`: `undefined` = false; `inputs`: { `internalType`: `string` = "bytes32"; `name`: `string` = "role"; `type`: `string` = "bytes32" }[] ; `name`: `string` = "getRoleAdmin"; `outputs`: { `internalType`: `string` = "bytes32"; `name`: `string` = ""; `type`: `string` = "bytes32" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" })[] = `_abi`

#### Defined in

packages/ethereum/types/factories/AccessControl__factory.ts:241

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`AccessControl`](AccessControl.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`AccessControl`](AccessControl.md)

#### Defined in

packages/ethereum/types/factories/AccessControl__factory.ts:245

___

### createInterface

▸ `Static` **createInterface**(): `AccessControlInterface`

#### Returns

`AccessControlInterface`

#### Defined in

packages/ethereum/types/factories/AccessControl__factory.ts:242
