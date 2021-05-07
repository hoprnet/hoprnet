[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprWrapper](../modules/contracts_hoprwrapper.md) / HoprWrapperInterface

# Interface: HoprWrapperInterface

[contracts/HoprWrapper](../modules/contracts_hoprwrapper.md).HoprWrapperInterface

## Hierarchy

- *Interface*

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
| `OwnershipTransferred(address,address)` | *EventFragment* |
| `Unwrapped(address,uint256)` | *EventFragment* |
| `Wrapped(address,uint256)` | *EventFragment* |

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:109

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
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | *FunctionFragment* |
| `canImplementInterfaceForAddress(bytes32,address)` | *FunctionFragment* |
| `onTokenTransfer(address,uint256,bytes)` | *FunctionFragment* |
| `owner()` | *FunctionFragment* |
| `recoverTokens()` | *FunctionFragment* |
| `renounceOwnership()` | *FunctionFragment* |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | *FunctionFragment* |
| `transferOwnership(address)` | *FunctionFragment* |
| `wxHOPR()` | *FunctionFragment* |
| `xHOPR()` | *FunctionFragment* |
| `xHoprAmount()` | *FunctionFragment* |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:23

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

▸ **decodeFunctionResult**(`functionFragment`: ``"TOKENS_RECIPIENT_INTERFACE_HASH"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"TOKENS_RECIPIENT_INTERFACE_HASH"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:73

▸ **decodeFunctionResult**(`functionFragment`: ``"canImplementInterfaceForAddress"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"canImplementInterfaceForAddress"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:77

▸ **decodeFunctionResult**(`functionFragment`: ``"onTokenTransfer"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"onTokenTransfer"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:81

▸ **decodeFunctionResult**(`functionFragment`: ``"owner"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"owner"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:85

▸ **decodeFunctionResult**(`functionFragment`: ``"recoverTokens"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"recoverTokens"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:86

▸ **decodeFunctionResult**(`functionFragment`: ``"renounceOwnership"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"renounceOwnership"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:90

▸ **decodeFunctionResult**(`functionFragment`: ``"tokensReceived"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"tokensReceived"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:94

▸ **decodeFunctionResult**(`functionFragment`: ``"transferOwnership"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"transferOwnership"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:98

▸ **decodeFunctionResult**(`functionFragment`: ``"wxHOPR"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"wxHOPR"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:102

▸ **decodeFunctionResult**(`functionFragment`: ``"xHOPR"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"xHOPR"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:103

▸ **decodeFunctionResult**(`functionFragment`: ``"xHoprAmount"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"xHoprAmount"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:104

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

▸ **encodeFunctionData**(`functionFragment`: ``"TOKENS_RECIPIENT_INTERFACE_HASH"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"TOKENS_RECIPIENT_INTERFACE_HASH"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:37

▸ **encodeFunctionData**(`functionFragment`: ``"canImplementInterfaceForAddress"``, `values`: [BytesLike, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"canImplementInterfaceForAddress"`` |
| `values` | [BytesLike, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:41

▸ **encodeFunctionData**(`functionFragment`: ``"onTokenTransfer"``, `values`: [*string*, BigNumberish, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"onTokenTransfer"`` |
| `values` | [*string*, BigNumberish, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:45

▸ **encodeFunctionData**(`functionFragment`: ``"owner"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"owner"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:49

▸ **encodeFunctionData**(`functionFragment`: ``"recoverTokens"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"recoverTokens"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:50

▸ **encodeFunctionData**(`functionFragment`: ``"renounceOwnership"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"renounceOwnership"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:54

▸ **encodeFunctionData**(`functionFragment`: ``"tokensReceived"``, `values`: [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"tokensReceived"`` |
| `values` | [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:58

▸ **encodeFunctionData**(`functionFragment`: ``"transferOwnership"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"transferOwnership"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:62

▸ **encodeFunctionData**(`functionFragment`: ``"wxHOPR"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"wxHOPR"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:66

▸ **encodeFunctionData**(`functionFragment`: ``"xHOPR"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"xHOPR"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:67

▸ **encodeFunctionData**(`functionFragment`: ``"xHoprAmount"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"xHoprAmount"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:68

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

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"OwnershipTransferred"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"OwnershipTransferred"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:115

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Unwrapped"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Unwrapped"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:116

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Wrapped"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Wrapped"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:117

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
