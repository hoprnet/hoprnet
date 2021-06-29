[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprChannels__factory

# Class: HoprChannels\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`HoprChannels__factory`**

## Table of contents

### Constructors

- [constructor](hoprchannels__factory.md#constructor)

### Properties

- [bytecode](hoprchannels__factory.md#bytecode)
- [interface](hoprchannels__factory.md#interface)
- [signer](hoprchannels__factory.md#signer)

### Methods

- [attach](hoprchannels__factory.md#attach)
- [connect](hoprchannels__factory.md#connect)
- [deploy](hoprchannels__factory.md#deploy)
- [getDeployTransaction](hoprchannels__factory.md#getdeploytransaction)
- [connect](hoprchannels__factory.md#connect)
- [fromSolidity](hoprchannels__factory.md#fromsolidity)
- [getContract](hoprchannels__factory.md#getcontract)
- [getContractAddress](hoprchannels__factory.md#getcontractaddress)
- [getInterface](hoprchannels__factory.md#getinterface)

## Constructors

### constructor

• **new HoprChannels__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/HoprChannels__factory.ts:16

## Properties

### bytecode

• `Readonly` **bytecode**: `string`

#### Inherited from

ContractFactory.bytecode

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:135

___

### interface

• `Readonly` **interface**: `Interface`

#### Inherited from

ContractFactory.interface

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:134

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

ContractFactory.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:136

## Methods

### attach

▸ **attach**(`address`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/HoprChannels__factory.ts:39

___

### connect

▸ **connect**(`signer`): [`HoprChannels__factory`](hoprchannels__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`HoprChannels__factory`](hoprchannels__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/HoprChannels__factory.ts:42

___

### deploy

▸ **deploy**(`_token`, `_secsClosure`, `overrides?`): `Promise`<[`HoprChannels`](hoprchannels.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_token` | `string` |
| `_secsClosure` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`HoprChannels`](hoprchannels.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/HoprChannels__factory.ts:21

___

### getDeployTransaction

▸ **getDeployTransaction**(`_token`, `_secsClosure`, `overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `_token` | `string` |
| `_secsClosure` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/HoprChannels__factory.ts:32

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Defined in

packages/ethereum/types/factories/HoprChannels__factory.ts:45

___

### fromSolidity

▸ `Static` **fromSolidity**(`compilerOutput`, `signer?`): `ContractFactory`

#### Parameters

| Name | Type |
| :------ | :------ |
| `compilerOutput` | `any` |
| `signer?` | `Signer` |

#### Returns

`ContractFactory`

#### Inherited from

ContractFactory.fromSolidity

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:144

___

### getContract

▸ `Static` **getContract**(`address`, `contractInterface`, `signer?`): `Contract`

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `contractInterface` | `ContractInterface` |
| `signer?` | `Signer` |

#### Returns

`Contract`

#### Inherited from

ContractFactory.getContract

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:150

___

### getContractAddress

▸ `Static` **getContractAddress**(`tx`): `string`

#### Parameters

| Name | Type |
| :------ | :------ |
| `tx` | `Object` |
| `tx.from` | `string` |
| `tx.nonce` | `number` \| `BigNumber` \| `BytesLike` |

#### Returns

`string`

#### Inherited from

ContractFactory.getContractAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:146

___

### getInterface

▸ `Static` **getInterface**(`contractInterface`): `Interface`

#### Parameters

| Name | Type |
| :------ | :------ |
| `contractInterface` | `ContractInterface` |

#### Returns

`Interface`

#### Inherited from

ContractFactory.getInterface

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:145
