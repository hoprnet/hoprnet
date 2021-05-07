[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprToken](../modules/contracts_hoprtoken.md) / HoprTokenInterface

# Interface: HoprTokenInterface

[contracts/HoprToken](../modules/contracts_hoprtoken.md).HoprTokenInterface

## Hierarchy

- *Interface*

  ↳ **HoprTokenInterface**

## Table of contents

### Properties

- [\_abiCoder](contracts_hoprtoken.hoprtokeninterface.md#_abicoder)
- [\_isInterface](contracts_hoprtoken.hoprtokeninterface.md#_isinterface)
- [deploy](contracts_hoprtoken.hoprtokeninterface.md#deploy)
- [errors](contracts_hoprtoken.hoprtokeninterface.md#errors)
- [events](contracts_hoprtoken.hoprtokeninterface.md#events)
- [fragments](contracts_hoprtoken.hoprtokeninterface.md#fragments)
- [functions](contracts_hoprtoken.hoprtokeninterface.md#functions)
- [structs](contracts_hoprtoken.hoprtokeninterface.md#structs)

### Methods

- [\_decodeParams](contracts_hoprtoken.hoprtokeninterface.md#_decodeparams)
- [\_encodeParams](contracts_hoprtoken.hoprtokeninterface.md#_encodeparams)
- [decodeEventLog](contracts_hoprtoken.hoprtokeninterface.md#decodeeventlog)
- [decodeFunctionData](contracts_hoprtoken.hoprtokeninterface.md#decodefunctiondata)
- [decodeFunctionResult](contracts_hoprtoken.hoprtokeninterface.md#decodefunctionresult)
- [encodeDeploy](contracts_hoprtoken.hoprtokeninterface.md#encodedeploy)
- [encodeEventLog](contracts_hoprtoken.hoprtokeninterface.md#encodeeventlog)
- [encodeFilterTopics](contracts_hoprtoken.hoprtokeninterface.md#encodefiltertopics)
- [encodeFunctionData](contracts_hoprtoken.hoprtokeninterface.md#encodefunctiondata)
- [encodeFunctionResult](contracts_hoprtoken.hoprtokeninterface.md#encodefunctionresult)
- [format](contracts_hoprtoken.hoprtokeninterface.md#format)
- [getEvent](contracts_hoprtoken.hoprtokeninterface.md#getevent)
- [getEventTopic](contracts_hoprtoken.hoprtokeninterface.md#geteventtopic)
- [getFunction](contracts_hoprtoken.hoprtokeninterface.md#getfunction)
- [getSighash](contracts_hoprtoken.hoprtokeninterface.md#getsighash)
- [parseLog](contracts_hoprtoken.hoprtokeninterface.md#parselog)
- [parseTransaction](contracts_hoprtoken.hoprtokeninterface.md#parsetransaction)

## Properties

### \_abiCoder

• `Readonly` **\_abiCoder**: *AbiCoder*

Inherited from: ethers.utils.Interface.\_abiCoder

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:43

___

### \_isInterface

• `Readonly` **\_isInterface**: *boolean*

Inherited from: ethers.utils.Interface.\_isInterface

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:44

___

### deploy

• `Readonly` **deploy**: *ConstructorFragment*

Inherited from: ethers.utils.Interface.deploy

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:42

___

### errors

• `Readonly` **errors**: *object*

#### Type declaration

Inherited from: ethers.utils.Interface.errors

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:30

___

### events

• **events**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval(address,address,uint256)` | *EventFragment* |
| `AuthorizedOperator(address,address)` | *EventFragment* |
| `Burned(address,address,uint256,bytes,bytes)` | *EventFragment* |
| `Minted(address,address,uint256,bytes,bytes)` | *EventFragment* |
| `RevokedOperator(address,address)` | *EventFragment* |
| `RoleGranted(bytes32,address,address)` | *EventFragment* |
| `RoleRevoked(bytes32,address,address)` | *EventFragment* |
| `Sent(address,address,address,uint256,bytes,bytes)` | *EventFragment* |
| `Transfer(address,address,uint256)` | *EventFragment* |

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:265

___

### fragments

• `Readonly` **fragments**: readonly *Fragment*[]

Inherited from: ethers.utils.Interface.fragments

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:29

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE()` | *FunctionFragment* |
| `MINTER_ROLE()` | *FunctionFragment* |
| `accountSnapshots(address,uint256)` | *FunctionFragment* |
| `allowance(address,address)` | *FunctionFragment* |
| `approve(address,uint256)` | *FunctionFragment* |
| `authorizeOperator(address)` | *FunctionFragment* |
| `balanceOf(address)` | *FunctionFragment* |
| `balanceOfAt(address,uint128)` | *FunctionFragment* |
| `burn(uint256,bytes)` | *FunctionFragment* |
| `decimals()` | *FunctionFragment* |
| `defaultOperators()` | *FunctionFragment* |
| `getRoleAdmin(bytes32)` | *FunctionFragment* |
| `getRoleMember(bytes32,uint256)` | *FunctionFragment* |
| `getRoleMemberCount(bytes32)` | *FunctionFragment* |
| `grantRole(bytes32,address)` | *FunctionFragment* |
| `granularity()` | *FunctionFragment* |
| `hasRole(bytes32,address)` | *FunctionFragment* |
| `isOperatorFor(address,address)` | *FunctionFragment* |
| `mint(address,uint256,bytes,bytes)` | *FunctionFragment* |
| `name()` | *FunctionFragment* |
| `operatorBurn(address,uint256,bytes,bytes)` | *FunctionFragment* |
| `operatorSend(address,address,uint256,bytes,bytes)` | *FunctionFragment* |
| `renounceRole(bytes32,address)` | *FunctionFragment* |
| `revokeOperator(address)` | *FunctionFragment* |
| `revokeRole(bytes32,address)` | *FunctionFragment* |
| `send(address,uint256,bytes)` | *FunctionFragment* |
| `symbol()` | *FunctionFragment* |
| `totalSupply()` | *FunctionFragment* |
| `totalSupplyAt(uint128)` | *FunctionFragment* |
| `totalSupplySnapshots(uint256)` | *FunctionFragment* |
| `transfer(address,uint256)` | *FunctionFragment* |
| `transferFrom(address,address,uint256)` | *FunctionFragment* |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:23

___

### structs

• `Readonly` **structs**: *object*

#### Type declaration

Inherited from: ethers.utils.Interface.structs

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:39

## Methods

### \_decodeParams

▸ **_decodeParams**(`params`: readonly *ParamType*[], `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `params` | readonly *ParamType*[] |
| `data` | BytesLike |

**Returns:** *Result*

Inherited from: ethers.utils.Interface.\_decodeParams

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:55

___

### \_encodeParams

▸ **_encodeParams**(`params`: readonly *ParamType*[], `values`: readonly *any*[]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `params` | readonly *ParamType*[] |
| `values` | readonly *any*[] |

**Returns:** *string*

Inherited from: ethers.utils.Interface.\_encodeParams

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:56

___

### decodeEventLog

▸ **decodeEventLog**(`eventFragment`: *string* \| *EventFragment*, `data`: BytesLike, `topics?`: readonly *string*[]): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFragment` | *string* \| *EventFragment* |
| `data` | BytesLike |
| `topics?` | readonly *string*[] |

**Returns:** *Result*

Inherited from: ethers.utils.Interface.decodeEventLog

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:67

___

### decodeFunctionData

▸ **decodeFunctionData**(`functionFragment`: *string* \| *FunctionFragment*, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | *string* \| *FunctionFragment* |
| `data` | BytesLike |

**Returns:** *Result*

Inherited from: ethers.utils.Interface.decodeFunctionData

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:58

___

### decodeFunctionResult

▸ **decodeFunctionResult**(`functionFragment`: ``"DEFAULT_ADMIN_ROLE"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"DEFAULT_ADMIN_ROLE"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:175

▸ **decodeFunctionResult**(`functionFragment`: ``"MINTER_ROLE"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"MINTER_ROLE"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:179

▸ **decodeFunctionResult**(`functionFragment`: ``"accountSnapshots"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"accountSnapshots"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:183

▸ **decodeFunctionResult**(`functionFragment`: ``"allowance"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"allowance"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:187

▸ **decodeFunctionResult**(`functionFragment`: ``"approve"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"approve"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:188

▸ **decodeFunctionResult**(`functionFragment`: ``"authorizeOperator"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"authorizeOperator"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:189

▸ **decodeFunctionResult**(`functionFragment`: ``"balanceOf"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"balanceOf"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:193

▸ **decodeFunctionResult**(`functionFragment`: ``"balanceOfAt"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"balanceOfAt"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:194

▸ **decodeFunctionResult**(`functionFragment`: ``"burn"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"burn"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:198

▸ **decodeFunctionResult**(`functionFragment`: ``"decimals"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"decimals"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:199

▸ **decodeFunctionResult**(`functionFragment`: ``"defaultOperators"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"defaultOperators"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:200

▸ **decodeFunctionResult**(`functionFragment`: ``"getRoleAdmin"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getRoleAdmin"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:204

▸ **decodeFunctionResult**(`functionFragment`: ``"getRoleMember"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getRoleMember"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:208

▸ **decodeFunctionResult**(`functionFragment`: ``"getRoleMemberCount"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getRoleMemberCount"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:212

▸ **decodeFunctionResult**(`functionFragment`: ``"grantRole"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"grantRole"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:216

▸ **decodeFunctionResult**(`functionFragment`: ``"granularity"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"granularity"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:217

▸ **decodeFunctionResult**(`functionFragment`: ``"hasRole"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"hasRole"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:221

▸ **decodeFunctionResult**(`functionFragment`: ``"isOperatorFor"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"isOperatorFor"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:222

▸ **decodeFunctionResult**(`functionFragment`: ``"mint"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"mint"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:226

▸ **decodeFunctionResult**(`functionFragment`: ``"name"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"name"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:227

▸ **decodeFunctionResult**(`functionFragment`: ``"operatorBurn"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"operatorBurn"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:228

▸ **decodeFunctionResult**(`functionFragment`: ``"operatorSend"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"operatorSend"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:232

▸ **decodeFunctionResult**(`functionFragment`: ``"renounceRole"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"renounceRole"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:236

▸ **decodeFunctionResult**(`functionFragment`: ``"revokeOperator"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"revokeOperator"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:240

▸ **decodeFunctionResult**(`functionFragment`: ``"revokeRole"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"revokeRole"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:244

▸ **decodeFunctionResult**(`functionFragment`: ``"send"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"send"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:245

▸ **decodeFunctionResult**(`functionFragment`: ``"symbol"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"symbol"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:246

▸ **decodeFunctionResult**(`functionFragment`: ``"totalSupply"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalSupply"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:247

▸ **decodeFunctionResult**(`functionFragment`: ``"totalSupplyAt"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalSupplyAt"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:251

▸ **decodeFunctionResult**(`functionFragment`: ``"totalSupplySnapshots"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalSupplySnapshots"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:255

▸ **decodeFunctionResult**(`functionFragment`: ``"transfer"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"transfer"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:259

▸ **decodeFunctionResult**(`functionFragment`: ``"transferFrom"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"transferFrom"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:260

___

### encodeDeploy

▸ **encodeDeploy**(`values?`: readonly *any*[]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `values?` | readonly *any*[] |

**Returns:** *string*

Inherited from: ethers.utils.Interface.encodeDeploy

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:57

___

### encodeEventLog

▸ **encodeEventLog**(`eventFragment`: *EventFragment*, `values`: readonly *any*[]): *object*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFragment` | *EventFragment* |
| `values` | readonly *any*[] |

**Returns:** *object*

| Name | Type |
| :------ | :------ |
| `data` | *string* |
| `topics` | *string*[] |

Inherited from: ethers.utils.Interface.encodeEventLog

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:63

___

### encodeFilterTopics

▸ **encodeFilterTopics**(`eventFragment`: *EventFragment*, `values`: readonly *any*[]): (*string* \| *string*[])[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFragment` | *EventFragment* |
| `values` | readonly *any*[] |

**Returns:** (*string* \| *string*[])[]

Inherited from: ethers.utils.Interface.encodeFilterTopics

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:62

___

### encodeFunctionData

▸ **encodeFunctionData**(`functionFragment`: ``"DEFAULT_ADMIN_ROLE"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"DEFAULT_ADMIN_ROLE"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:58

▸ **encodeFunctionData**(`functionFragment`: ``"MINTER_ROLE"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"MINTER_ROLE"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:62

▸ **encodeFunctionData**(`functionFragment`: ``"accountSnapshots"``, `values`: [*string*, BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"accountSnapshots"`` |
| `values` | [*string*, BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:66

▸ **encodeFunctionData**(`functionFragment`: ``"allowance"``, `values`: [*string*, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"allowance"`` |
| `values` | [*string*, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:70

▸ **encodeFunctionData**(`functionFragment`: ``"approve"``, `values`: [*string*, BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"approve"`` |
| `values` | [*string*, BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:74

▸ **encodeFunctionData**(`functionFragment`: ``"authorizeOperator"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"authorizeOperator"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:78

▸ **encodeFunctionData**(`functionFragment`: ``"balanceOf"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"balanceOf"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:82

▸ **encodeFunctionData**(`functionFragment`: ``"balanceOfAt"``, `values`: [*string*, BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"balanceOfAt"`` |
| `values` | [*string*, BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:83

▸ **encodeFunctionData**(`functionFragment`: ``"burn"``, `values`: [BigNumberish, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"burn"`` |
| `values` | [BigNumberish, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:87

▸ **encodeFunctionData**(`functionFragment`: ``"decimals"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"decimals"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:91

▸ **encodeFunctionData**(`functionFragment`: ``"defaultOperators"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"defaultOperators"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:92

▸ **encodeFunctionData**(`functionFragment`: ``"getRoleAdmin"``, `values`: [BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getRoleAdmin"`` |
| `values` | [BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:96

▸ **encodeFunctionData**(`functionFragment`: ``"getRoleMember"``, `values`: [BytesLike, BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getRoleMember"`` |
| `values` | [BytesLike, BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:100

▸ **encodeFunctionData**(`functionFragment`: ``"getRoleMemberCount"``, `values`: [BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getRoleMemberCount"`` |
| `values` | [BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:104

▸ **encodeFunctionData**(`functionFragment`: ``"grantRole"``, `values`: [BytesLike, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"grantRole"`` |
| `values` | [BytesLike, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:108

▸ **encodeFunctionData**(`functionFragment`: ``"granularity"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"granularity"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:112

▸ **encodeFunctionData**(`functionFragment`: ``"hasRole"``, `values`: [BytesLike, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"hasRole"`` |
| `values` | [BytesLike, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:116

▸ **encodeFunctionData**(`functionFragment`: ``"isOperatorFor"``, `values`: [*string*, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"isOperatorFor"`` |
| `values` | [*string*, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:120

▸ **encodeFunctionData**(`functionFragment`: ``"mint"``, `values`: [*string*, BigNumberish, BytesLike, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"mint"`` |
| `values` | [*string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:124

▸ **encodeFunctionData**(`functionFragment`: ``"name"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"name"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:128

▸ **encodeFunctionData**(`functionFragment`: ``"operatorBurn"``, `values`: [*string*, BigNumberish, BytesLike, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"operatorBurn"`` |
| `values` | [*string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:129

▸ **encodeFunctionData**(`functionFragment`: ``"operatorSend"``, `values`: [*string*, *string*, BigNumberish, BytesLike, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"operatorSend"`` |
| `values` | [*string*, *string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:133

▸ **encodeFunctionData**(`functionFragment`: ``"renounceRole"``, `values`: [BytesLike, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"renounceRole"`` |
| `values` | [BytesLike, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:137

▸ **encodeFunctionData**(`functionFragment`: ``"revokeOperator"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"revokeOperator"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:141

▸ **encodeFunctionData**(`functionFragment`: ``"revokeRole"``, `values`: [BytesLike, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"revokeRole"`` |
| `values` | [BytesLike, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:145

▸ **encodeFunctionData**(`functionFragment`: ``"send"``, `values`: [*string*, BigNumberish, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"send"`` |
| `values` | [*string*, BigNumberish, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:149

▸ **encodeFunctionData**(`functionFragment`: ``"symbol"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"symbol"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:153

▸ **encodeFunctionData**(`functionFragment`: ``"totalSupply"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalSupply"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:154

▸ **encodeFunctionData**(`functionFragment`: ``"totalSupplyAt"``, `values`: [BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalSupplyAt"`` |
| `values` | [BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:158

▸ **encodeFunctionData**(`functionFragment`: ``"totalSupplySnapshots"``, `values`: [BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalSupplySnapshots"`` |
| `values` | [BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:162

▸ **encodeFunctionData**(`functionFragment`: ``"transfer"``, `values`: [*string*, BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"transfer"`` |
| `values` | [*string*, BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:166

▸ **encodeFunctionData**(`functionFragment`: ``"transferFrom"``, `values`: [*string*, *string*, BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"transferFrom"`` |
| `values` | [*string*, *string*, BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:170

___

### encodeFunctionResult

▸ **encodeFunctionResult**(`functionFragment`: *string* \| *FunctionFragment*, `values?`: readonly *any*[]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | *string* \| *FunctionFragment* |
| `values?` | readonly *any*[] |

**Returns:** *string*

Inherited from: ethers.utils.Interface.encodeFunctionResult

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:61

___

### format

▸ **format**(`format?`: *string*): *string* \| *string*[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `format?` | *string* |

**Returns:** *string* \| *string*[]

Inherited from: ethers.utils.Interface.format

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:46

___

### getEvent

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Approval"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Approval"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:277

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"AuthorizedOperator"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"AuthorizedOperator"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:278

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Burned"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Burned"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:279

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Minted"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Minted"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:280

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"RevokedOperator"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"RevokedOperator"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:281

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"RoleGranted"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"RoleGranted"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:282

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"RoleRevoked"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"RoleRevoked"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:283

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Sent"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Sent"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:284

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Transfer"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Transfer"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:285

___

### getEventTopic

▸ **getEventTopic**(`eventFragment`: *string* \| *EventFragment*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFragment` | *string* \| *EventFragment* |

**Returns:** *string*

Inherited from: ethers.utils.Interface.getEventTopic

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:54

___

### getFunction

▸ **getFunction**(`nameOrSignatureOrSighash`: *string*): *FunctionFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrSighash` | *string* |

**Returns:** *FunctionFragment*

Inherited from: ethers.utils.Interface.getFunction

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:51

___

### getSighash

▸ **getSighash**(`functionFragment`: *string* \| *FunctionFragment*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | *string* \| *FunctionFragment* |

**Returns:** *string*

Inherited from: ethers.utils.Interface.getSighash

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:53

___

### parseLog

▸ **parseLog**(`log`: { `data`: *string* ; `topics`: *string*[]  }): *LogDescription*

#### Parameters

| Name | Type |
| :------ | :------ |
| `log` | *object* |
| `log.data` | *string* |
| `log.topics` | *string*[] |

**Returns:** *LogDescription*

Inherited from: ethers.utils.Interface.parseLog

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:72

___

### parseTransaction

▸ **parseTransaction**(`tx`: { `data`: *string* ; `value?`: BigNumberish  }): *TransactionDescription*

#### Parameters

| Name | Type |
| :------ | :------ |
| `tx` | *object* |
| `tx.data` | *string* |
| `tx.value?` | BigNumberish |

**Returns:** *TransactionDescription*

Inherited from: ethers.utils.Interface.parseTransaction

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:68
