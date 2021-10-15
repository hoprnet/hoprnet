[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / BurnableToken\_\_factory

# Class: BurnableToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`BurnableToken__factory`**

## Table of contents

### Constructors

- [constructor](BurnableToken__factory.md#constructor)

### Properties

- [bytecode](BurnableToken__factory.md#bytecode)
- [interface](BurnableToken__factory.md#interface)
- [signer](BurnableToken__factory.md#signer)
- [abi](BurnableToken__factory.md#abi)
- [bytecode](BurnableToken__factory.md#bytecode)

### Methods

- [attach](BurnableToken__factory.md#attach)
- [connect](BurnableToken__factory.md#connect)
- [deploy](BurnableToken__factory.md#deploy)
- [getDeployTransaction](BurnableToken__factory.md#getdeploytransaction)
- [connect](BurnableToken__factory.md#connect)
- [createInterface](BurnableToken__factory.md#createinterface)
- [fromSolidity](BurnableToken__factory.md#fromsolidity)
- [getContract](BurnableToken__factory.md#getcontract)
- [getContractAddress](BurnableToken__factory.md#getcontractaddress)
- [getInterface](BurnableToken__factory.md#getinterface)

## Constructors

### constructor

• **new BurnableToken__factory**(...`args`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `...args` | [contractInterface: ContractInterface, bytecode: BytesLike \| Object, signer?: Signer] \| [signer: Signer] |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:125

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

___

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = true; `inputs`: { `name`: `string` = "\_owner"; `type`: `string` = "address" }[] ; `name`: `string` = "balanceOf"; `outputs`: { `name`: `string` = ""; `type`: `string` = "uint256" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "view"; `type`: `string` = "function" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "burner"; `type`: `string` = "address" }[] ; `name`: `string` = "Burn"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[]

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:152

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405234801561001057600080fd5b5061035f806100206000396000f3006080604052600436106100615763ffffffff7c010000000000000000000000000000000000000000000000000000000060003504166318160ddd811461006657806342966c681461008d57806370a08231146100a7578063a9059cbb146100c8575b600080fd5b34801561007257600080fd5b5061007b610100565b60408051918252519081900360200190f35b34801561009957600080fd5b506100a5600435610106565b005b3480156100b357600080fd5b5061007b600160a060020a0360043516610113565b3480156100d457600080fd5b506100ec600160a060020a036004351660243561012e565b604080519115158252519081900360200190f35b60015490565b610110338261020d565b50565b600160a060020a031660009081526020819052604090205490565b3360009081526020819052604081205482111561014a57600080fd5b600160a060020a038316151561015f57600080fd5b3360009081526020819052604090205461017f908363ffffffff61030e16565b3360009081526020819052604080822092909255600160a060020a038516815220546101b1908363ffffffff61032016565b600160a060020a038416600081815260208181526040918290209390935580518581529051919233927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9281900390910190a350600192915050565b600160a060020a03821660009081526020819052604090205481111561023257600080fd5b600160a060020a03821660009081526020819052604090205461025b908263ffffffff61030e16565b600160a060020a038316600090815260208190526040902055600154610287908263ffffffff61030e16565b600155604080518281529051600160a060020a038416917fcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5919081900360200190a2604080518281529051600091600160a060020a038516917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9181900360200190a35050565b60008282111561031a57fe5b50900390565b8181018281101561032d57fe5b929150505600a165627a7a723058205d56505b728c01fa1b6266d644f4fc260f3bfc895e31ee32caab89114ad294160029"``

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:151

## Methods

### attach

▸ **attach**(`address`): [`BurnableToken`](BurnableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`BurnableToken`](BurnableToken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:145

___

### connect

▸ **connect**(`signer`): [`BurnableToken__factory`](BurnableToken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`BurnableToken__factory`](BurnableToken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:148

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`BurnableToken`](BurnableToken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`BurnableToken`](BurnableToken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:135

___

### getDeployTransaction

▸ **getDeployTransaction**(`overrides?`): `TransactionRequest`

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`TransactionRequest`

#### Overrides

ContractFactory.getDeployTransaction

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:140

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`BurnableToken`](BurnableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`BurnableToken`](BurnableToken.md)

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:156

___

### createInterface

▸ `Static` **createInterface**(): `BurnableTokenInterface`

#### Returns

`BurnableTokenInterface`

#### Defined in

packages/ethereum/types/factories/BurnableToken__factory.ts:153

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
