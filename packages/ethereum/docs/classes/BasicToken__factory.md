[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / BasicToken\_\_factory

# Class: BasicToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`BasicToken__factory`**

## Table of contents

### Constructors

- [constructor](BasicToken__factory.md#constructor)

### Properties

- [bytecode](BasicToken__factory.md#bytecode)
- [interface](BasicToken__factory.md#interface)
- [signer](BasicToken__factory.md#signer)
- [abi](BasicToken__factory.md#abi)
- [bytecode](BasicToken__factory.md#bytecode)

### Methods

- [attach](BasicToken__factory.md#attach)
- [connect](BasicToken__factory.md#connect)
- [deploy](BasicToken__factory.md#deploy)
- [getDeployTransaction](BasicToken__factory.md#getdeploytransaction)
- [connect](BasicToken__factory.md#connect)
- [createInterface](BasicToken__factory.md#createinterface)
- [fromSolidity](BasicToken__factory.md#fromsolidity)
- [getContract](BasicToken__factory.md#getcontract)
- [getContractAddress](BasicToken__factory.md#getcontractaddress)
- [getInterface](BasicToken__factory.md#getinterface)

## Constructors

### constructor

• **new BasicToken__factory**(...`args`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `...args` | [contractInterface: ContractInterface, bytecode: BytesLike \| Object, signer?: Signer] \| [signer: Signer] |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/src/types/factories/BasicToken__factory.ts:94

## Properties

### bytecode

• `Readonly` **bytecode**: `string`

#### Inherited from

ContractFactory.bytecode

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:137

___

### interface

• `Readonly` **interface**: `Interface`

#### Inherited from

ContractFactory.interface

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:136

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

ContractFactory.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:138

___

### abi

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = true; `inputs`: { `name`: `string` = "\_owner"; `type`: `string` = "address" }[] ; `name`: `string` = "balanceOf"; `outputs`: { `name`: `string` = ""; `type`: `string` = "uint256" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "view"; `type`: `string` = "function" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "from"; `type`: `string` = "address" }[] ; `name`: `string` = "Transfer"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[] = `_abi`

#### Defined in

packages/ethereum/src/types/factories/BasicToken__factory.ts:121

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405234801561001057600080fd5b5061027a806100206000396000f3006080604052600436106100565763ffffffff7c010000000000000000000000000000000000000000000000000000000060003504166318160ddd811461005b57806370a0823114610082578063a9059cbb146100b0575b600080fd5b34801561006757600080fd5b506100706100f5565b60408051918252519081900360200190f35b34801561008e57600080fd5b5061007073ffffffffffffffffffffffffffffffffffffffff600435166100fb565b3480156100bc57600080fd5b506100e173ffffffffffffffffffffffffffffffffffffffff60043516602435610123565b604080519115158252519081900360200190f35b60015490565b73ffffffffffffffffffffffffffffffffffffffff1660009081526020819052604090205490565b3360009081526020819052604081205482111561013f57600080fd5b73ffffffffffffffffffffffffffffffffffffffff8316151561016157600080fd5b33600090815260208190526040902054610181908363ffffffff61022916565b336000908152602081905260408082209290925573ffffffffffffffffffffffffffffffffffffffff8516815220546101c0908363ffffffff61023b16565b73ffffffffffffffffffffffffffffffffffffffff8416600081815260208181526040918290209390935580518581529051919233927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9281900390910190a350600192915050565b60008282111561023557fe5b50900390565b8181018281101561024857fe5b929150505600a165627a7a72305820a20abaa1c7f797936fb740943b6dd87579b4b9b26d232977f7dd8399b32c32060029"``

#### Defined in

packages/ethereum/src/types/factories/BasicToken__factory.ts:120

## Methods

### attach

▸ **attach**(`address`): [`BasicToken`](BasicToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`BasicToken`](BasicToken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/src/types/factories/BasicToken__factory.ts:114

___

### connect

▸ **connect**(`signer`): [`BasicToken__factory`](BasicToken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`BasicToken__factory`](BasicToken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/src/types/factories/BasicToken__factory.ts:117

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`BasicToken`](BasicToken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`BasicToken`](BasicToken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/src/types/factories/BasicToken__factory.ts:104

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

packages/ethereum/src/types/factories/BasicToken__factory.ts:109

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`BasicToken`](BasicToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`BasicToken`](BasicToken.md)

#### Defined in

packages/ethereum/src/types/factories/BasicToken__factory.ts:125

___

### createInterface

▸ `Static` **createInterface**(): `BasicTokenInterface`

#### Returns

`BasicTokenInterface`

#### Defined in

packages/ethereum/src/types/factories/BasicToken__factory.ts:122

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

node_modules/@ethersproject/contracts/lib/index.d.ts:146

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

node_modules/@ethersproject/contracts/lib/index.d.ts:152

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

node_modules/@ethersproject/contracts/lib/index.d.ts:148

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

node_modules/@ethersproject/contracts/lib/index.d.ts:147
