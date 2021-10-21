[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC1820Implementer\_\_factory

# Class: IERC1820Implementer\_\_factory

## Table of contents

### Constructors

- [constructor](IERC1820Implementer__factory.md#constructor)

### Properties

- [abi](IERC1820Implementer__factory.md#abi)

### Methods

- [connect](IERC1820Implementer__factory.md#connect)
- [createInterface](IERC1820Implementer__factory.md#createinterface)

## Constructors

### constructor

• **new IERC1820Implementer__factory**()

## Properties

### abi

▪ `Static` `Readonly` **abi**: { `inputs`: { `internalType`: `string` = "bytes32"; `name`: `string` = "interfaceHash"; `type`: `string` = "bytes32" }[] ; `name`: `string` = "canImplementInterfaceForAddress"; `outputs`: { `internalType`: `string` = "bytes32"; `name`: `string` = ""; `type`: `string` = "bytes32" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" }[] = `_abi`

#### Defined in

packages/ethereum/types/factories/IERC1820Implementer__factory.ts:40

## Methods

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`IERC1820Implementer`](IERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Defined in

packages/ethereum/types/factories/IERC1820Implementer__factory.ts:44

___

### createInterface

▸ `Static` **createInterface**(): `IERC1820ImplementerInterface`

#### Returns

`IERC1820ImplementerInterface`

#### Defined in

packages/ethereum/types/factories/IERC1820Implementer__factory.ts:41
