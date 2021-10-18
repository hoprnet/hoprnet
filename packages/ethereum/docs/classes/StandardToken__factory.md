[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / StandardToken\_\_factory

# Class: StandardToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`StandardToken__factory`**

## Table of contents

### Constructors

- [constructor](StandardToken__factory.md#constructor)

### Properties

- [bytecode](StandardToken__factory.md#bytecode)
- [interface](StandardToken__factory.md#interface)
- [signer](StandardToken__factory.md#signer)
- [abi](StandardToken__factory.md#abi)
- [bytecode](StandardToken__factory.md#bytecode)

### Methods

- [attach](StandardToken__factory.md#attach)
- [connect](StandardToken__factory.md#connect)
- [deploy](StandardToken__factory.md#deploy)
- [getDeployTransaction](StandardToken__factory.md#getdeploytransaction)
- [connect](StandardToken__factory.md#connect)
- [createInterface](StandardToken__factory.md#createinterface)
- [fromSolidity](StandardToken__factory.md#fromsolidity)
- [getContract](StandardToken__factory.md#getcontract)
- [getContractAddress](StandardToken__factory.md#getcontractaddress)
- [getInterface](StandardToken__factory.md#getinterface)

## Constructors

### constructor

• **new StandardToken__factory**(...`args`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `...args` | [contractInterface: ContractInterface, bytecode: BytesLike \| Object, signer?: Signer] \| [signer: Signer] |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/StandardToken__factory.ts:235

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

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = false; `inputs`: { `name`: `string` = "\_spender"; `type`: `string` = "address" }[] ; `name`: `string` = "approve"; `outputs`: { `name`: `string` = ""; `type`: `string` = "bool" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "owner"; `type`: `string` = "address" }[] ; `name`: `string` = "Approval"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[] = `_abi`

#### Defined in

packages/ethereum/types/factories/StandardToken__factory.ts:262

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405234801561001057600080fd5b506106ae806100206000396000f30060806040526004361061008d5763ffffffff7c0100000000000000000000000000000000000000000000000000000000600035041663095ea7b3811461009257806318160ddd146100ca57806323b872dd146100f1578063661884631461011b57806370a082311461013f578063a9059cbb14610160578063d73dd62314610184578063dd62ed3e146101a8575b600080fd5b34801561009e57600080fd5b506100b6600160a060020a03600435166024356101cf565b604080519115158252519081900360200190f35b3480156100d657600080fd5b506100df610235565b60408051918252519081900360200190f35b3480156100fd57600080fd5b506100b6600160a060020a036004358116906024351660443561023b565b34801561012757600080fd5b506100b6600160a060020a03600435166024356103b0565b34801561014b57600080fd5b506100df600160a060020a036004351661049f565b34801561016c57600080fd5b506100b6600160a060020a03600435166024356104ba565b34801561019057600080fd5b506100b6600160a060020a0360043516602435610599565b3480156101b457600080fd5b506100df600160a060020a0360043581169060243516610632565b336000818152600260209081526040808320600160a060020a038716808552908352818420869055815186815291519394909390927f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925928290030190a350600192915050565b60015490565b600160a060020a03831660009081526020819052604081205482111561026057600080fd5b600160a060020a038416600090815260026020908152604080832033845290915290205482111561029057600080fd5b600160a060020a03831615156102a557600080fd5b600160a060020a0384166000908152602081905260409020546102ce908363ffffffff61065d16565b600160a060020a038086166000908152602081905260408082209390935590851681522054610303908363ffffffff61066f16565b600160a060020a03808516600090815260208181526040808320949094559187168152600282528281203382529091522054610345908363ffffffff61065d16565b600160a060020a03808616600081815260026020908152604080832033845282529182902094909455805186815290519287169391927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef929181900390910190a35060019392505050565b336000908152600260209081526040808320600160a060020a038616845290915281205480831061040457336000908152600260209081526040808320600160a060020a0388168452909152812055610439565b610414818463ffffffff61065d16565b336000908152600260209081526040808320600160a060020a03891684529091529020555b336000818152600260209081526040808320600160a060020a0389168085529083529281902054815190815290519293927f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925929181900390910190a35060019392505050565b600160a060020a031660009081526020819052604090205490565b336000908152602081905260408120548211156104d657600080fd5b600160a060020a03831615156104eb57600080fd5b3360009081526020819052604090205461050b908363ffffffff61065d16565b3360009081526020819052604080822092909255600160a060020a0385168152205461053d908363ffffffff61066f16565b600160a060020a038416600081815260208181526040918290209390935580518581529051919233927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9281900390910190a350600192915050565b336000908152600260209081526040808320600160a060020a03861684529091528120546105cd908363ffffffff61066f16565b336000818152600260209081526040808320600160a060020a0389168085529083529281902085905580519485525191937f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925929081900390910190a350600192915050565b600160a060020a03918216600090815260026020908152604080832093909416825291909152205490565b60008282111561066957fe5b50900390565b8181018281101561067c57fe5b929150505600a165627a7a723058209bc6c644fde35dd619ca1fe1c3a1e8e64cccfc4a26e6400d319b0ce06620ce2e0029"``

#### Defined in

packages/ethereum/types/factories/StandardToken__factory.ts:261

## Methods

### attach

▸ **attach**(`address`): [`StandardToken`](StandardToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`StandardToken`](StandardToken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/StandardToken__factory.ts:255

___

### connect

▸ **connect**(`signer`): [`StandardToken__factory`](StandardToken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`StandardToken__factory`](StandardToken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/StandardToken__factory.ts:258

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`StandardToken`](StandardToken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`StandardToken`](StandardToken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/StandardToken__factory.ts:245

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

packages/ethereum/types/factories/StandardToken__factory.ts:250

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`StandardToken`](StandardToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`StandardToken`](StandardToken.md)

#### Defined in

packages/ethereum/types/factories/StandardToken__factory.ts:266

___

### createInterface

▸ `Static` **createInterface**(): `StandardTokenInterface`

#### Returns

`StandardTokenInterface`

#### Defined in

packages/ethereum/types/factories/StandardToken__factory.ts:263

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
