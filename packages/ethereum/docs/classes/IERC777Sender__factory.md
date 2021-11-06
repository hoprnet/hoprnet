[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC777Sender\_\_factory

# Class: IERC777Sender\_\_factory

## Table of contents

### Constructors

- [constructor](IERC777Sender__factory.md#constructor)

### Properties

- [abi](IERC777Sender__factory.md#abi)

### Methods

- [connect](IERC777Sender__factory.md#connect)
- [createInterface](IERC777Sender__factory.md#createinterface)

## Constructors

### constructor

• **new IERC777Sender__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: { `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "operator"; `type`: `string` = "address" }[] ; `name`: `string` = "tokensToSend"; `outputs`: `any`[] = []; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" }[] = `_abi`

#### Defined in

packages/ethereum/src/types/factories/IERC777Sender__factory.ts:51

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`IERC777Sender`](IERC777Sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`IERC777Sender`](IERC777Sender.md)

#### Defined in

packages/ethereum/src/types/factories/IERC777Sender__factory.ts:55

___

### createInterface

▸ `Static` **createInterface**(): `IERC777SenderInterface`

#### Returns

`IERC777SenderInterface`

#### Defined in

packages/ethereum/src/types/factories/IERC777Sender__factory.ts:52
