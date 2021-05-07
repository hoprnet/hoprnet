[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprToken](../modules/contracts_hoprtoken.md) / HoprTokenInterface

# Interface: HoprTokenInterface

[contracts/HoprToken](../modules/contracts_hoprtoken.md).HoprTokenInterface

## Hierarchy

- _Interface_

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

• `Readonly` **\_abiCoder**: _AbiCoder_

Inherited from: ethers.utils.Interface.\_abiCoder

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:43

---

### \_isInterface

• `Readonly` **\_isInterface**: _boolean_

Inherited from: ethers.utils.Interface.\_isInterface

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:44

---

### deploy

• `Readonly` **deploy**: _ConstructorFragment_

Inherited from: ethers.utils.Interface.deploy

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:42

---

### errors

• `Readonly` **errors**: _object_

#### Type declaration

Inherited from: ethers.utils.Interface.errors

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:30

---

### events

• **events**: _object_

#### Type declaration

| Name                                                | Type            |
| :-------------------------------------------------- | :-------------- |
| `Approval(address,address,uint256)`                 | _EventFragment_ |
| `AuthorizedOperator(address,address)`               | _EventFragment_ |
| `Burned(address,address,uint256,bytes,bytes)`       | _EventFragment_ |
| `Minted(address,address,uint256,bytes,bytes)`       | _EventFragment_ |
| `RevokedOperator(address,address)`                  | _EventFragment_ |
| `RoleGranted(bytes32,address,address)`              | _EventFragment_ |
| `RoleRevoked(bytes32,address,address)`              | _EventFragment_ |
| `Sent(address,address,address,uint256,bytes,bytes)` | _EventFragment_ |
| `Transfer(address,address,uint256)`                 | _EventFragment_ |

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:265

---

### fragments

• `Readonly` **fragments**: readonly _Fragment_[]

Inherited from: ethers.utils.Interface.fragments

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:29

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                                | Type               |
| :-------------------------------------------------- | :----------------- |
| `DEFAULT_ADMIN_ROLE()`                              | _FunctionFragment_ |
| `MINTER_ROLE()`                                     | _FunctionFragment_ |
| `accountSnapshots(address,uint256)`                 | _FunctionFragment_ |
| `allowance(address,address)`                        | _FunctionFragment_ |
| `approve(address,uint256)`                          | _FunctionFragment_ |
| `authorizeOperator(address)`                        | _FunctionFragment_ |
| `balanceOf(address)`                                | _FunctionFragment_ |
| `balanceOfAt(address,uint128)`                      | _FunctionFragment_ |
| `burn(uint256,bytes)`                               | _FunctionFragment_ |
| `decimals()`                                        | _FunctionFragment_ |
| `defaultOperators()`                                | _FunctionFragment_ |
| `getRoleAdmin(bytes32)`                             | _FunctionFragment_ |
| `getRoleMember(bytes32,uint256)`                    | _FunctionFragment_ |
| `getRoleMemberCount(bytes32)`                       | _FunctionFragment_ |
| `grantRole(bytes32,address)`                        | _FunctionFragment_ |
| `granularity()`                                     | _FunctionFragment_ |
| `hasRole(bytes32,address)`                          | _FunctionFragment_ |
| `isOperatorFor(address,address)`                    | _FunctionFragment_ |
| `mint(address,uint256,bytes,bytes)`                 | _FunctionFragment_ |
| `name()`                                            | _FunctionFragment_ |
| `operatorBurn(address,uint256,bytes,bytes)`         | _FunctionFragment_ |
| `operatorSend(address,address,uint256,bytes,bytes)` | _FunctionFragment_ |
| `renounceRole(bytes32,address)`                     | _FunctionFragment_ |
| `revokeOperator(address)`                           | _FunctionFragment_ |
| `revokeRole(bytes32,address)`                       | _FunctionFragment_ |
| `send(address,uint256,bytes)`                       | _FunctionFragment_ |
| `symbol()`                                          | _FunctionFragment_ |
| `totalSupply()`                                     | _FunctionFragment_ |
| `totalSupplyAt(uint128)`                            | _FunctionFragment_ |
| `totalSupplySnapshots(uint256)`                     | _FunctionFragment_ |
| `transfer(address,uint256)`                         | _FunctionFragment_ |
| `transferFrom(address,address,uint256)`             | _FunctionFragment_ |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:23

