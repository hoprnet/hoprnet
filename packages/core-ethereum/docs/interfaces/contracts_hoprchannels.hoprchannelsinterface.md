[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprChannels](../modules/contracts_hoprchannels.md) / HoprChannelsInterface

# Interface: HoprChannelsInterface

[contracts/HoprChannels](../modules/contracts_hoprchannels.md).HoprChannelsInterface

## Hierarchy

- *Interface*

  ↳ **HoprChannelsInterface**

## Table of contents

### Properties

- [\_abiCoder](contracts_hoprchannels.hoprchannelsinterface.md#_abicoder)
- [\_isInterface](contracts_hoprchannels.hoprchannelsinterface.md#_isinterface)
- [deploy](contracts_hoprchannels.hoprchannelsinterface.md#deploy)
- [errors](contracts_hoprchannels.hoprchannelsinterface.md#errors)
- [events](contracts_hoprchannels.hoprchannelsinterface.md#events)
- [fragments](contracts_hoprchannels.hoprchannelsinterface.md#fragments)
- [functions](contracts_hoprchannels.hoprchannelsinterface.md#functions)
- [structs](contracts_hoprchannels.hoprchannelsinterface.md#structs)

### Methods

- [\_decodeParams](contracts_hoprchannels.hoprchannelsinterface.md#_decodeparams)
- [\_encodeParams](contracts_hoprchannels.hoprchannelsinterface.md#_encodeparams)
- [decodeEventLog](contracts_hoprchannels.hoprchannelsinterface.md#decodeeventlog)
- [decodeFunctionData](contracts_hoprchannels.hoprchannelsinterface.md#decodefunctiondata)
- [decodeFunctionResult](contracts_hoprchannels.hoprchannelsinterface.md#decodefunctionresult)
- [encodeDeploy](contracts_hoprchannels.hoprchannelsinterface.md#encodedeploy)
- [encodeEventLog](contracts_hoprchannels.hoprchannelsinterface.md#encodeeventlog)
- [encodeFilterTopics](contracts_hoprchannels.hoprchannelsinterface.md#encodefiltertopics)
- [encodeFunctionData](contracts_hoprchannels.hoprchannelsinterface.md#encodefunctiondata)
- [encodeFunctionResult](contracts_hoprchannels.hoprchannelsinterface.md#encodefunctionresult)
- [format](contracts_hoprchannels.hoprchannelsinterface.md#format)
- [getEvent](contracts_hoprchannels.hoprchannelsinterface.md#getevent)
- [getEventTopic](contracts_hoprchannels.hoprchannelsinterface.md#geteventtopic)
- [getFunction](contracts_hoprchannels.hoprchannelsinterface.md#getfunction)
- [getSighash](contracts_hoprchannels.hoprchannelsinterface.md#getsighash)
- [parseLog](contracts_hoprchannels.hoprchannelsinterface.md#parselog)
- [parseTransaction](contracts_hoprchannels.hoprchannelsinterface.md#parsetransaction)

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
| `Announcement(address,bytes)` | *EventFragment* |
| `ChannelUpdate(address,address,tuple)` | *EventFragment* |

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:145

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
| `FUND_CHANNEL_MULTI_SIZE()` | *FunctionFragment* |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | *FunctionFragment* |
| `announce(bytes)` | *FunctionFragment* |
| `bumpChannel(address,bytes32)` | *FunctionFragment* |
| `canImplementInterfaceForAddress(bytes32,address)` | *FunctionFragment* |
| `channels(bytes32)` | *FunctionFragment* |
| `computeChallenge(bytes32)` | *FunctionFragment* |
| `finalizeChannelClosure(address)` | *FunctionFragment* |
| `fundChannelMulti(address,address,uint256,uint256)` | *FunctionFragment* |
| `initiateChannelClosure(address)` | *FunctionFragment* |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | *FunctionFragment* |
| `secsClosure()` | *FunctionFragment* |
| `token()` | *FunctionFragment* |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | *FunctionFragment* |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:23

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

▸ **decodeFunctionResult**(`functionFragment`: ``"FUND_CHANNEL_MULTI_SIZE"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"FUND_CHANNEL_MULTI_SIZE"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:97

▸ **decodeFunctionResult**(`functionFragment`: ``"TOKENS_RECIPIENT_INTERFACE_HASH"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"TOKENS_RECIPIENT_INTERFACE_HASH"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:101

▸ **decodeFunctionResult**(`functionFragment`: ``"announce"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"announce"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:105

▸ **decodeFunctionResult**(`functionFragment`: ``"bumpChannel"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"bumpChannel"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:106

▸ **decodeFunctionResult**(`functionFragment`: ``"canImplementInterfaceForAddress"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"canImplementInterfaceForAddress"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:110

▸ **decodeFunctionResult**(`functionFragment`: ``"channels"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"channels"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:114

▸ **decodeFunctionResult**(`functionFragment`: ``"computeChallenge"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"computeChallenge"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:115

▸ **decodeFunctionResult**(`functionFragment`: ``"finalizeChannelClosure"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"finalizeChannelClosure"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:119

▸ **decodeFunctionResult**(`functionFragment`: ``"fundChannelMulti"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"fundChannelMulti"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:123

▸ **decodeFunctionResult**(`functionFragment`: ``"initiateChannelClosure"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"initiateChannelClosure"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:127

▸ **decodeFunctionResult**(`functionFragment`: ``"redeemTicket"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"redeemTicket"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:131

▸ **decodeFunctionResult**(`functionFragment`: ``"secsClosure"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"secsClosure"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:135

▸ **decodeFunctionResult**(`functionFragment`: ``"token"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"token"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:139

▸ **decodeFunctionResult**(`functionFragment`: ``"tokensReceived"``, `data`: BytesLike): *Result*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"tokensReceived"`` |
| `data` | BytesLike |

**Returns:** *Result*

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:140

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

▸ **encodeFunctionData**(`functionFragment`: ``"FUND_CHANNEL_MULTI_SIZE"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"FUND_CHANNEL_MULTI_SIZE"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:40

▸ **encodeFunctionData**(`functionFragment`: ``"TOKENS_RECIPIENT_INTERFACE_HASH"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"TOKENS_RECIPIENT_INTERFACE_HASH"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:44

▸ **encodeFunctionData**(`functionFragment`: ``"announce"``, `values`: [BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"announce"`` |
| `values` | [BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:48

▸ **encodeFunctionData**(`functionFragment`: ``"bumpChannel"``, `values`: [*string*, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"bumpChannel"`` |
| `values` | [*string*, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:49

▸ **encodeFunctionData**(`functionFragment`: ``"canImplementInterfaceForAddress"``, `values`: [BytesLike, *string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"canImplementInterfaceForAddress"`` |
| `values` | [BytesLike, *string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:53

▸ **encodeFunctionData**(`functionFragment`: ``"channels"``, `values`: [BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"channels"`` |
| `values` | [BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:57

▸ **encodeFunctionData**(`functionFragment`: ``"computeChallenge"``, `values`: [BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"computeChallenge"`` |
| `values` | [BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:58

▸ **encodeFunctionData**(`functionFragment`: ``"finalizeChannelClosure"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"finalizeChannelClosure"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:62

▸ **encodeFunctionData**(`functionFragment`: ``"fundChannelMulti"``, `values`: [*string*, *string*, BigNumberish, BigNumberish]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"fundChannelMulti"`` |
| `values` | [*string*, *string*, BigNumberish, BigNumberish] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:66

▸ **encodeFunctionData**(`functionFragment`: ``"initiateChannelClosure"``, `values`: [*string*]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"initiateChannelClosure"`` |
| `values` | [*string*] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:70

▸ **encodeFunctionData**(`functionFragment`: ``"redeemTicket"``, `values`: [*string*, BytesLike, BigNumberish, BigNumberish, BytesLike, BigNumberish, BigNumberish, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"redeemTicket"`` |
| `values` | [*string*, BytesLike, BigNumberish, BigNumberish, BytesLike, BigNumberish, BigNumberish, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:74

▸ **encodeFunctionData**(`functionFragment`: ``"secsClosure"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"secsClosure"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:87

▸ **encodeFunctionData**(`functionFragment`: ``"token"``, `values?`: *undefined*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"token"`` |
| `values?` | *undefined* |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:91

▸ **encodeFunctionData**(`functionFragment`: ``"tokensReceived"``, `values`: [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `functionFragment` | ``"tokensReceived"`` |
| `values` | [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** *string*

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:92

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

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"Announcement"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"Announcement"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:150

▸ **getEvent**(`nameOrSignatureOrTopic`: ``"ChannelUpdate"``): *EventFragment*

#### Parameters

| Name | Type |
| :------ | :------ |
| `nameOrSignatureOrTopic` | ``"ChannelUpdate"`` |

**Returns:** *EventFragment*

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:151

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
