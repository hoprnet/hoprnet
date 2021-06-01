[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC777Sender

# Class: IERC777Sender

## Hierarchy

- *Contract*

  ↳ **IERC777Sender**

## Table of contents

### Constructors

- [constructor](ierc777sender.md#constructor)

### Properties

- [\_deployedPromise](ierc777sender.md#_deployedpromise)
- [\_runningEvents](ierc777sender.md#_runningevents)
- [\_wrappedEmits](ierc777sender.md#_wrappedemits)
- [address](ierc777sender.md#address)
- [callStatic](ierc777sender.md#callstatic)
- [deployTransaction](ierc777sender.md#deploytransaction)
- [estimateGas](ierc777sender.md#estimategas)
- [filters](ierc777sender.md#filters)
- [functions](ierc777sender.md#functions)
- [interface](ierc777sender.md#interface)
- [populateTransaction](ierc777sender.md#populatetransaction)
- [provider](ierc777sender.md#provider)
- [resolvedAddress](ierc777sender.md#resolvedaddress)
- [signer](ierc777sender.md#signer)

### Methods

- [\_checkRunningEvents](ierc777sender.md#_checkrunningevents)
- [\_deployed](ierc777sender.md#_deployed)
- [\_wrapEvent](ierc777sender.md#_wrapevent)
- [attach](ierc777sender.md#attach)
- [connect](ierc777sender.md#connect)
- [deployed](ierc777sender.md#deployed)
- [emit](ierc777sender.md#emit)
- [fallback](ierc777sender.md#fallback)
- [listenerCount](ierc777sender.md#listenercount)
- [listeners](ierc777sender.md#listeners)
- [off](ierc777sender.md#off)
- [on](ierc777sender.md#on)
- [once](ierc777sender.md#once)
- [queryFilter](ierc777sender.md#queryfilter)
- [removeAllListeners](ierc777sender.md#removealllisteners)
- [removeListener](ierc777sender.md#removelistener)
- [tokensToSend](ierc777sender.md#tokenstosend)
- [tokensToSend(address,address,address,uint256,bytes,bytes)](ierc777sender.md#tokenstosend(address,address,address,uint256,bytes,bytes))
- [getContractAddress](ierc777sender.md#getcontractaddress)
- [getInterface](ierc777sender.md#getinterface)
- [isIndexed](ierc777sender.md#isindexed)

## Constructors

### constructor

\+ **new IERC777Sender**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*IERC777Sender*](ierc777sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Inherited from: Contract.constructor

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:98

## Properties

### \_deployedPromise

• **\_deployedPromise**: *Promise*<Contract\>

Inherited from: Contract.\_deployedPromise

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:92

___

### \_runningEvents

• **\_runningEvents**: *object*

#### Type declaration

Inherited from: Contract.\_runningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:93

___

### \_wrappedEmits

• **\_wrappedEmits**: *object*

#### Type declaration

Inherited from: Contract.\_wrappedEmits

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### address

• `Readonly` **address**: *string*

Inherited from: Contract.address

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:71

___

### callStatic

• **callStatic**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tokensToSend` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `tokensToSend(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/IERC777Sender.d.ts:125

___

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

___

### estimateGas

• **estimateGas**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tokensToSend` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `tokensToSend(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/IERC777Sender.d.ts:149

___

### filters

• **filters**: *object*

#### Type declaration

Overrides: Contract.filters

Defined in: packages/ethereum/types/IERC777Sender.d.ts:147

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tokensToSend` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `tokensToSend(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/IERC777Sender.d.ts:83

___

### interface

• **interface**: *IERC777SenderInterface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/IERC777Sender.d.ts:81

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tokensToSend` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `tokensToSend(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/IERC777Sender.d.ts:171

___

### provider

• `Readonly` **provider**: *Provider*

Inherited from: Contract.provider

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:74

___

### resolvedAddress

• `Readonly` **resolvedAddress**: *Promise*<string\>

Inherited from: Contract.resolvedAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:90

___

### signer

• `Readonly` **signer**: *Signer*

Inherited from: Contract.signer

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:73

## Methods

### \_checkRunningEvents

▸ **_checkRunningEvents**(`runningEvent`: *RunningEvent*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | *RunningEvent* |

**Returns:** *void*

Inherited from: Contract.\_checkRunningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:113

___

### \_deployed

▸ **_deployed**(`blockTag?`: BlockTag): *Promise*<Contract\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockTag?` | BlockTag |

**Returns:** *Promise*<Contract\>

Inherited from: Contract.\_deployed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:106

___

### \_wrapEvent

▸ **_wrapEvent**(`runningEvent`: *RunningEvent*, `log`: Log, `listener`: Listener): Event

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | *RunningEvent* |
| `log` | Log |
| `listener` | Listener |

**Returns:** Event

Inherited from: Contract.\_wrapEvent

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:114

___

### attach

▸ **attach**(`addressOrName`: *string*): [*IERC777Sender*](ierc777sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/IERC777Sender.d.ts:42

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*IERC777Sender*](ierc777sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/IERC777Sender.d.ts:41

___

### deployed

▸ **deployed**(): *Promise*<[*IERC777Sender*](ierc777sender.md)\>

**Returns:** *Promise*<[*IERC777Sender*](ierc777sender.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/IERC777Sender.d.ts:43

___

### emit

▸ **emit**(`eventName`: *string* \| EventFilter, ...`args`: *any*[]): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* \| EventFilter |
| `...args` | *any*[] |

**Returns:** *boolean*

Inherited from: Contract.emit

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:119

___

### fallback

▸ **fallback**(`overrides?`: TransactionRequest): *Promise*<TransactionResponse\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | TransactionRequest |

**Returns:** *Promise*<TransactionResponse\>

Inherited from: Contract.fallback

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:107

___

### listenerCount

▸ **listenerCount**(`eventName?`: *string* \| EventFilter): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* \| EventFilter |

**Returns:** *number*

Inherited from: Contract.listenerCount

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:120

___

### listeners

▸ **listeners**<EventArgsArray, EventArgsObject\>(`eventFilter?`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/IERC777Sender.d.ts:45

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/IERC777Sender.d.ts:68

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*IERC777Sender*](ierc777sender.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/IERC777Sender.d.ts:48

▸ **off**(`eventName`: *string*, `listener`: Listener): [*IERC777Sender*](ierc777sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/IERC777Sender.d.ts:69

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*IERC777Sender*](ierc777sender.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/IERC777Sender.d.ts:52

▸ **on**(`eventName`: *string*, `listener`: Listener): [*IERC777Sender*](ierc777sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/IERC777Sender.d.ts:70

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*IERC777Sender*](ierc777sender.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/IERC777Sender.d.ts:56

▸ **once**(`eventName`: *string*, `listener`: Listener): [*IERC777Sender*](ierc777sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/IERC777Sender.d.ts:71

___

### queryFilter

▸ **queryFilter**<EventArgsArray, EventArgsObject\>(`event`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `fromBlockOrBlockhash?`: *string* \| *number*, `toBlock?`: *string* \| *number*): *Promise*<[*TypedEvent*](../interfaces/typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `fromBlockOrBlockhash?` | *string* \| *number* |
| `toBlock?` | *string* \| *number* |

**Returns:** *Promise*<[*TypedEvent*](../interfaces/typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

Overrides: Contract.queryFilter

Defined in: packages/ethereum/types/IERC777Sender.d.ts:75

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*IERC777Sender*](ierc777sender.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/IERC777Sender.d.ts:64

▸ **removeAllListeners**(`eventName?`: *string*): [*IERC777Sender*](ierc777sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/IERC777Sender.d.ts:73

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*IERC777Sender*](ierc777sender.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/IERC777Sender.d.ts:60

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*IERC777Sender*](ierc777sender.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*IERC777Sender*](ierc777sender.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/IERC777Sender.d.ts:72

___

### tokensToSend

▸ **tokensToSend**(`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `from` | *string* |
| `to` | *string* |
| `amount` | BigNumberish |
| `userData` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/IERC777Sender.d.ts:105

___

### tokensToSend(address,address,address,uint256,bytes,bytes)

▸ **tokensToSend(address,address,address,uint256,bytes,bytes)**(`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `from` | *string* |
| `to` | *string* |
| `amount` | BigNumberish |
| `userData` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/IERC777Sender.d.ts:113

___

### getContractAddress

▸ `Static` **getContractAddress**(`transaction`: { `from`: *string* ; `nonce`: BigNumberish  }): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `transaction` | *object* |
| `transaction.from` | *string* |
| `transaction.nonce` | BigNumberish |

**Returns:** *string*

Inherited from: Contract.getContractAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:100

___

### getInterface

▸ `Static` **getInterface**(`contractInterface`: ContractInterface): *Interface*

#### Parameters

| Name | Type |
| :------ | :------ |
| `contractInterface` | ContractInterface |

**Returns:** *Interface*

Inherited from: Contract.getInterface

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:104

___

### isIndexed

▸ `Static` **isIndexed**(`value`: *any*): value is Indexed

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | *any* |

**Returns:** value is Indexed

Inherited from: Contract.isIndexed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:110