---

### structs

• `Readonly` **structs**: _object_

#### Type declaration

Inherited from: ethers.utils.Interface.structs

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:39

## Methods

### \_decodeParams

▸ **\_decodeParams**(`params`: readonly _ParamType_[], `data`: BytesLike): _Result_

#### Parameters

| Name     | Type                   |
| :------- | :--------------------- |
| `params` | readonly _ParamType_[] |
| `data`   | BytesLike              |

**Returns:** _Result_

Inherited from: ethers.utils.Interface.\_decodeParams

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:55

---

### \_encodeParams

▸ **\_encodeParams**(`params`: readonly _ParamType_[], `values`: readonly _any_[]): _string_

#### Parameters

| Name     | Type                   |
| :------- | :--------------------- |
| `params` | readonly _ParamType_[] |
| `values` | readonly _any_[]       |

**Returns:** _string_

Inherited from: ethers.utils.Interface.\_encodeParams

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:56

---

### decodeEventLog

▸ **decodeEventLog**(`eventFragment`: _string_ \| _EventFragment_, `data`: BytesLike, `topics?`: readonly _string_[]): _Result_

#### Parameters

| Name            | Type                        |
| :-------------- | :-------------------------- |
| `eventFragment` | _string_ \| _EventFragment_ |
| `data`          | BytesLike                   |
| `topics?`       | readonly _string_[]         |

**Returns:** _Result_

Inherited from: ethers.utils.Interface.decodeEventLog

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:67

---

### decodeFunctionData

