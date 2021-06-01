[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ReentrancyGuard

# Class: ReentrancyGuard

## Hierarchy

- *Contract*

  ↳ **ReentrancyGuard**

## Table of contents

### Constructors

- [constructor](reentrancyguard.md#constructor)

### Properties

- [\_deployedPromise](reentrancyguard.md#_deployedpromise)
- [\_runningEvents](reentrancyguard.md#_runningevents)
- [\_wrappedEmits](reentrancyguard.md#_wrappedemits)
- [address](reentrancyguard.md#address)
- [callStatic](reentrancyguard.md#callstatic)
- [deployTransaction](reentrancyguard.md#deploytransaction)
- [estimateGas](reentrancyguard.md#estimategas)
- [filters](reentrancyguard.md#filters)
- [functions](reentrancyguard.md#functions)
- [interface](reentrancyguard.md#interface)
- [populateTransaction](reentrancyguard.md#populatetransaction)
- [provider](reentrancyguard.md#provider)
- [resolvedAddress](reentrancyguard.md#resolvedaddress)
- [signer](reentrancyguard.md#signer)

### Methods

- [\_checkRunningEvents](reentrancyguard.md#_checkrunningevents)
- [\_deployed](reentrancyguard.md#_deployed)
- [\_wrapEvent](reentrancyguard.md#_wrapevent)
- [attach](reentrancyguard.md#attach)
- [connect](reentrancyguard.md#connect)
- [deployed](reentrancyguard.md#deployed)
- [emit](reentrancyguard.md#emit)
- [fallback](reentrancyguard.md#fallback)
- [listenerCount](reentrancyguard.md#listenercount)
- [listeners](reentrancyguard.md#listeners)
- [off](reentrancyguard.md#off)
- [on](reentrancyguard.md#on)
- [once](reentrancyguard.md#once)
- [queryFilter](reentrancyguard.md#queryfilter)
- [removeAllListeners](reentrancyguard.md#removealllisteners)
- [removeListener](reentrancyguard.md#removelistener)
- [getContractAddress](reentrancyguard.md#getcontractaddress)
- [getInterface](reentrancyguard.md#getinterface)
- [isIndexed](reentrancyguard.md#isindexed)

## Constructors

### constructor

\+ **new ReentrancyGuard**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*ReentrancyGuard*](reentrancyguard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

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

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:71

___

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

___

### estimateGas

• **estimateGas**: *object*

#### Type declaration

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:75

___

### filters

• **filters**: *object*

#### Type declaration

Overrides: Contract.filters

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:73

___

### functions

• **functions**: *object*

#### Type declaration

Overrides: Contract.functions

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:69

___

### interface

• **interface**: *ReentrancyGuardInterface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:67

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:77

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

▸ **attach**(`addressOrName`: *string*): [*ReentrancyGuard*](reentrancyguard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:28

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*ReentrancyGuard*](reentrancyguard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:27

___

### deployed

▸ **deployed**(): *Promise*<[*ReentrancyGuard*](reentrancyguard.md)\>

**Returns:** *Promise*<[*ReentrancyGuard*](reentrancyguard.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:29

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

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:31

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:54

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ReentrancyGuard*](reentrancyguard.md)

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

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:34

▸ **off**(`eventName`: *string*, `listener`: Listener): [*ReentrancyGuard*](reentrancyguard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:55

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ReentrancyGuard*](reentrancyguard.md)

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

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:38

▸ **on**(`eventName`: *string*, `listener`: Listener): [*ReentrancyGuard*](reentrancyguard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:56

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ReentrancyGuard*](reentrancyguard.md)

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

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:42

▸ **once**(`eventName`: *string*, `listener`: Listener): [*ReentrancyGuard*](reentrancyguard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:57

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

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:61

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*ReentrancyGuard*](reentrancyguard.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:50

▸ **removeAllListeners**(`eventName?`: *string*): [*ReentrancyGuard*](reentrancyguard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:59

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ReentrancyGuard*](reentrancyguard.md)

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

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:46

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*ReentrancyGuard*](reentrancyguard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ReentrancyGuard*](reentrancyguard.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/ReentrancyGuard.d.ts:58

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
