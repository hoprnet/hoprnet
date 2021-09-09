[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC777Recipient__factory

# Class: IERC777Recipient\_\_factory

## Table of contents

### Constructors

- [constructor](IERC777Recipient__factory.md#constructor)

### Properties

- [abi](IERC777Recipient__factory.md#abi)

### Methods

- [connect](IERC777Recipient__factory.md#connect)
- [createInterface](IERC777Recipient__factory.md#createinterface)

## Constructors

### constructor

• **new IERC777Recipient__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: { `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "operator"; `type`: `string` = "address" }[] ; `name`: `string` = "tokensReceived"; `outputs`: `any`[] = []; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" }[]

#### Defined in

packages/ethereum/types/factories/IERC777Recipient__factory.ts:54

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`IERC777Recipient`](IERC777Recipient.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`IERC777Recipient`](IERC777Recipient.md)

#### Defined in

packages/ethereum/types/factories/IERC777Recipient__factory.ts:58

___

### createInterface

▸ `Static` **createInterface**(): `IERC777RecipientInterface`

#### Returns

`IERC777RecipientInterface`

#### Defined in

packages/ethereum/types/factories/IERC777Recipient__factory.ts:55
