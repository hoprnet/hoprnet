[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprWrapper](../modules/contracts_hoprwrapper.md) / HoprWrapperInterface

# Interface: HoprWrapperInterface

[contracts/HoprWrapper](../modules/contracts_hoprwrapper.md).HoprWrapperInterface

## Hierarchy

- _Interface_

  ↳ **HoprWrapperInterface**

## Table of contents

### Properties

- [\_abiCoder](contracts_hoprwrapper.hoprwrapperinterface.md#_abicoder)
- [\_isInterface](contracts_hoprwrapper.hoprwrapperinterface.md#_isinterface)
- [deploy](contracts_hoprwrapper.hoprwrapperinterface.md#deploy)
- [errors](contracts_hoprwrapper.hoprwrapperinterface.md#errors)
- [events](contracts_hoprwrapper.hoprwrapperinterface.md#events)
- [fragments](contracts_hoprwrapper.hoprwrapperinterface.md#fragments)
- [functions](contracts_hoprwrapper.hoprwrapperinterface.md#functions)
- [structs](contracts_hoprwrapper.hoprwrapperinterface.md#structs)

### Methods

- [\_decodeParams](contracts_hoprwrapper.hoprwrapperinterface.md#_decodeparams)
- [\_encodeParams](contracts_hoprwrapper.hoprwrapperinterface.md#_encodeparams)
- [decodeEventLog](contracts_hoprwrapper.hoprwrapperinterface.md#decodeeventlog)
- [decodeFunctionData](contracts_hoprwrapper.hoprwrapperinterface.md#decodefunctiondata)
- [decodeFunctionResult](contracts_hoprwrapper.hoprwrapperinterface.md#decodefunctionresult)
- [encodeDeploy](contracts_hoprwrapper.hoprwrapperinterface.md#encodedeploy)
- [encodeEventLog](contracts_hoprwrapper.hoprwrapperinterface.md#encodeeventlog)
- [encodeFilterTopics](contracts_hoprwrapper.hoprwrapperinterface.md#encodefiltertopics)
- [encodeFunctionData](contracts_hoprwrapper.hoprwrapperinterface.md#encodefunctiondata)
- [encodeFunctionResult](contracts_hoprwrapper.hoprwrapperinterface.md#encodefunctionresult)
- [format](contracts_hoprwrapper.hoprwrapperinterface.md#format)
- [getEvent](contracts_hoprwrapper.hoprwrapperinterface.md#getevent)
- [getEventTopic](contracts_hoprwrapper.hoprwrapperinterface.md#geteventtopic)
- [getFunction](contracts_hoprwrapper.hoprwrapperinterface.md#getfunction)
- [getSighash](contracts_hoprwrapper.hoprwrapperinterface.md#getsighash)
- [parseLog](contracts_hoprwrapper.hoprwrapperinterface.md#parselog)
- [parseTransaction](contracts_hoprwrapper.hoprwrapperinterface.md#parsetransaction)

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

| Name                                    | Type            |
| :-------------------------------------- | :-------------- |
| `OwnershipTransferred(address,address)` | _EventFragment_ |
| `Unwrapped(address,uint256)`            | _EventFragment_ |
| `Wrapped(address,uint256)`              | _EventFragment_ |

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:109

---

### fragments

• `Readonly` **fragments**: readonly _Fragment_[]

Inherited from: ethers.utils.Interface.fragments

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:29

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                                          | Type               |
| :------------------------------------------------------------ | :----------------- |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | _FunctionFragment_ |
| `canImplementInterfaceForAddress(bytes32,address)`            | _FunctionFragment_ |
| `onTokenTransfer(address,uint256,bytes)`                      | _FunctionFragment_ |
| `owner()`                                                     | _FunctionFragment_ |
| `recoverTokens()`                                             | _FunctionFragment_ |
| `renounceOwnership()`                                         | _FunctionFragment_ |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | _FunctionFragment_ |
| `transferOwnership(address)`                                  | _FunctionFragment_ |
| `wxHOPR()`                                                    | _FunctionFragment_ |
| `xHOPR()`                                                     | _FunctionFragment_ |
| `xHoprAmount()`                                               | _FunctionFragment_ |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:23

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

▸ **decodeFunctionResult**(`functionFragment`: `"TOKENS_RECIPIENT_INTERFACE_HASH"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"TOKENS_RECIPIENT_INTERFACE_HASH"` |
| `data`             | BytesLike                           |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:73

▸ **decodeFunctionResult**(`functionFragment`: `"canImplementInterfaceForAddress"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"canImplementInterfaceForAddress"` |
| `data`             | BytesLike                           |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:77

▸ **decodeFunctionResult**(`functionFragment`: `"onTokenTransfer"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                |
| :----------------- | :------------------ |
| `functionFragment` | `"onTokenTransfer"` |
| `data`             | BytesLike           |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:81

▸ **decodeFunctionResult**(`functionFragment`: `"owner"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"owner"` |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:85

▸ **decodeFunctionResult**(`functionFragment`: `"recoverTokens"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"recoverTokens"` |
| `data`             | BytesLike         |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:86

▸ **decodeFunctionResult**(`functionFragment`: `"renounceOwnership"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"renounceOwnership"` |
| `data`             | BytesLike             |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:90

▸ **decodeFunctionResult**(`functionFragment`: `"tokensReceived"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type               |
| :----------------- | :----------------- |
| `functionFragment` | `"tokensReceived"` |
| `data`             | BytesLike          |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:94

▸ **decodeFunctionResult**(`functionFragment`: `"transferOwnership"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"transferOwnership"` |
| `data`             | BytesLike             |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:98

▸ **decodeFunctionResult**(`functionFragment`: `"wxHOPR"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type       |
| :----------------- | :--------- |
| `functionFragment` | `"wxHOPR"` |
| `data`             | BytesLike  |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:102

▸ **decodeFunctionResult**(`functionFragment`: `"xHOPR"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"xHOPR"` |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:103

▸ **decodeFunctionResult**(`functionFragment`: `"xHoprAmount"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"xHoprAmount"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:104

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

▸ **encodeFunctionData**(`functionFragment`: `"TOKENS_RECIPIENT_INTERFACE_HASH"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"TOKENS_RECIPIENT_INTERFACE_HASH"` |
| `values?`          | _undefined_                         |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:37

▸ **encodeFunctionData**(`functionFragment`: `"canImplementInterfaceForAddress"`, `values`: [BytesLike, *string*]): _string_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"canImplementInterfaceForAddress"` |
| `values`           | [BytesLike, *string*]               |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:41

▸ **encodeFunctionData**(`functionFragment`: `"onTokenTransfer"`, `values`: [*string*, BigNumberish, BytesLike]): _string_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"onTokenTransfer"`                 |
| `values`           | [*string*, BigNumberish, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:45

▸ **encodeFunctionData**(`functionFragment`: `"owner"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"owner"`   |
| `values?`          | _undefined_ |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:49

▸ **encodeFunctionData**(`functionFragment`: `"recoverTokens"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"recoverTokens"` |
| `values?`          | _undefined_       |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:50

▸ **encodeFunctionData**(`functionFragment`: `"renounceOwnership"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"renounceOwnership"` |
| `values?`          | _undefined_           |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:54

▸ **encodeFunctionData**(`functionFragment`: `"tokensReceived"`, `values`: [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike]): _string_

#### Parameters

| Name               | Type                                                               |
| :----------------- | :----------------------------------------------------------------- |
| `functionFragment` | `"tokensReceived"`                                                 |
| `values`           | [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:58

▸ **encodeFunctionData**(`functionFragment`: `"transferOwnership"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"transferOwnership"` |
| `values`           | [*string*]            |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:62

▸ **encodeFunctionData**(`functionFragment`: `"wxHOPR"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"wxHOPR"`  |
| `values?`          | _undefined_ |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:66

▸ **encodeFunctionData**(`functionFragment`: `"xHOPR"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"xHOPR"`   |
| `values?`          | _undefined_ |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:67

▸ **encodeFunctionData**(`functionFragment`: `"xHoprAmount"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"xHoprAmount"` |
| `values?`          | _undefined_     |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:68

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

▸ **getEvent**(`nameOrSignatureOrTopic`: `"OwnershipTransferred"`): _EventFragment_

#### Parameters

| Name                     | Type                     |
| :----------------------- | :----------------------- |
| `nameOrSignatureOrTopic` | `"OwnershipTransferred"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:115

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Unwrapped"`): _EventFragment_

#### Parameters

| Name                     | Type          |
| :----------------------- | :------------ |
| `nameOrSignatureOrTopic` | `"Unwrapped"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:116

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Wrapped"`): _EventFragment_

#### Parameters

| Name                     | Type        |
| :----------------------- | :---------- |
| `nameOrSignatureOrTopic` | `"Wrapped"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:117

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
