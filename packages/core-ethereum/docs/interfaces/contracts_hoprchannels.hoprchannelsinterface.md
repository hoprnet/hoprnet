[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprChannels](../modules/contracts_hoprchannels.md) / HoprChannelsInterface

# Interface: HoprChannelsInterface

[contracts/HoprChannels](../modules/contracts_hoprchannels.md).HoprChannelsInterface

## Hierarchy

- _Interface_

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

| Name                                   | Type            |
| :------------------------------------- | :-------------- |
| `Announcement(address,bytes)`          | _EventFragment_ |
| `ChannelUpdate(address,address,tuple)` | _EventFragment_ |

Overrides: ethers.utils.Interface.events

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:145

---

### fragments

• `Readonly` **fragments**: readonly _Fragment_[]

Inherited from: ethers.utils.Interface.fragments

Defined in: node_modules/@ethersproject/abi/lib/interface.d.ts:29

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                                                          | Type               |
| :---------------------------------------------------------------------------- | :----------------- |
| `FUND_CHANNEL_MULTI_SIZE()`                                                   | _FunctionFragment_ |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                                           | _FunctionFragment_ |
| `announce(bytes)`                                                             | _FunctionFragment_ |
| `bumpChannel(address,bytes32)`                                                | _FunctionFragment_ |
| `canImplementInterfaceForAddress(bytes32,address)`                            | _FunctionFragment_ |
| `channels(bytes32)`                                                           | _FunctionFragment_ |
| `computeChallenge(bytes32)`                                                   | _FunctionFragment_ |
| `finalizeChannelClosure(address)`                                             | _FunctionFragment_ |
| `fundChannelMulti(address,address,uint256,uint256)`                           | _FunctionFragment_ |
| `initiateChannelClosure(address)`                                             | _FunctionFragment_ |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | _FunctionFragment_ |
| `secsClosure()`                                                               | _FunctionFragment_ |
| `token()`                                                                     | _FunctionFragment_ |
| `tokensReceived(address,address,address,uint256,bytes,bytes)`                 | _FunctionFragment_ |

Overrides: ethers.utils.Interface.functions

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:23

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

