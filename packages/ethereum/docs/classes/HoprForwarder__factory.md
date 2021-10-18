[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprForwarder\_\_factory

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

• **new HoprForwarder__factory**(...`args`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `...args` | [contractInterface: ContractInterface, bytecode: BytesLike \| Object, signer?: Signer] \| [signer: Signer] |

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

▪ `Static` `Readonly` **abi**: ({ `inputs`: `any`[] = []; `name`: `undefined` = "allowance"; `outputs`: `undefined` ; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "constructor" } \| { `inputs`: `any`[] = []; `name`: `string` = "ERC1820\_REGISTRY"; `outputs`: { `internalType`: `string` = "contract IERC1820Registry"; `name`: `string` = ""; `type`: `string` = "address" }[] ; `stateMutability`: `string` = "view"; `type`: `string` = "function" } \| { `inputs`: { `internalType`: `string` = "address"; `name`: `string` = "token"; `type`: `string` = "address" }[] ; `name`: `string` = "recoverTokens"; `outputs`: `any`[] = []; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" })[] = `_abi`

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:151

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405234801561001057600080fd5b506040516329965a1d60e01b815230600482018190527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b15801561008a57600080fd5b505af115801561009e573d6000803e3d6000fd5b5050505061087e806100b16000396000f3fe608060405234801561001057600080fd5b50600436106100615760003560e01c806223de2914610066578063013eb1771461007b57806316114acd146100b35780631ba6bac2146100c65780632530b145146100e157806372581cc0146100fc575b600080fd5b6100796100743660046106c8565b610131565b005b610096731820a4b7618bde71dce8cdc73aab6c95905fad2481565b6040516001600160a01b0390911681526020015b60405180910390f35b6100796100c1366004610773565b6102c9565b61009673f5581dfefd8fb0e4aec526be659cfab1f8c781da81565b610096734f50ab4e931289344a57f2fe4bbd10546a6fdc1781565b6101237fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b6040519081526020016100aa565b3373f5581dfefd8fb0e4aec526be659cfab1f8c781da146101ab5760405162461bcd60e51b815260206004820152602960248201527f486f70724d696e746572577261707065723a204f6e6c7920616363657074204860448201526827a829103a37b5b2b760b91b60648201526084015b60405180910390fd5b6001600160a01b0387161561020e5760405162461bcd60e51b8152602060048201526024808201527f486f70724d696e746572577261707065723a204f6e6c792072656365697665206044820152631b5a5b9d60e21b60648201526084016101a2565b6001600160a01b038616301461028c5760405162461bcd60e51b815260206004820152603f60248201527f486f70724d696e746572577261707065723a204d7573742062652073656e646960448201527f6e6720746f6b656e7320746f20746865206d696e74657220777261707065720060648201526084016101a2565b6102bf73f5581dfefd8fb0e4aec526be659cfab1f8c781da734f50ab4e931289344a57f2fe4bbd10546a6fdc17876103c0565b5050505050505050565b6001600160a01b03811661031b57604051734f50ab4e931289344a57f2fe4bbd10546a6fdc17904780156108fc02916000818181858888f19350505050158015610317573d6000803e3d6000fd5b5050565b6040516370a0823160e01b81523060048201526103bd90734f50ab4e931289344a57f2fe4bbd10546a6fdc17906001600160a01b038416906370a082319060240160206040518083038186803b15801561037457600080fd5b505afa158015610388573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906103ac919061078e565b6001600160a01b03841691906103c0565b50565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b179052610412908490610417565b505050565b600061046c826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564815250856001600160a01b03166104e99092919063ffffffff16565b805190915015610412578080602001905181019061048a91906107a7565b6104125760405162461bcd60e51b815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e6044820152691bdd081cdd58d8d9595960b21b60648201526084016101a2565b60606104f88484600085610502565b90505b9392505050565b6060824710156105635760405162461bcd60e51b815260206004820152602660248201527f416464726573733a20696e73756666696369656e742062616c616e636520666f6044820152651c8818d85b1b60d21b60648201526084016101a2565b843b6105b15760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e747261637400000060448201526064016101a2565b600080866001600160a01b031685876040516105cd91906107f9565b60006040518083038185875af1925050503d806000811461060a576040519150601f19603f3d011682016040523d82523d6000602084013e61060f565b606091505b509150915061061f82828661062a565b979650505050505050565b606083156106395750816104fb565b8251156106495782518084602001fd5b8160405162461bcd60e51b81526004016101a29190610815565b80356001600160a01b038116811461067a57600080fd5b919050565b60008083601f84011261069157600080fd5b50813567ffffffffffffffff8111156106a957600080fd5b6020830191508360208285010111156106c157600080fd5b9250929050565b60008060008060008060008060c0898b0312156106e457600080fd5b6106ed89610663565b97506106fb60208a01610663565b965061070960408a01610663565b955060608901359450608089013567ffffffffffffffff8082111561072d57600080fd5b6107398c838d0161067f565b909650945060a08b013591508082111561075257600080fd5b5061075f8b828c0161067f565b999c989b5096995094979396929594505050565b60006020828403121561078557600080fd5b6104fb82610663565b6000602082840312156107a057600080fd5b5051919050565b6000602082840312156107b957600080fd5b815180151581146104fb57600080fd5b60005b838110156107e45781810151838201526020016107cc565b838111156107f3576000848401525b50505050565b6000825161080b8184602087016107c9565b9190910192915050565b60208152600082518060208401526108348160408501602087016107c9565b601f01601f1916919091016040019291505056fea2646970667358221220f81445de9565023b414d209c5281b3b1003efe726537e2fd29d71c23a751a73e64736f6c63430008090033"``

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:150

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

packages/ethereum/types/factories/HoprForwarder__factory.ts:144

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

packages/ethereum/types/factories/HoprForwarder__factory.ts:147

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

packages/ethereum/types/factories/HoprForwarder__factory.ts:134

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

packages/ethereum/types/factories/HoprForwarder__factory.ts:139

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

packages/ethereum/types/factories/HoprForwarder__factory.ts:155

___

### createInterface

▸ `Static` **createInterface**(): `HoprForwarderInterface`

#### Returns

`HoprForwarderInterface`

#### Defined in

packages/ethereum/types/factories/HoprForwarder__factory.ts:152

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
