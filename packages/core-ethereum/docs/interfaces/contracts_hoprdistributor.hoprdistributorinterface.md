[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprDistributor](../modules/contracts_hoprdistributor.md) / HoprDistributorInterface

# Interface: HoprDistributorInterface

[contracts/HoprDistributor](../modules/contracts_hoprdistributor.md).HoprDistributorInterface

## Hierarchy

- _Interface_

  ↳ **HoprDistributorInterface**

## Table of contents

### Properties

- [\_abiCoder](contracts_hoprdistributor.hoprdistributorinterface.md#_abicoder)
- [\_isInterface](contracts_hoprdistributor.hoprdistributorinterface.md#_isinterface)
- [deploy](contracts_hoprdistributor.hoprdistributorinterface.md#deploy)
- [errors](contracts_hoprdistributor.hoprdistributorinterface.md#errors)
- [events](contracts_hoprdistributor.hoprdistributorinterface.md#events)
- [fragments](contracts_hoprdistributor.hoprdistributorinterface.md#fragments)
- [functions](contracts_hoprdistributor.hoprdistributorinterface.md#functions)
- [structs](contracts_hoprdistributor.hoprdistributorinterface.md#structs)

### Methods

- [\_decodeParams](contracts_hoprdistributor.hoprdistributorinterface.md#_decodeparams)
- [\_encodeParams](contracts_hoprdistributor.hoprdistributorinterface.md#_encodeparams)
- [decodeEventLog](contracts_hoprdistributor.hoprdistributorinterface.md#decodeeventlog)
- [decodeFunctionData](contracts_hoprdistributor.hoprdistributorinterface.md#decodefunctiondata)
- [decodeFunctionResult](contracts_hoprdistributor.hoprdistributorinterface.md#decodefunctionresult)
- [encodeDeploy](contracts_hoprdistributor.hoprdistributorinterface.md#encodedeploy)
- [encodeEventLog](contracts_hoprdistributor.hoprdistributorinterface.md#encodeeventlog)
- [encodeFilterTopics](contracts_hoprdistributor.hoprdistributorinterface.md#encodefiltertopics)
- [encodeFunctionData](contracts_hoprdistributor.hoprdistributorinterface.md#encodefunctiondata)
- [encodeFunctionResult](contracts_hoprdistributor.hoprdistributorinterface.md#encodefunctionresult)
- [format](contracts_hoprdistributor.hoprdistributorinterface.md#format)
- [getEvent](contracts_hoprdistributor.hoprdistributorinterface.md#getevent)
- [getEventTopic](contracts_hoprdistributor.hoprdistributorinterface.md#geteventtopic)
- [getFunction](contracts_hoprdistributor.hoprdistributorinterface.md#getfunction)
- [getSighash](contracts_hoprdistributor.hoprdistributorinterface.md#getsighash)
- [parseLog](contracts_hoprdistributor.hoprdistributorinterface.md#parselog)
- [parseTransaction](contracts_hoprdistributor.hoprdistributorinterface.md#parsetransaction)

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

| Name                                        | Type            |
| :------------------------------------------ | :-------------- |
| `AllocationAdded(address,uint128,string)`   | _EventFragment_ |
| `Claimed(address,uint128,string)`           | _EventFragment_ |
| `OwnershipTransferred(address,address)`     | _EventFragment_ |
| `ScheduleAdded(uint128[],uint128[],string)` | _EventFragment_ |

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:157

---

### fragments

• `Readonly` **fragments**: readonly _Fragment_[]

Inherited from: ethers.utils.Interface.fragments

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:29

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                         | Type               |
| :------------------------------------------- | :----------------- |
| `MULTIPLIER()`                               | _FunctionFragment_ |
| `addAllocations(address[],uint128[],string)` | _FunctionFragment_ |
| `addSchedule(uint128[],uint128[],string)`    | _FunctionFragment_ |
| `allocations(address,string)`                | _FunctionFragment_ |
| `claim(string)`                              | _FunctionFragment_ |
| `claimFor(address,string)`                   | _FunctionFragment_ |
| `getClaimable(address,string)`               | _FunctionFragment_ |
| `getSchedule(string)`                        | _FunctionFragment_ |
| `maxMintAmount()`                            | _FunctionFragment_ |
| `owner()`                                    | _FunctionFragment_ |
| `renounceOwnership()`                        | _FunctionFragment_ |
| `revokeAccount(address,string)`              | _FunctionFragment_ |
| `startTime()`                                | _FunctionFragment_ |
| `token()`                                    | _FunctionFragment_ |
| `totalMinted()`                              | _FunctionFragment_ |
| `totalToBeMinted()`                          | _FunctionFragment_ |
| `transferOwnership(address)`                 | _FunctionFragment_ |
| `updateStartTime(uint128)`                   | _FunctionFragment_ |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:23

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

▸ **decodeFunctionResult**(`functionFragment`: `"MULTIPLIER"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type           |
| :----------------- | :------------- |
| `functionFragment` | `"MULTIPLIER"` |
| `data`             | BytesLike      |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:102

▸ **decodeFunctionResult**(`functionFragment`: `"addAllocations"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type               |
| :----------------- | :----------------- |
| `functionFragment` | `"addAllocations"` |
| `data`             | BytesLike          |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:103

▸ **decodeFunctionResult**(`functionFragment`: `"addSchedule"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"addSchedule"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:107

▸ **decodeFunctionResult**(`functionFragment`: `"allocations"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"allocations"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:111

▸ **decodeFunctionResult**(`functionFragment`: `"claim"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"claim"` |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:115

▸ **decodeFunctionResult**(`functionFragment`: `"claimFor"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"claimFor"` |
| `data`             | BytesLike    |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:116

▸ **decodeFunctionResult**(`functionFragment`: `"getClaimable"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type             |
| :----------------- | :--------------- |
| `functionFragment` | `"getClaimable"` |
| `data`             | BytesLike        |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:117

▸ **decodeFunctionResult**(`functionFragment`: `"getSchedule"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"getSchedule"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:121

▸ **decodeFunctionResult**(`functionFragment`: `"maxMintAmount"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"maxMintAmount"` |
| `data`             | BytesLike         |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:125

▸ **decodeFunctionResult**(`functionFragment`: `"owner"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"owner"` |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:129

▸ **decodeFunctionResult**(`functionFragment`: `"renounceOwnership"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"renounceOwnership"` |
| `data`             | BytesLike             |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:130

▸ **decodeFunctionResult**(`functionFragment`: `"revokeAccount"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"revokeAccount"` |
| `data`             | BytesLike         |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:134

▸ **decodeFunctionResult**(`functionFragment`: `"startTime"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type          |
| :----------------- | :------------ |
| `functionFragment` | `"startTime"` |
| `data`             | BytesLike     |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:138

▸ **decodeFunctionResult**(`functionFragment`: `"token"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"token"` |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:139

▸ **decodeFunctionResult**(`functionFragment`: `"totalMinted"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"totalMinted"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:140

▸ **decodeFunctionResult**(`functionFragment`: `"totalToBeMinted"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                |
| :----------------- | :------------------ |
| `functionFragment` | `"totalToBeMinted"` |
| `data`             | BytesLike           |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:144

▸ **decodeFunctionResult**(`functionFragment`: `"transferOwnership"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"transferOwnership"` |
| `data`             | BytesLike             |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:148

▸ **decodeFunctionResult**(`functionFragment`: `"updateStartTime"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                |
| :----------------- | :------------------ |
| `functionFragment` | `"updateStartTime"` |
| `data`             | BytesLike           |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:152

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

▸ **encodeFunctionData**(`functionFragment`: `"MULTIPLIER"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type           |
| :----------------- | :------------- |
| `functionFragment` | `"MULTIPLIER"` |
| `values?`          | _undefined_    |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:44

▸ **encodeFunctionData**(`functionFragment`: `"addAllocations"`, `values`: [_string_[], BigNumberish[], _string_]): _string_

#### Parameters

| Name               | Type                                   |
| :----------------- | :------------------------------------- |
| `functionFragment` | `"addAllocations"`                     |
| `values`           | [_string_[], BigNumberish[], _string_] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:48

▸ **encodeFunctionData**(`functionFragment`: `"addSchedule"`, `values`: [BigNumberish[], BigNumberish[], _string_]): _string_

#### Parameters

| Name               | Type                                       |
| :----------------- | :----------------------------------------- |
| `functionFragment` | `"addSchedule"`                            |
| `values`           | [BigNumberish[], BigNumberish[], _string_] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:52

▸ **encodeFunctionData**(`functionFragment`: `"allocations"`, `values`: [*string*, *string*]): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"allocations"`      |
| `values`           | [*string*, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:56

▸ **encodeFunctionData**(`functionFragment`: `"claim"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type       |
| :----------------- | :--------- |
| `functionFragment` | `"claim"`  |
| `values`           | [*string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:60

▸ **encodeFunctionData**(`functionFragment`: `"claimFor"`, `values`: [*string*, *string*]): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"claimFor"`         |
| `values`           | [*string*, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:61

▸ **encodeFunctionData**(`functionFragment`: `"getClaimable"`, `values`: [*string*, *string*]): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"getClaimable"`     |
| `values`           | [*string*, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:65

▸ **encodeFunctionData**(`functionFragment`: `"getSchedule"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"getSchedule"` |
| `values`           | [*string*]      |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:69

▸ **encodeFunctionData**(`functionFragment`: `"maxMintAmount"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"maxMintAmount"` |
| `values?`          | _undefined_       |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:70

▸ **encodeFunctionData**(`functionFragment`: `"owner"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"owner"`   |
| `values?`          | _undefined_ |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:74

▸ **encodeFunctionData**(`functionFragment`: `"renounceOwnership"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"renounceOwnership"` |
| `values?`          | _undefined_           |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:75

▸ **encodeFunctionData**(`functionFragment`: `"revokeAccount"`, `values`: [*string*, *string*]): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"revokeAccount"`    |
| `values`           | [*string*, *string*] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:79

▸ **encodeFunctionData**(`functionFragment`: `"startTime"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type          |
| :----------------- | :------------ |
| `functionFragment` | `"startTime"` |
| `values?`          | _undefined_   |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:83

▸ **encodeFunctionData**(`functionFragment`: `"token"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"token"`   |
| `values?`          | _undefined_ |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:84

▸ **encodeFunctionData**(`functionFragment`: `"totalMinted"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"totalMinted"` |
| `values?`          | _undefined_     |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:85

▸ **encodeFunctionData**(`functionFragment`: `"totalToBeMinted"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                |
| :----------------- | :------------------ |
| `functionFragment` | `"totalToBeMinted"` |
| `values?`          | _undefined_         |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:89

▸ **encodeFunctionData**(`functionFragment`: `"transferOwnership"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"transferOwnership"` |
| `values`           | [*string*]            |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:93

▸ **encodeFunctionData**(`functionFragment`: `"updateStartTime"`, `values`: [BigNumberish]): _string_

#### Parameters

| Name               | Type                |
| :----------------- | :------------------ |
| `functionFragment` | `"updateStartTime"` |
| `values`           | [BigNumberish]      |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:97

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

▸ **getEvent**(`nameOrSignatureOrTopic`: `"AllocationAdded"`): _EventFragment_

#### Parameters

| Name                     | Type                |
| :----------------------- | :------------------ |
| `nameOrSignatureOrTopic` | `"AllocationAdded"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:164

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Claimed"`): _EventFragment_

#### Parameters

| Name                     | Type        |
| :----------------------- | :---------- |
| `nameOrSignatureOrTopic` | `"Claimed"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:165

▸ **getEvent**(`nameOrSignatureOrTopic`: `"OwnershipTransferred"`): _EventFragment_

#### Parameters

| Name                     | Type                     |
| :----------------------- | :----------------------- |
| `nameOrSignatureOrTopic` | `"OwnershipTransferred"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:166

▸ **getEvent**(`nameOrSignatureOrTopic`: `"ScheduleAdded"`): _EventFragment_

#### Parameters

| Name                     | Type              |
| :----------------------- | :---------------- |
| `nameOrSignatureOrTopic` | `"ScheduleAdded"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:167

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