▸ **decodeFunctionResult**(`functionFragment`: `"FUND_CHANNEL_MULTI_SIZE"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                        |
| :----------------- | :-------------------------- |
| `functionFragment` | `"FUND_CHANNEL_MULTI_SIZE"` |
| `data`             | BytesLike                   |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:97

▸ **decodeFunctionResult**(`functionFragment`: `"TOKENS_RECIPIENT_INTERFACE_HASH"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"TOKENS_RECIPIENT_INTERFACE_HASH"` |
| `data`             | BytesLike                           |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:101

▸ **decodeFunctionResult**(`functionFragment`: `"announce"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"announce"` |
| `data`             | BytesLike    |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:105

▸ **decodeFunctionResult**(`functionFragment`: `"bumpChannel"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"bumpChannel"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:106

▸ **decodeFunctionResult**(`functionFragment`: `"canImplementInterfaceForAddress"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"canImplementInterfaceForAddress"` |
| `data`             | BytesLike                           |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:110

▸ **decodeFunctionResult**(`functionFragment`: `"channels"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"channels"` |
| `data`             | BytesLike    |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:114

▸ **decodeFunctionResult**(`functionFragment`: `"computeChallenge"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"computeChallenge"` |
| `data`             | BytesLike            |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:115

▸ **decodeFunctionResult**(`functionFragment`: `"finalizeChannelClosure"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                       |
| :----------------- | :------------------------- |
| `functionFragment` | `"finalizeChannelClosure"` |
| `data`             | BytesLike                  |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:119

▸ **decodeFunctionResult**(`functionFragment`: `"fundChannelMulti"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"fundChannelMulti"` |
| `data`             | BytesLike            |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:123

▸ **decodeFunctionResult**(`functionFragment`: `"initiateChannelClosure"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type                       |
| :----------------- | :------------------------- |
| `functionFragment` | `"initiateChannelClosure"` |
| `data`             | BytesLike                  |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:127

▸ **decodeFunctionResult**(`functionFragment`: `"redeemTicket"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type             |
| :----------------- | :--------------- |
| `functionFragment` | `"redeemTicket"` |
| `data`             | BytesLike        |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:131

▸ **decodeFunctionResult**(`functionFragment`: `"secsClosure"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"secsClosure"` |
| `data`             | BytesLike       |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:135

▸ **decodeFunctionResult**(`functionFragment`: `"token"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `functionFragment` | `"token"` |
| `data`             | BytesLike |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:139

▸ **decodeFunctionResult**(`functionFragment`: `"tokensReceived"`, `data`: BytesLike): _Result_

#### Parameters

| Name               | Type               |
| :----------------- | :----------------- |
| `functionFragment` | `"tokensReceived"` |
| `data`             | BytesLike          |

**Returns:** _Result_

Overrides: ethers.utils.Interface.decodeFunctionResult

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:140

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

▸ **encodeFunctionData**(`functionFragment`: `"FUND_CHANNEL_MULTI_SIZE"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                        |
| :----------------- | :-------------------------- |
| `functionFragment` | `"FUND_CHANNEL_MULTI_SIZE"` |
| `values?`          | _undefined_                 |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:40

▸ **encodeFunctionData**(`functionFragment`: `"TOKENS_RECIPIENT_INTERFACE_HASH"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"TOKENS_RECIPIENT_INTERFACE_HASH"` |
| `values?`          | _undefined_                         |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:44

▸ **encodeFunctionData**(`functionFragment`: `"announce"`, `values`: [BytesLike]): _string_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"announce"` |
| `values`           | [BytesLike]  |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:48

▸ **encodeFunctionData**(`functionFragment`: `"bumpChannel"`, `values`: [*string*, BytesLike]): _string_

#### Parameters

| Name               | Type                  |
| :----------------- | :-------------------- |
| `functionFragment` | `"bumpChannel"`       |
| `values`           | [*string*, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:49

▸ **encodeFunctionData**(`functionFragment`: `"canImplementInterfaceForAddress"`, `values`: [BytesLike, *string*]): _string_

#### Parameters

| Name               | Type                                |
| :----------------- | :---------------------------------- |
| `functionFragment` | `"canImplementInterfaceForAddress"` |
| `values`           | [BytesLike, *string*]               |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:53

▸ **encodeFunctionData**(`functionFragment`: `"channels"`, `values`: [BytesLike]): _string_

#### Parameters

| Name               | Type         |
| :----------------- | :----------- |
| `functionFragment` | `"channels"` |
| `values`           | [BytesLike]  |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:57

▸ **encodeFunctionData**(`functionFragment`: `"computeChallenge"`, `values`: [BytesLike]): _string_

#### Parameters

| Name               | Type                 |
| :----------------- | :------------------- |
| `functionFragment` | `"computeChallenge"` |
| `values`           | [BytesLike]          |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:58

▸ **encodeFunctionData**(`functionFragment`: `"finalizeChannelClosure"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type                       |
| :----------------- | :------------------------- |
| `functionFragment` | `"finalizeChannelClosure"` |
| `values`           | [*string*]                 |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:62

▸ **encodeFunctionData**(`functionFragment`: `"fundChannelMulti"`, `values`: [*string*, *string*, BigNumberish, BigNumberish]): _string_

#### Parameters

| Name               | Type                                             |
| :----------------- | :----------------------------------------------- |
| `functionFragment` | `"fundChannelMulti"`                             |
| `values`           | [*string*, *string*, BigNumberish, BigNumberish] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:66

▸ **encodeFunctionData**(`functionFragment`: `"initiateChannelClosure"`, `values`: [*string*]): _string_

#### Parameters

| Name               | Type                       |
| :----------------- | :------------------------- |
| `functionFragment` | `"initiateChannelClosure"` |
| `values`           | [*string*]                 |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:70

▸ **encodeFunctionData**(`functionFragment`: `"redeemTicket"`, `values`: [*string*, BytesLike, BigNumberish, BigNumberish, BytesLike, BigNumberish, BigNumberish, BytesLike]): _string_

#### Parameters

| Name               | Type                                                                                                |
| :----------------- | :-------------------------------------------------------------------------------------------------- |
| `functionFragment` | `"redeemTicket"`                                                                                    |
| `values`           | [*string*, BytesLike, BigNumberish, BigNumberish, BytesLike, BigNumberish, BigNumberish, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:74

▸ **encodeFunctionData**(`functionFragment`: `"secsClosure"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type            |
| :----------------- | :-------------- |
| `functionFragment` | `"secsClosure"` |
| `values?`          | _undefined_     |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:87

▸ **encodeFunctionData**(`functionFragment`: `"token"`, `values?`: _undefined_): _string_

#### Parameters

| Name               | Type        |
| :----------------- | :---------- |
| `functionFragment` | `"token"`   |
| `values?`          | _undefined_ |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:91

▸ **encodeFunctionData**(`functionFragment`: `"tokensReceived"`, `values`: [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike]): _string_

#### Parameters

| Name               | Type                                                               |
| :----------------- | :----------------------------------------------------------------- |
| `functionFragment` | `"tokensReceived"`                                                 |
| `values`           | [*string*, *string*, *string*, BigNumberish, BytesLike, BytesLike] |

**Returns:** _string_

Overrides: ethers.utils.Interface.encodeFunctionData

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:92

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

▸ **getEvent**(`nameOrSignatureOrTopic`: `"Announcement"`): _EventFragment_

#### Parameters

| Name                     | Type             |
| :----------------------- | :--------------- |
| `nameOrSignatureOrTopic` | `"Announcement"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:150

▸ **getEvent**(`nameOrSignatureOrTopic`: `"ChannelUpdate"`): _EventFragment_

#### Parameters

| Name                     | Type              |
| :----------------------- | :---------------- |
| `nameOrSignatureOrTopic` | `"ChannelUpdate"` |

**Returns:** _EventFragment_

Overrides: ethers.utils.Interface.getEvent

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:151

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
