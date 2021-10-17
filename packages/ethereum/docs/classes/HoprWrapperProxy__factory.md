[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprWrapperProxy\_\_factory

# Class: HoprWrapperProxy\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`HoprWrapperProxy__factory`**

## Table of contents

### Constructors

- [constructor](HoprWrapperProxy__factory.md#constructor)

### Properties

- [bytecode](HoprWrapperProxy__factory.md#bytecode)
- [interface](HoprWrapperProxy__factory.md#interface)
- [signer](HoprWrapperProxy__factory.md#signer)
- [abi](HoprWrapperProxy__factory.md#abi)
- [bytecode](HoprWrapperProxy__factory.md#bytecode)

### Methods

- [attach](HoprWrapperProxy__factory.md#attach)
- [connect](HoprWrapperProxy__factory.md#connect)
- [deploy](HoprWrapperProxy__factory.md#deploy)
- [getDeployTransaction](HoprWrapperProxy__factory.md#getdeploytransaction)
- [connect](HoprWrapperProxy__factory.md#connect)
- [createInterface](HoprWrapperProxy__factory.md#createinterface)
- [fromSolidity](HoprWrapperProxy__factory.md#fromsolidity)
- [getContract](HoprWrapperProxy__factory.md#getcontract)
- [getContractAddress](HoprWrapperProxy__factory.md#getcontractaddress)
- [getInterface](HoprWrapperProxy__factory.md#getinterface)

## Constructors

### constructor

• **new HoprWrapperProxy__factory**(...`args`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `...args` | [contractInterface: ContractInterface, bytecode: BytesLike \| Object, signer?: Signer] \| [signer: Signer] |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:220

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

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `inputs`: `any`[] = []; `name`: `undefined` = "allowance"; `outputs`: `undefined` ; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "constructor" } \| { `anonymous`: `boolean` = false; `inputs`: { `indexed`: `boolean` = false; `internalType`: `string` = "address"; `name`: `string` = "from"; `type`: `string` = "address" }[] ; `name`: `string` = "FowardedFrom"; `outputs`: `undefined` ; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" } \| { `anonymous`: `undefined` = false; `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "\_from"; `type`: `string` = "address" }[] ; `name`: `string` = "onTokenTransfer"; `outputs`: { `internalType`: `string` = "bool"; `name`: `string` = ""; `type`: `string` = "bool" }[] ; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" })[]

#### Defined in

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:247

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405234801561001057600080fd5b506040516329965a1d60e01b815230600482018190527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b15801561008a57600080fd5b505af115801561009e573d6000803e3d6000fd5b50505050610bd6806100b16000396000f3fe608060405234801561001057600080fd5b50600436106100925760003560e01c8063a4c0ed3611610066578063a4c0ed361461012c578063b861fa9e1461014f578063e1eb13c11461016a578063e28f56f314610185578063fe26277b146101a057600080fd5b806223de2914610097578063013eb177146100ac57806316114acd146100e457806372581cc0146100f7575b600080fd5b6100aa6100a536600461093f565b6101bb565b005b6100c7731820a4b7618bde71dce8cdc73aab6c95905fad2481565b6040516001600160a01b0390911681526020015b60405180910390f35b6100aa6100f23660046109ea565b610332565b61011e7fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b6040519081526020016100db565b61013f61013a366004610a1b565b610429565b60405190151581526020016100db565b6100c773d057604a14982fe8d88c5fc25aac3267ea142a0881565b6100c773097707143e01318734535676cfe2e5cf8b656ae881565b6100c7735e1c4e7004b7411ba27dc354330fab31147dfef181565b6100c773d4fdec44db9d44b8f2b6d529620f9c0c7066a2c181565b3373d4fdec44db9d44b8f2b6d529620f9c0c7066a2c11461023c5760405162461bcd60e51b815260206004820152603060248201527f486f70725772617070657250726f78793a204f6e6c792061636365707420575860448201526f2427a8292faa27a5a2a7103a37b5b2b760811b60648201526084015b60405180910390fd5b6001600160a01b03861630146102b35760405162461bcd60e51b815260206004820152603660248201527f486f70725772617070657250726f78793a204d7573742062652073656e64696e6044820152756720746f6b656e7320746f20746869732070726f787960501b6064820152608401610233565b604080516001600160a01b0389168152602081018790527f7c6d66a12116d23472c9d07a15684954389e3cecd458a973e2d52340dcc40077910160405180910390a161032873d4fdec44db9d44b8f2b6d529620f9c0c7066a2c1735e1c4e7004b7411ba27dc354330fab31147dfef187610639565b5050505050505050565b6001600160a01b03811661038457604051735e1c4e7004b7411ba27dc354330fab31147dfef1904780156108fc02916000818181858888f19350505050158015610380573d6000803e3d6000fd5b5050565b6040516370a0823160e01b815230600482015261042690735e1c4e7004b7411ba27dc354330fab31147dfef1906001600160a01b038416906370a082319060240160206040518083038186803b1580156103dd57600080fd5b505afa1580156103f1573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906104159190610ae6565b6001600160a01b0384169190610639565b50565b60003373d057604a14982fe8d88c5fc25aac3267ea142a08146104a05760405162461bcd60e51b815260206004820152602960248201527f486f70725772617070657250726f78793a204f6e6c792061636365707420784860448201526827a829103a37b5b2b760b91b6064820152608401610233565b6001600160a01b038416735e1c4e7004b7411ba27dc354330fab31147dfef1146105325760405162461bcd60e51b815260206004820152603a60248201527f486f70725772617070657250726f78793a204f6e6c792061636365707420784860448201527f4f50522066726f6d20746865204173736f204d756c74695369670000000000006064820152608401610233565b6040805173097707143e01318734535676cfe2e5cf8b656ae88152602081018590527f136ea539e14badd3720f65f9ff6414b0e6291b05e7c191cc3f1c81fa9d2dd569910160405180910390a1604051630200057560e51b815273097707143e01318734535676cfe2e5cf8b656ae8600482015260248101849052606060448201526000606482015273d057604a14982fe8d88c5fc25aac3267ea142a0890634000aea090608401602060405180830381600087803b1580156105f457600080fd5b505af1158015610608573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061062c9190610aff565b50600190505b9392505050565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b17905261068b908490610690565b505050565b60006106e5826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564815250856001600160a01b03166107629092919063ffffffff16565b80519091501561068b57808060200190518101906107039190610aff565b61068b5760405162461bcd60e51b815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e6044820152691bdd081cdd58d8d9595960b21b6064820152608401610233565b60606107718484600085610779565b949350505050565b6060824710156107da5760405162461bcd60e51b815260206004820152602660248201527f416464726573733a20696e73756666696369656e742062616c616e636520666f6044820152651c8818d85b1b60d21b6064820152608401610233565b843b6108285760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e74726163740000006044820152606401610233565b600080866001600160a01b031685876040516108449190610b51565b60006040518083038185875af1925050503d8060008114610881576040519150601f19603f3d011682016040523d82523d6000602084013e610886565b606091505b50915091506108968282866108a1565b979650505050505050565b606083156108b0575081610632565b8251156108c05782518084602001fd5b8160405162461bcd60e51b81526004016102339190610b6d565b80356001600160a01b03811681146108f157600080fd5b919050565b60008083601f84011261090857600080fd5b50813567ffffffffffffffff81111561092057600080fd5b60208301915083602082850101111561093857600080fd5b9250929050565b60008060008060008060008060c0898b03121561095b57600080fd5b610964896108da565b975061097260208a016108da565b965061098060408a016108da565b955060608901359450608089013567ffffffffffffffff808211156109a457600080fd5b6109b08c838d016108f6565b909650945060a08b01359150808211156109c957600080fd5b506109d68b828c016108f6565b999c989b5096995094979396929594505050565b6000602082840312156109fc57600080fd5b610632826108da565b634e487b7160e01b600052604160045260246000fd5b600080600060608486031215610a3057600080fd5b610a39846108da565b925060208401359150604084013567ffffffffffffffff80821115610a5d57600080fd5b818601915086601f830112610a7157600080fd5b813581811115610a8357610a83610a05565b604051601f8201601f19908116603f01168101908382118183101715610aab57610aab610a05565b81604052828152896020848701011115610ac457600080fd5b8260208601602083013760006020848301015280955050505050509250925092565b600060208284031215610af857600080fd5b5051919050565b600060208284031215610b1157600080fd5b8151801515811461063257600080fd5b60005b83811015610b3c578181015183820152602001610b24565b83811115610b4b576000848401525b50505050565b60008251610b63818460208701610b21565b9190910192915050565b6020815260008251806020840152610b8c816040850160208701610b21565b601f01601f1916919091016040019291505056fea2646970667358221220a84e5eb25df899965e98f5ba819ec5f0b2cca6313ad3318e9c4d1b897ff53f5364736f6c63430008090033"``

#### Defined in

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:246

## Methods

### attach

▸ **attach**(`address`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:240

___

### connect

▸ **connect**(`signer`): [`HoprWrapperProxy__factory`](HoprWrapperProxy__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`HoprWrapperProxy__factory`](HoprWrapperProxy__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:243

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`HoprWrapperProxy`](HoprWrapperProxy.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`HoprWrapperProxy`](HoprWrapperProxy.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:230

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

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:235

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Defined in

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:251

___

### createInterface

▸ `Static` **createInterface**(): `HoprWrapperProxyInterface`

#### Returns

`HoprWrapperProxyInterface`

#### Defined in

packages/ethereum/types/factories/HoprWrapperProxy__factory.ts:248

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
