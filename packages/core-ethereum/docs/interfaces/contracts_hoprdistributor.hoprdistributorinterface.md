[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprDistributor](../modules/contracts_hoprdistributor.md) / HoprDistributorInterface

# Interface: HoprDistributorInterface

[contracts/HoprDistributor](../modules/contracts_hoprdistributor.md).HoprDistributorInterface

## Hierarchy

- *Interface*

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
| `AllocationAdded(address,uint128,string)` | *EventFragment* |
| `Claimed(address,uint128,string)` | *EventFragment* |
| `OwnershipTransferred(address,address)` | *EventFragment* |
| `ScheduleAdded(uint128[],uint128[],string)` | *EventFragment* |

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:157

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
| `MULTIPLIER()` | *FunctionFragment* |
| `addAllocations(address[],uint128[],string)` | *FunctionFragment* |
| `addSchedule(uint128[],uint128[],string)` | *FunctionFragment* |
| `allocations(address,string)` | *FunctionFragment* |
| `claim(string)` | *FunctionFragment* |
| `claimFor(address,string)` | *FunctionFragment* |
| `getClaimable(address,string)` | *FunctionFragment* |
| `getSchedule(string)` | *FunctionFragment* |
| `maxMintAmount()` | *FunctionFragment* |
| `owner()` | *FunctionFragment* |
| `renounceOwnership()` | *FunctionFragment* |
| `revokeAccount(address,string)` | *FunctionFragment* |
| `startTime()` | *FunctionFragment* |
| `token()` | *FunctionFragment* |
| `totalMinted()` | *FunctionFragment* |
| `totalToBeMinted()` | *FunctionFragment* |
| `transferOwnership(address)` | *FunctionFragment* |
| `updateStartTime(uint128)` | *FunctionFragment* |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:23

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

▸ **decodeFunctionResult**(`functionFragment`: ``"MULTIPLIER"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"MULTIPLIER"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:102

▸ **decodeFunctionResult**(`functionFragment`: ``"addAllocations"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"addAllocations"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:103

▸ **decodeFunctionResult**(`functionFragment`: ``"addSchedule"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"addSchedule"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:107

▸ **decodeFunctionResult**(`functionFragment`: ``"allocations"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"allocations"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:111

▸ **decodeFunctionResult**(`functionFragment`: ``"claim"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"claim"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:115

▸ **decodeFunctionResult**(`functionFragment`: ``"claimFor"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"claimFor"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:116

▸ **decodeFunctionResult**(`functionFragment`: ``"getClaimable"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getClaimable"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:117

▸ **decodeFunctionResult**(`functionFragment`: ``"getSchedule"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getSchedule"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:121

▸ **decodeFunctionResult**(`functionFragment`: ``"maxMintAmount"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"maxMintAmount"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:125

▸ **decodeFunctionResult**(`functionFragment`: ``"owner"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"owner"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:129

▸ **decodeFunctionResult**(`functionFragment`: ``"renounceOwnership"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"renounceOwnership"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:130

▸ **decodeFunctionResult**(`functionFragment`: ``"revokeAccount"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"revokeAccount"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:134

▸ **decodeFunctionResult**(`functionFragment`: ``"startTime"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"startTime"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:138

▸ **decodeFunctionResult**(`functionFragment`: ``"token"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"token"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:139

▸ **decodeFunctionResult**(`functionFragment`: ``"totalMinted"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalMinted"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:140

▸ **decodeFunctionResult**(`functionFragment`: ``"totalToBeMinted"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalToBeMinted"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:144

▸ **decodeFunctionResult**(`functionFragment`: ``"transferOwnership"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"transferOwnership"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:148

▸ **decodeFunctionResult**(`functionFragment`: ``"updateStartTime"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"updateStartTime"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:152

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

▸ **encodeFunctionData**(`functionFragment`: ``"MULTIPLIER"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"MULTIPLIER"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:44

▸ **encodeFunctionData**(`functionFragment`: ``"addAllocations"``, `values`: [*string*[], BigNumberish[], *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"addAllocations"`` |
| `values` | [*string*[], BigNumberish[], *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:48

▸ **encodeFunctionData**(`functionFragment`: ``"addSchedule"``, `values`: [BigNumberish[], BigNumberish[], *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"addSchedule"`` |
| `values` | [BigNumberish[], BigNumberish[], *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:52

▸ **encodeFunctionData**(`functionFragment`: ``"allocations"``, `values`: [*string*, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"allocations"`` |
| `values` | [*string*, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:56

▸ **encodeFunctionData**(`functionFragment`: ``"claim"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"claim"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:60

▸ **encodeFunctionData**(`functionFragment`: ``"claimFor"``, `values`: [*string*, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"claimFor"`` |
| `values` | [*string*, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:61

▸ **encodeFunctionData**(`functionFragment`: ``"getClaimable"``, `values`: [*string*, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getClaimable"`` |
| `values` | [*string*, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:65

▸ **encodeFunctionData**(`functionFragment`: ``"getSchedule"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"getSchedule"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:69

▸ **encodeFunctionData**(`functionFragment`: ``"maxMintAmount"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"maxMintAmount"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:70

▸ **encodeFunctionData**(`functionFragment`: ``"owner"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"owner"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:74

▸ **encodeFunctionData**(`functionFragment`: ``"renounceOwnership"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"renounceOwnership"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:75

▸ **encodeFunctionData**(`functionFragment`: ``"revokeAccount"``, `values`: [*string*, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"revokeAccount"`` |
| `values` | [*string*, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:79

▸ **encodeFunctionData**(`functionFragment`: ``"startTime"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"startTime"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:83

▸ **encodeFunctionData**(`functionFragment`: ``"token"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"token"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:84

▸ **encodeFunctionData**(`functionFragment`: ``"totalMinted"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalMinted"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:85

▸ **encodeFunctionData**(`functionFragment`: ``"totalToBeMinted"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"totalToBeMinted"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:89

▸ **encodeFunctionData**(`functionFragment`: ``"transferOwnership"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"transferOwnership"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:93

▸ **encodeFunctionData**(`functionFragment`: ``"updateStartTime"``, `values`: [BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"updateStartTime"`` |
| `values` | [BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:97

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

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"AllocationAdded"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"AllocationAdded"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:164

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Claimed"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Claimed"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:165

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"OwnershipTransferred"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"OwnershipTransferred"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:166

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"ScheduleAdded"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"ScheduleAdded"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:167

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
