[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / MintableToken\_\_factory

# Class: MintableToken\_\_factory

## Hierarchy

- `ContractFactory`

  ↳ **`MintableToken__factory`**

## Table of contents

### Constructors

- [constructor](MintableToken__factory.md#constructor)

### Properties

- [bytecode](MintableToken__factory.md#bytecode)
- [interface](MintableToken__factory.md#interface)
- [signer](MintableToken__factory.md#signer)
- [abi](MintableToken__factory.md#abi)
- [bytecode](MintableToken__factory.md#bytecode)

### Methods

- [attach](MintableToken__factory.md#attach)
- [connect](MintableToken__factory.md#connect)
- [deploy](MintableToken__factory.md#deploy)
- [getDeployTransaction](MintableToken__factory.md#getdeploytransaction)
- [connect](MintableToken__factory.md#connect)
- [createInterface](MintableToken__factory.md#createinterface)
- [fromSolidity](MintableToken__factory.md#fromsolidity)
- [getContract](MintableToken__factory.md#getcontract)
- [getContractAddress](MintableToken__factory.md#getcontractaddress)
- [getInterface](MintableToken__factory.md#getinterface)

## Constructors

### constructor

• **new MintableToken__factory**(...`args`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `...args` | [contractInterface: ContractInterface, bytecode: BytesLike \| Object, signer?: Signer] \| [signer: Signer] |

#### Overrides

ContractFactory.constructor

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:375

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

▪ `Static` `Readonly` **abi**: ({ `anonymous`: `undefined` = false; `constant`: `boolean` = false; `inputs`: { `name`: `string` = "\_spender"; `type`: `string` = "address" }[] ; `name`: `string` = "approve"; `outputs`: { `name`: `string` = ""; `type`: `string` = "bool" }[] ; `payable`: `boolean` = false; `stateMutability`: `string` = "nonpayable"; `type`: `string` = "function" } \| { `anonymous`: `boolean` = false; `constant`: `undefined` = true; `inputs`: { `indexed`: `boolean` = true; `name`: `string` = "to"; `type`: `string` = "address" }[] ; `name`: `string` = "Mint"; `outputs`: `undefined` ; `payable`: `undefined` = false; `stateMutability`: `undefined` = "view"; `type`: `string` = "event" })[] = `_abi`

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:402

___

### bytecode

▪ `Static` `Readonly` **bytecode**: ``"0x608060405260038054600160a860020a03191633179055610aa7806100256000396000f3006080604052600436106100cf5763ffffffff7c010000000000000000000000000000000000000000000000000000000060003504166305d2035b81146100d4578063095ea7b3146100fd57806318160ddd1461012157806323b872dd1461014857806340c10f1914610172578063661884631461019657806370a08231146101ba578063715018a6146101db5780637d64bcb4146101f25780638da5cb5b14610207578063a9059cbb14610238578063d73dd6231461025c578063dd62ed3e14610280578063f2fde38b146102a7575b600080fd5b3480156100e057600080fd5b506100e96102c8565b604080519115158252519081900360200190f35b34801561010957600080fd5b506100e9600160a060020a03600435166024356102e9565b34801561012d57600080fd5b5061013661034f565b60408051918252519081900360200190f35b34801561015457600080fd5b506100e9600160a060020a0360043581169060243516604435610355565b34801561017e57600080fd5b506100e9600160a060020a03600435166024356104ca565b3480156101a257600080fd5b506100e9600160a060020a03600435166024356105e5565b3480156101c657600080fd5b50610136600160a060020a03600435166106d4565b3480156101e757600080fd5b506101f06106ef565b005b3480156101fe57600080fd5b506100e961075d565b34801561021357600080fd5b5061021c610803565b60408051600160a060020a039092168252519081900360200190f35b34801561024457600080fd5b506100e9600160a060020a0360043516602435610812565b34801561026857600080fd5b506100e9600160a060020a03600435166024356108f1565b34801561028c57600080fd5b50610136600160a060020a036004358116906024351661098a565b3480156102b357600080fd5b506101f0600160a060020a03600435166109b5565b60035474010000000000000000000000000000000000000000900460ff1681565b336000818152600260209081526040808320600160a060020a038716808552908352818420869055815186815291519394909390927f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925928290030190a350600192915050565b60015490565b600160a060020a03831660009081526020819052604081205482111561037a57600080fd5b600160a060020a03841660009081526002602090815260408083203384529091529020548211156103aa57600080fd5b600160a060020a03831615156103bf57600080fd5b600160a060020a0384166000908152602081905260409020546103e8908363ffffffff6109d816565b600160a060020a03808616600090815260208190526040808220939093559085168152205461041d908363ffffffff6109ea16565b600160a060020a0380851660009081526020818152604080832094909455918716815260028252828120338252909152205461045f908363ffffffff6109d816565b600160a060020a03808616600081815260026020908152604080832033845282529182902094909455805186815290519287169391927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef929181900390910190a35060019392505050565b600354600090600160a060020a031633146104e457600080fd5b60035474010000000000000000000000000000000000000000900460ff161561050c57600080fd5b60015461051f908363ffffffff6109ea16565b600155600160a060020a03831660009081526020819052604090205461054b908363ffffffff6109ea16565b600160a060020a03841660008181526020818152604091829020939093558051858152905191927f0f6798a560793a54c3bcfe86a93cde1e73087d944c0ea20544137d412139688592918290030190a2604080518381529051600160a060020a038516916000917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9181900360200190a350600192915050565b336000908152600260209081526040808320600160a060020a038616845290915281205480831061063957336000908152600260209081526040808320600160a060020a038816845290915281205561066e565b610649818463ffffffff6109d816565b336000908152600260209081526040808320600160a060020a03891684529091529020555b336000818152600260209081526040808320600160a060020a0389168085529083529281902054815190815290519293927f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925929181900390910190a35060019392505050565b600160a060020a031660009081526020819052604090205490565b600354600160a060020a0316331461070657600080fd5b600354604051600160a060020a03909116907ff8df31144d9c2f0f6b59d69b8b98abd5459d07f2742c4df920b25aae33c6482090600090a26003805473ffffffffffffffffffffffffffffffffffffffff19169055565b600354600090600160a060020a0316331461077757600080fd5b60035474010000000000000000000000000000000000000000900460ff161561079f57600080fd5b6003805474ff00000000000000000000000000000000000000001916740100000000000000000000000000000000000000001790556040517fae5184fba832cb2b1f702aca6117b8d265eaf03ad33eb133f19dde0f5920fa0890600090a150600190565b600354600160a060020a031681565b3360009081526020819052604081205482111561082e57600080fd5b600160a060020a038316151561084357600080fd5b33600090815260208190526040902054610863908363ffffffff6109d816565b3360009081526020819052604080822092909255600160a060020a03851681522054610895908363ffffffff6109ea16565b600160a060020a038416600081815260208181526040918290209390935580518581529051919233927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9281900390910190a350600192915050565b336000908152600260209081526040808320600160a060020a0386168452909152812054610925908363ffffffff6109ea16565b336000818152600260209081526040808320600160a060020a0389168085529083529281902085905580519485525191937f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925929081900390910190a350600192915050565b600160a060020a03918216600090815260026020908152604080832093909416825291909152205490565b600354600160a060020a031633146109cc57600080fd5b6109d5816109fd565b50565b6000828211156109e457fe5b50900390565b818101828110156109f757fe5b92915050565b600160a060020a0381161515610a1257600080fd5b600354604051600160a060020a038084169216907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a36003805473ffffffffffffffffffffffffffffffffffffffff1916600160a060020a03929092169190911790555600a165627a7a72305820b07f865cfd51072121caccd01c1903f1fe1fe5ee11a1f11776f0eb8357e133b90029"``

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:401

## Methods

### attach

▸ **attach**(`address`): [`MintableToken`](MintableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |

#### Returns

[`MintableToken`](MintableToken.md)

#### Overrides

ContractFactory.attach

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:395

___

### connect

▸ **connect**(`signer`): [`MintableToken__factory`](MintableToken__factory.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signer` | `Signer` |

#### Returns

[`MintableToken__factory`](MintableToken__factory.md)

#### Overrides

ContractFactory.connect

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:398

___

### deploy

▸ **deploy**(`overrides?`): `Promise`<[`MintableToken`](MintableToken.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<[`MintableToken`](MintableToken.md)\>

#### Overrides

ContractFactory.deploy

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:385

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

packages/ethereum/types/factories/MintableToken__factory.ts:390

___

### connect

▸ `Static` **connect**(`address`, `signerOrProvider`): [`MintableToken`](MintableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `string` |
| `signerOrProvider` | `Signer` \| `Provider` |

#### Returns

[`MintableToken`](MintableToken.md)

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:406

___

### createInterface

▸ `Static` **createInterface**(): `MintableTokenInterface`

#### Returns

`MintableTokenInterface`

#### Defined in

packages/ethereum/types/factories/MintableToken__factory.ts:403

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