▸ **decodeFunctionData**(`functionFragment`: _string_ \| _FunctionFragment_, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                           |
| :----------------- | :----------------------------- |
| `functionFragment` | _string_ \| _FunctionFragment_ |
| `data`             | BytesLike                      |

**Returns:** _Result_

Inherited from: ethers.utils.Interface.decodeFunctionData

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:58

---

### decodeFunctionResult

▸ **decodeFunctionResult**(`functionFragment`: `"DEFAULT_ADMIN_ROLE"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                   |
| :----------------- | :--------------------- |
| `functionFragment` | `"DEFAULT_ADMIN_ROLE"` |
| `data`             | BytesLike              |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:175

▸ **decodeFunctionResult**(`functionFragment`: `"MINTER_ROLE"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"MINTER_ROLE"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:179

▸ **decodeFunctionResult**(`functionFragment`: `"accountSnapshots"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"accountSnapshots"` |
| `data`             | BytesLike            |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:183

▸ **decodeFunctionResult**(`functionFragment`: `"allowance"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type          |
| :----------------- | :------------ |
| `functionFragment` | `"allowance"` |
| `data`             | BytesLike     |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:187

▸ **decodeFunctionResult**(`functionFragment`: `"approve"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"approve"` |
| `data`             | BytesLike   |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:188

▸ **decodeFunctionResult**(`functionFragment`: `"authorizeOperator"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"authorizeOperator"` |
| `data`             | BytesLike             |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:189

▸ **decodeFunctionResult**(`functionFragment`: `"balanceOf"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type          |
| :----------------- | :------------ |
| `functionFragment` | `"balanceOf"` |
| `data`             | BytesLike     |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:193

▸ **decodeFunctionResult**(`functionFragment`: `"balanceOfAt"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"balanceOfAt"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:194

▸ **decodeFunctionResult**(`functionFragment`: `"burn"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"burn"`  |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:198

▸ **decodeFunctionResult**(`functionFragment`: `"decimals"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"decimals"` |
| `data`             | BytesLike    |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:199

▸ **decodeFunctionResult**(`functionFragment`: `"defaultOperators"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"defaultOperators"` |
| `data`             | BytesLike            |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:200

▸ **decodeFunctionResult**(`functionFragment`: `"getRoleAdmin"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type             |
| :----------------- | :--------------- |
| `functionFragment` | `"getRoleAdmin"` |
| `data`             | BytesLike        |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:204

▸ **decodeFunctionResult**(`functionFragment`: `"getRoleMember"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"getRoleMember"` |
| `data`             | BytesLike         |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:208

▸ **decodeFunctionResult**(`functionFragment`: `"getRoleMemberCount"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                   |
| :----------------- | :--------------------- |
| `functionFragment` | `"getRoleMemberCount"` |
| `data`             | BytesLike              |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:212

▸ **decodeFunctionResult**(`functionFragment`: `"grantRole"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type          |
| :----------------- | :------------ |
| `functionFragment` | `"grantRole"` |
| `data`             | BytesLike     |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:216

▸ **decodeFunctionResult**(`functionFragment`: `"granularity"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"granularity"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:217

▸ **decodeFunctionResult**(`functionFragment`: `"hasRole"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"hasRole"` |
| `data`             | BytesLike   |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:221

▸ **decodeFunctionResult**(`functionFragment`: `"isOperatorFor"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"isOperatorFor"` |
| `data`             | BytesLike         |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:222

▸ **decodeFunctionResult**(`functionFragment`: `"mint"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"mint"`  |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:226

▸ **decodeFunctionResult**(`functionFragment`: `"name"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"name"`  |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:227

▸ **decodeFunctionResult**(`functionFragment`: `"operatorBurn"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type             |
| :----------------- | :--------------- |
| `functionFragment` | `"operatorBurn"` |
| `data`             | BytesLike        |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:228

▸ **decodeFunctionResult**(`functionFragment`: `"operatorSend"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type             |
| :----------------- | :--------------- |
| `functionFragment` | `"operatorSend"` |
| `data`             | BytesLike        |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:232

▸ **decodeFunctionResult**(`functionFragment`: `"renounceRole"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type             |
| :----------------- | :--------------- |
| `functionFragment` | `"renounceRole"` |
| `data`             | BytesLike        |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:236

▸ **decodeFunctionResult**(`functionFragment`: `"revokeOperator"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type               |
| :----------------- | :----------------- |
| `functionFragment` | `"revokeOperator"` |
| `data`             | BytesLike          |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:240

▸ **decodeFunctionResult**(`functionFragment`: `"revokeRole"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type           |
| :----------------- | :------------- |
| `functionFragment` | `"revokeRole"` |
| `data`             | BytesLike      |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:244

▸ **decodeFunctionResult**(`functionFragment`: `"send"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"send"`  |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:245

▸ **decodeFunctionResult**(`functionFragment`: `"symbol"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type       |
| :----------------- | :--------- |
| `functionFragment` | `"symbol"` |
| `data`             | BytesLike  |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:246

▸ **decodeFunctionResult**(`functionFragment`: `"totalSupply"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"totalSupply"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:247

▸ **decodeFunctionResult**(`functionFragment`: `"totalSupplyAt"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"totalSupplyAt"` |
| `data`             | BytesLike         |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:251

▸ **decodeFunctionResult**(`functionFragment`: `"totalSupplySnapshots"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                     |
| :----------------- | :----------------------- |
| `functionFragment` | `"totalSupplySnapshots"` |
| `data`             | BytesLike                |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:255

▸ **decodeFunctionResult**(`functionFragment`: `"transfer"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"transfer"` |
| `data`             | BytesLike    |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:259

▸ **decodeFunctionResult**(`functionFragment`: `"transferFrom"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type             |
| :----------------- | :--------------- |
| `functionFragment` | `"transferFrom"` |
| `data`             | BytesLike        |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:260

---

### encodeDeploy

▸ **encodeDeploy**(`values?`: readonly _any_[]): _string_

#### Parameters

| Name      | Type             |
| :-------- | :--------------- |
| `values?` | readonly _any_[] |

**Returns:** _string_

Inherited from: ethers.utils.Interface.encodeDeploy

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:57

---

### encodeEventLog

▸ **encodeEventLog**(`eventFragment`: _EventFragment_, `values`: readonly _any_[]): _object_

#### Parameters

| Name            | Type             |
| :-------------- | :--------------- |
| `eventFragment` | _EventFragment_  |
| `values`        | readonly _any_[] |

**Returns:** _object_

| Name     | Type       |
| :------- | :--------- |
| `data`   | _string_   |
| `topics` | _string_[] |

Inherited from: ethers.utils.Interface.encodeEventLog

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:63

---

### encodeFilterTopics

▸ **encodeFilterTopics**(`eventFragment`: _EventFragment_, `values`: readonly _any_[]): (_string_ \| _string_[])[]

#### Parameters

| Name            | Type             |
| :-------------- | :--------------- |
| `eventFragment` | _EventFragment_  |
| `values`        | readonly _any_[] |

**Returns:** (_string_ \| _string_[])[]

Inherited from: ethers.utils.Interface.encodeFilterTopics

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:62

---

### encodeFunctionData

▸ **encodeFunctionData**(`functionFragment`: `"DEFAULT_ADMIN_ROLE"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                   |
| :----------------- | :--------------------- |
| `functionFragment` | `"DEFAULT_ADMIN_ROLE"` |
| `values?`          | _undefined_            |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:58

▸ **encodeFunctionData**(`functionFragment`: `"MINTER_ROLE"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"MINTER_ROLE"` |
| `values?`          | _undefined_     |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:62

▸ **encodeFunctionData**(`functionFragment`: `"accountSnapshots"`, `values`: [*string*, BigNumberish]): _string_

#### Parameters

| Name               | Type                     |
| :----------------- | :----------------------- |
| `functionFragment` | `"accountSnapshots"`     |
| `values`           | [*string*, BigNumberish] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:66

▸ **encodeFunctionData**(`functionFragment`: `"allowance"`, `values`: [*string*, *string*]): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"allowance"`        |
| `values`           | [*string*, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:70

▸ **encodeFunctionData**(`functionFragment`: `"approve"`, `values`: [*string*, BigNumberish]): _string_

#### Parameters

| Name               | Type                     |
| :----------------- | :----------------------- |
| `functionFragment` | `"approve"`              |
| `values`           | [*string*, BigNumberish] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:74

▸ **encodeFunctionData**(`functionFragment`: `"authorizeOperator"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"authorizeOperator"` |
| `values`           | [*string*]            |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:78

▸ **encodeFunctionData**(`functionFragment`: `"balanceOf"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type          |
| :----------------- | :------------ |
| `functionFragment` | `"balanceOf"` |
| `values`           | [*string*]    |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:82

▸ **encodeFunctionData**(`functionFragment`: `"balanceOfAt"`, `values`: [*string*, BigNumberish]): _string_

#### Parameters

| Name               | Type                     |
| :----------------- | :----------------------- |
| `functionFragment` | `"balanceOfAt"`          |
| `values`           | [*string*, BigNumberish] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:83

▸ **encodeFunctionData**(`functionFragment`: `"burn"`, `values`: [BigNumberish, BytesLike]): _string_

#### Parameters

| Name               | Type                      |
| :----------------- | :------------------------ |
| `functionFragment` | `"burn"`                  |
| `values`           | [BigNumberish, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:87

▸ **encodeFunctionData**(`functionFragment`: `"decimals"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"decimals"` |
| `values?`          | _undefined_  |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:91

▸ **encodeFunctionData**(`functionFragment`: `"defaultOperators"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"defaultOperators"` |
| `values?`          | _undefined_          |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:92

▸ **encodeFunctionData**(`functionFragment`: `"getRoleAdmin"`, `values`: [BytesLike]): _string_

#### Parameters

| Name               | Type             |
| :----------------- | :--------------- |
| `functionFragment` | `"getRoleAdmin"` |
| `values`           | [BytesLike]      |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:96

▸ **encodeFunctionData**(`functionFragment`: `"getRoleMember"`, `values`: [BytesLike, BigNumberish]): _string_

#### Parameters

| Name               | Type                      |
| :----------------- | :------------------------ |
| `functionFragment` | `"getRoleMember"`         |
| `values`           | [BytesLike, BigNumberish] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:100

▸ **encodeFunctionData**(`functionFragment`: `"getRoleMemberCount"`, `values`: [BytesLike]): _string_

#### Parameters

| Name               | Type                   |
| :----------------- | :--------------------- |
| `functionFragment` | `"getRoleMemberCount"` |
| `values`           | [BytesLike]            |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:104

▸ **encodeFunctionData**(`functionFragment`: `"grantRole"`, `values`: [BytesLike, *string*]): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"grantRole"`         |
| `values`           | [BytesLike, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:108

▸ **encodeFunctionData**(`functionFragment`: `"granularity"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"granularity"` |
| `values?`          | _undefined_     |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:112

▸ **encodeFunctionData**(`functionFragment`: `"hasRole"`, `values`: [BytesLike, *string*]): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"hasRole"`           |
| `values`           | [BytesLike, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:116

▸ **encodeFunctionData**(`functionFragment`: `"isOperatorFor"`, `values`: [*string*, *string*]): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"isOperatorFor"`    |
| `values`           | [*string*, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:120

▸ **encodeFunctionData**(`functionFragment`: `"mint"`, `values`: [*string*, BigNumberish, BytesLike, BytesLike]): _string_

#### Parameters

| Name               | Type                                           |
| :----------------- | :--------------------------------------------- |
| `functionFragment` | `"mint"`                                       |
| `values`           | [*string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:124

▸ **encodeFunctionData**(`functionFragment`: `"name"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"name"`    |
| `values?`          | _undefined_ |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:128

▸ **encodeFunctionData**(`functionFragment`: `"operatorBurn"`, `values`: [*string*, BigNumberish, BytesLike, BytesLike]): _string_

#### Parameters

| Name               | Type                                           |
| :----------------- | :--------------------------------------------- |
| `functionFragment` | `"operatorBurn"`                               |
| `values`           | [*string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:129

▸ **encodeFunctionData**(`functionFragment`: `"operatorSend"`, `values`: [*string*, *string*, BigNumberish, BytesLike, BytesLike]): _string_

#### Parameters

| Name               | Type                                                     |
| :----------------- | :------------------------------------------------------- |
| `functionFragment` | `"operatorSend"`                                         |
| `values`           | [*string*, *string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:133

▸ **encodeFunctionData**(`functionFragment`: `"renounceRole"`, `values`: [BytesLike, *string*]): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"renounceRole"`      |
| `values`           | [BytesLike, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:137

▸ **encodeFunctionData**(`functionFragment`: `"revokeOperator"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type               |
| :----------------- | :----------------- |
| `functionFragment` | `"revokeOperator"` |
| `values`           | [*string*]         |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:141

▸ **encodeFunctionData**(`functionFragment`: `"revokeRole"`, `values`: [BytesLike, *string*]): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"revokeRole"`        |
| `values`           | [BytesLike, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:145

▸ **encodeFunctionData**(`functionFragment`: `"send"`, `values`: [*string*, BigNumberish, BytesLike]): _string_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"send"`                            |
| `values`           | [*string*, BigNumberish, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:149

▸ **encodeFunctionData**(`functionFragment`: `"symbol"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"symbol"`  |
| `values?`          | _undefined_ |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:153

▸ **encodeFunctionData**(`functionFragment`: `"totalSupply"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"totalSupply"` |
| `values?`          | _undefined_     |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:154

▸ **encodeFunctionData**(`functionFragment`: `"totalSupplyAt"`, `values`: [BigNumberish]): _string_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"totalSupplyAt"` |
| `values`           | [BigNumberish]    |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:158

▸ **encodeFunctionData**(`functionFragment`: `"totalSupplySnapshots"`, `values`: [BigNumberish]): _string_

#### Parameters

| Name               | Type                     |
| :----------------- | :----------------------- |
| `functionFragment` | `"totalSupplySnapshots"` |
| `values`           | [BigNumberish]           |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:162

▸ **encodeFunctionData**(`functionFragment`: `"transfer"`, `values`: [*string*, BigNumberish]): _string_

#### Parameters

| Name               | Type                     |
| :----------------- | :----------------------- |
| `functionFragment` | `"transfer"`             |
| `values`           | [*string*, BigNumberish] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:166

▸ **encodeFunctionData**(`functionFragment`: `"transferFrom"`, `values`: [*string*, *string*, BigNumberish]): _string_

#### Parameters

| Name               | Type                               |
| :----------------- | :--------------------------------- |
| `functionFragment` | `"transferFrom"`                   |
| `values`           | [*string*, *string*, BigNumberish] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:170

---

### encodeFunctionResult

▸ **encodeFunctionResult**(`functionFragment`: _string_ \| _FunctionFragment_, `values?`: readonly _any_[]): _string_

#### Parameters

| Name               | Type                           |
| :----------------- | :----------------------------- |
| `functionFragment` | _string_ \| _FunctionFragment_ |
| `values?`          | readonly _any_[]               |

**Returns:** _string_

Inherited from: ethers.utils.Interface.encodeFunctionResult

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:61

---

### format

▸ **format**(`format?`: _string_): _string_ \| _string_[]

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `format?` | _string_ |

**Returns:** _string_ \| _string_[]

Inherited from: ethers.utils.Interface.format

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:46

---

### getEvent

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Approval"`): _EventFragment_

#### Parameters

| Name                     | Type         |
| :----------------------- | :----------- |
| `nameOrSignatureOrTopic` | `"Approval"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:277

▸ **getEvent**(`nameOrSignatureOrTopic`: `"AuthorizedOperator"`): _EventFragment_

#### Parameters

| Name                     | Type                   |
| :----------------------- | :--------------------- |
| `nameOrSignatureOrTopic` | `"AuthorizedOperator"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:278

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Burned"`): _EventFragment_

#### Parameters

| Name                     | Type       |
| :----------------------- | :--------- |
| `nameOrSignatureOrTopic` | `"Burned"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:279

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Minted"`): _EventFragment_

#### Parameters

| Name                     | Type       |
| :----------------------- | :--------- |
| `nameOrSignatureOrTopic` | `"Minted"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:280

▸ **getEvent**(`nameOrSignatureOrTopic`: `"RevokedOperator"`): _EventFragment_

#### Parameters

| Name                     | Type                |
| :----------------------- | :------------------ |
| `nameOrSignatureOrTopic` | `"RevokedOperator"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:281

▸ **getEvent**(`nameOrSignatureOrTopic`: `"RoleGranted"`): _EventFragment_

#### Parameters

| Name                     | Type            |
| :----------------------- | :-------------- |
| `nameOrSignatureOrTopic` | `"RoleGranted"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:282

▸ **getEvent**(`nameOrSignatureOrTopic`: `"RoleRevoked"`): _EventFragment_

#### Parameters

| Name                     | Type            |
| :----------------------- | :-------------- |
| `nameOrSignatureOrTopic` | `"RoleRevoked"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:283

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Sent"`): _EventFragment_

#### Parameters

| Name                     | Type     |
| :----------------------- | :------- |
| `nameOrSignatureOrTopic` | `"Sent"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:284

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Transfer"`): _EventFragment_

#### Parameters

| Name                     | Type         |
| :----------------------- | :----------- |
| `nameOrSignatureOrTopic` | `"Transfer"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:285

---

### getEventTopic

▸ **getEventTopic**(`eventFragment`: _string_ \| _EventFragment_): _string_

#### Parameters

| Name            | Type                        |
| :-------------- | :-------------------------- |
| `eventFragment` | _string_ \| _EventFragment_ |

**Returns:** _string_

Inherited from: ethers.utils.Interface.getEventTopic

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:54

---

### getFunction

▸ **getFunction**(`nameOrSignatureOrSighash`: _string_): _FunctionFragment_

#### Parameters

| Name                       | Type     |
| :------------------------- | :------- |
| `nameOrSignatureOrSighash` | _string_ |

**Returns:** _FunctionFragment_

Inherited from: ethers.utils.Interface.getFunction

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:51

---

### getSighash

▸ **getSighash**(`functionFragment`: _string_ \| _FunctionFragment_): _string_

#### Parameters

| Name               | Type                           |
| :----------------- | :----------------------------- |
| `functionFragment` | _string_ \| _FunctionFragment_ |

**Returns:** _string_

Inherited from: ethers.utils.Interface.getSighash

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:53

---

### parseLog

▸ **parseLog**(`log`: { `data`: _string_ ; `topics`: _string_[] }): _LogDescription_

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `log`        | _object_   |
| `log.data`   | _string_   |
| `log.topics` | _string_[] |

**Returns:** _LogDescription_

Inherited from: ethers.utils.Interface.parseLog

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:72

---

### parseTransaction

▸ **parseTransaction**(`tx`: { `data`: _string_ ; `value?`: BigNumberish }): _TransactionDescription_

#### Parameters

| Name        | Type         |
| :---------- | :----------- |
| `tx`        | _object_     |
| `tx.data`   | _string_     |
| `tx.value?` | BigNumberish |

**Returns:** _TransactionDescription_

Inherited from: ethers.utils.Interface.parseTransaction

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:68
