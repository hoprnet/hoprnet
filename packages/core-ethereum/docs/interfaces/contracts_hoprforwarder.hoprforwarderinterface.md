[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprForwarder](../modules/contracts_hoprforwarder.md) / HoprForwarderInterface

# Interface: HoprForwarderInterface

[contracts/HoprForwarder](../modules/contracts_hoprforwarder.md).HoprForwarderInterface

## Hierarchy

- _Interface_

  ↳ **HoprForwarderInterface**

## Table of contents

### Properties

- [\_abiCoder](contracts_hoprforwarder.hoprforwarderinterface.md#_abicoder)
- [\_isInterface](contracts_hoprforwarder.hoprforwarderinterface.md#_isinterface)
- [deploy](contracts_hoprforwarder.hoprforwarderinterface.md#deploy)
- [errors](contracts_hoprforwarder.hoprforwarderinterface.md#errors)
- [events](contracts_hoprforwarder.hoprforwarderinterface.md#events)
- [fragments](contracts_hoprforwarder.hoprforwarderinterface.md#fragments)
- [functions](contracts_hoprforwarder.hoprforwarderinterface.md#functions)
- [structs](contracts_hoprforwarder.hoprforwarderinterface.md#structs)

### Methods

- [\_decodeParams](contracts_hoprforwarder.hoprforwarderinterface.md#_decodeparams)
- [\_encodeParams](contracts_hoprforwarder.hoprforwarderinterface.md#_encodeparams)
- [decodeEventLog](contracts_hoprforwarder.hoprforwarderinterface.md#decodeeventlog)
- [decodeFunctionData](contracts_hoprforwarder.hoprforwarderinterface.md#decodefunctiondata)
- [decodeFunctionResult](contracts_hoprforwarder.hoprforwarderinterface.md#decodefunctionresult)
- [encodeDeploy](contracts_hoprforwarder.hoprforwarderinterface.md#encodedeploy)
- [encodeEventLog](contracts_hoprforwarder.hoprforwarderinterface.md#encodeeventlog)
- [encodeFilterTopics](contracts_hoprforwarder.hoprforwarderinterface.md#encodefiltertopics)
- [encodeFunctionData](contracts_hoprforwarder.hoprforwarderinterface.md#encodefunctiondata)
- [encodeFunctionResult](contracts_hoprforwarder.hoprforwarderinterface.md#encodefunctionresult)
- [format](contracts_hoprforwarder.hoprforwarderinterface.md#format)
- [getEvent](contracts_hoprforwarder.hoprforwarderinterface.md#getevent)
- [getEventTopic](contracts_hoprforwarder.hoprforwarderinterface.md#geteventtopic)
- [getFunction](contracts_hoprforwarder.hoprforwarderinterface.md#getfunction)
- [getSighash](contracts_hoprforwarder.hoprforwarderinterface.md#getsighash)
- [parseLog](contracts_hoprforwarder.hoprforwarderinterface.md#parselog)
- [parseTransaction](contracts_hoprforwarder.hoprforwarderinterface.md#parsetransaction)

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

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:73

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
| `ERC1820_REGISTRY()`                                          | _FunctionFragment_ |
| `HOPR_TOKEN()`                                                | _FunctionFragment_ |
| `MULTISIG()`                                                  | _FunctionFragment_ |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | _FunctionFragment_ |
| `recoverTokens(address)`                                      | _FunctionFragment_ |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | _FunctionFragment_ |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:23

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

▸ **decodeFunctionResult**(`functionFragment`: `"ERC1820_REGISTRY"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"ERC1820_REGISTRY"` |
| `data`             | BytesLike            |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:54

▸ **decodeFunctionResult**(`functionFragment`: `"HOPR_TOKEN"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type           |
| :----------------- | :------------- |
| `functionFragment` | `"HOPR_TOKEN"` |
| `data`             | BytesLike      |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:58

▸ **decodeFunctionResult**(`functionFragment`: `"MULTISIG"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"MULTISIG"` |
| `data`             | BytesLike    |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:59

▸ **decodeFunctionResult**(`functionFragment`: `"TOKENS_RECIPIENT_INTERFACE_HASH"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"TOKENS_RECIPIENT_INTERFACE_HASH"` |
| `data`             | BytesLike                           |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:60

▸ **decodeFunctionResult**(`functionFragment`: `"recoverTokens"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"recoverTokens"` |
| `data`             | BytesLike         |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:64

▸ **decodeFunctionResult**(`functionFragment`: `"tokensReceived"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type               |
| :----------------- | :----------------- |
| `functionFragment` | `"tokensReceived"` |
| `data`             | BytesLike          |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:68

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

▸ **encodeFunctionData**(`functionFragment`: `"ERC1820_REGISTRY"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"ERC1820_REGISTRY"` |
| `values?`          | _undefined_          |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:32

▸ **encodeFunctionData**(`functionFragment`: `"HOPR_TOKEN"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type           |
| :----------------- | :------------- |
| `functionFragment` | `"HOPR_TOKEN"` |
| `values?`          | _undefined_    |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:36

▸ **encodeFunctionData**(`functionFragment`: `"MULTISIG"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"MULTISIG"` |
| `values?`          | _undefined_  |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:40

▸ **encodeFunctionData**(`functionFragment`: `"TOKENS_RECIPIENT_INTERFACE_HASH"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"TOKENS_RECIPIENT_INTERFACE_HASH"` |
| `values?`          | _undefined_                         |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:41

▸ **encodeFunctionData**(`functionFragment`: `"recoverTokens"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type              |
| :----------------- | :---------------- |
| `functionFragment` | `"recoverTokens"` |
| `values`           | [*string*]        |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:45

▸ **encodeFunctionData**(`functionFragment`: `"tokensReceived"`, `values`: [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike]): _string_

#### Parameters

| Name               | Type                                                               |
| :----------------- | :----------------------------------------------------------------- |
| `functionFragment` | `"tokensReceived"`                                                 |
| `values`           | [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:49

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

▸ **getEvent**(`nameOrSignatureOrTopic`: _string_): _EventFragment_

#### Parameters

| Name                     | Type     |
| :----------------------- | :------- |
| `nameOrSignatureOrTopic` | _string_ |

**Returns:** _EventFragment_

Inherited from: ethers.utils.Interface.getEvent

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:52

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
