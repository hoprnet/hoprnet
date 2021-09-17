[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprForwarder__factory

# Class: HoprForwarder\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`HoprForwarder__factory`**

## Table of contents

### Constructors

- [constructor](HoprForwarder__factory.md#constructor)

### Properties

- [bytecode](HoprForwarder__factory.md#bytecode)
- [interface](HoprForwarder__factory.md#interface)
- [signer](HoprForwarder__factory.md#signer)
- [abi](HoprForwarder__factory.md#abi)
- [bytecode](HoprForwarder__factory.md#bytecode)

### Methods

- [attach](HoprForwarder__factory.md#attach)
- [connect](HoprForwarder__factory.md#connect)
- [deploy](HoprForwarder__factory.md#deploy)
- [getDeployTransaction](HoprForwarder__factory.md#getdeploytransaction)
- [connect](HoprForwarder__factory.md#connect)
- [createInterface](HoprForwarder__factory.md#createinterface)
- [fromSolidity](HoprForwarder__factory.md#fromsolidity)
- [getContract](HoprForwarder__factory.md#getcontract)
- [getContractAddress](HoprForwarder__factory.md#getcontractaddress)
- [getInterface](HoprForwarder__factory.md#getinterface)

## Constructors

### constructor

• **new HoprForwarder__factory**(`signer?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer?` | `Signer` |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:124

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

▪ `Static` `Readonly` **abi**: ({ `inputs`: `any`[] = []; `name`: `undefined` = "allowance"; `outputs`: `undefined` ; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "constructor" } \| { `inputs`: `any`[] = []; `name`: `string` = "ERC1820\_REGISTRY"; `outputs`: { `internalType`: `string` = "contract IERC1820Registry"; `name`: `string` = ""; `type`: `string` = "address" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" } \| { `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "token"; `type`: `string` = "address" }[] ; `name`: `string` = "recoverTokens"; `outputs`: `any`[] = []; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" })[]

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:145

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405234801561001057600080fd5b506040516329965a1d60e01b815230600482018190527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b15801561008a57600080fd5b505af115801561009e573d6000803e3d6000fd5b50505050610883806100b16000396000f3fe608060405234801561001057600080fd5b50600436106100615760003560e01c806223de2914610066578063013eb1771461007b57806316114acd146100b35780631ba6bac2146100c65780632530b145146100e157806372581cc0146100fc575b600080fd5b6100796100743660046106ee565b610131565b005b610096731820a4b7618bde71dce8cdc73aab6c95905fad2481565b6040516001600160a01b0390911681526020015b60405180910390f35b6100796100c13660046106d4565b6102c9565b61009673f5581dfefd8fb0e4aec526be659cfab1f8c781da81565b610096734f50ab4e931289344a57f2fe4bbd10546a6fdc1781565b6101237fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b6040519081526020016100aa565b3373f5581dfefd8fb0e4aec526be659cfab1f8c781da146101ab5760405162461bcd60e51b815260206004820152602960248201527f486f70724d696e746572577261707065723a204f6e6c7920616363657074204860448201526827a829103a37b5b2b760b91b60648201526084015b60405180910390fd5b6001600160a01b0387161561020e5760405162461bcd60e51b8152602060048201526024808201527f486f70724d696e746572577261707065723a204f6e6c792072656365697665206044820152631b5a5b9d60e21b60648201526084016101a2565b6001600160a01b038616301461028c5760405162461bcd60e51b815260206004820152603f60248201527f486f70724d696e746572577261707065723a204d7573742062652073656e646960448201527f6e6720746f6b656e7320746f20746865206d696e74657220777261707065720060648201526084016101a2565b6102bf73f5581dfefd8fb0e4aec526be659cfab1f8c781da734f50ab4e931289344a57f2fe4bbd10546a6fdc17876103c2565b5050505050505050565b6001600160a01b03811661031d57604051734f50ab4e931289344a57f2fe4bbd10546a6fdc17904780156108fc02916000818181858888f19350505050158015610317573d6000803e3d6000fd5b506103bf565b6040516370a0823160e01b81523060048201526103bf90734f50ab4e931289344a57f2fe4bbd10546a6fdc17906001600160a01b038416906370a082319060240160206040518083038186803b15801561037657600080fd5b505afa15801561038a573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906103ae91906107b6565b6001600160a01b03841691906103c2565b50565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b179052610414908490610419565b505050565b600061046e826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564815250856001600160a01b03166104eb9092919063ffffffff16565b805190915015610414578080602001905181019061048c9190610796565b6104145760405162461bcd60e51b815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e6044820152691bdd081cdd58d8d9595960b21b60648201526084016101a2565b60606104fa8484600085610504565b90505b9392505050565b6060824710156105655760405162461bcd60e51b815260206004820152602660248201527f416464726573733a20696e73756666696369656e742062616c616e636520666f6044820152651c8818d85b1b60d21b60648201526084016101a2565b61056e85610633565b6105ba5760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e747261637400000060448201526064016101a2565b600080866001600160a01b031685876040516105d691906107ce565b60006040518083038185875af1925050503d8060008114610613576040519150601f19603f3d011682016040523d82523d6000602084013e610618565b606091505b509150915061062882828661063d565b979650505050505050565b803b15155b919050565b6060831561064c5750816104fd565b82511561065c5782518084602001fd5b8160405162461bcd60e51b81526004016101a291906107ea565b80356001600160a01b038116811461063857600080fd5b60008083601f84011261069e578182fd5b50813567ffffffffffffffff8111156106b5578182fd5b6020830191508360208285010111156106cd57600080fd5b9250929050565b6000602082840312156106e5578081fd5b6104fd82610676565b60008060008060008060008060c0898b031215610709578384fd5b61071289610676565b975061072060208a01610676565b965061072e60408a01610676565b955060608901359450608089013567ffffffffffffffff80821115610751578586fd5b61075d8c838d0161068d565b909650945060a08b0135915080821115610775578384fd5b506107828b828c0161068d565b999c989b5096995094979396929594505050565b6000602082840312156107a7578081fd5b815180151581146104fd578182fd5b6000602082840312156107c7578081fd5b5051919050565b600082516107e081846020870161081d565b9190910192915050565b600060208252825180602084015261080981604085016020870161081d565b601f01601f19169190910160400192915050565b60005b83811015610838578181015183820152602001610820565b83811115610847576000848401525b5050505056fea26469706673582212206c3b5017aa66da4aa944df6cc9bd86f08d272f7a9a26b97d510048c93c8b427364736f6c63430008030033"``

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:144

## Methods

### attach

▸ **attach**(`address`): [`HoprForwarder`](HoprForwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`HoprForwarder`](HoprForwarder.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:138

___

### connect

▸ **connect**(`signer`): [`HoprForwarder__factory`](HoprForwarder__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`HoprForwarder__factory`](HoprForwarder__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:141

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`HoprForwarder`](HoprForwarder.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`HoprForwarder`](HoprForwarder.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:128

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

packages/ethereum/types/factories/HoprForwarder__factory.ts:133

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`HoprForwarder`](HoprForwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`HoprForwarder`](HoprForwarder.md)

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:149

___

### createInterface

▸ `Static` **createInterface**(): `HoprForwarderInterface`

#### Returns

`HoprForwarderInterface`

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:146

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
