[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprForwarder](../modules/contracts_hoprforwarder.md) / HoprForwarderInterface

# Interface: HoprForwarderInterface

[contracts/HoprForwarder](../modules/contracts_hoprforwarder.md).HoprForwarderInterface

## Hierarchy

- *Interface*

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

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:73

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
| `ERC1820_REGISTRY()` | *FunctionFragment* |
| `HOPR_TOKEN()` | *FunctionFragment* |
| `MULTISIG()` | *FunctionFragment* |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | *FunctionFragment* |
| `recoverTokens(address)` | *FunctionFragment* |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | *FunctionFragment* |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:23

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

▸ **decodeFunctionResult**(`functionFragment`: ``"ERC1820_REGISTRY"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"ERC1820_REGISTRY"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:54

▸ **decodeFunctionResult**(`functionFragment`: ``"HOPR_TOKEN"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"HOPR_TOKEN"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:58

▸ **decodeFunctionResult**(`functionFragment`: ``"MULTISIG"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"MULTISIG"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:59

▸ **decodeFunctionResult**(`functionFragment`: ``"TOKENS_RECIPIENT_INTERFACE_HASH"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"TOKENS_RECIPIENT_INTERFACE_HASH"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:60

▸ **decodeFunctionResult**(`functionFragment`: ``"recoverTokens"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"recoverTokens"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:64

▸ **decodeFunctionResult**(`functionFragment`: ``"tokensReceived"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"tokensReceived"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:68

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

▸ **encodeFunctionData**(`functionFragment`: ``"ERC1820_REGISTRY"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"ERC1820_REGISTRY"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:32

▸ **encodeFunctionData**(`functionFragment`: ``"HOPR_TOKEN"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"HOPR_TOKEN"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:36

▸ **encodeFunctionData**(`functionFragment`: ``"MULTISIG"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"MULTISIG"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:40

▸ **encodeFunctionData**(`functionFragment`: ``"TOKENS_RECIPIENT_INTERFACE_HASH"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"TOKENS_RECIPIENT_INTERFACE_HASH"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:41

▸ **encodeFunctionData**(`functionFragment`: ``"recoverTokens"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"recoverTokens"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:45

▸ **encodeFunctionData**(`functionFragment`: ``"tokensReceived"``, `values`: [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"tokensReceived"`` |
| `values` | [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:49

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

▸ **getEvent**(`nameOrSignatureOrTopic`: *string*): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | *string* |

**Returns:** *EventFragment*

Inherited from: ethers.utils.Interface.getEvent

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:52

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
