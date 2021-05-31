[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / Ownable

# Class: Ownable

## Hierarchy

- *Contract*

  ↳ **Ownable**

## Table of contents

### Constructors

- [constructor](ownable.md#constructor)

### Properties

- [\_deployedPromise](ownable.md#_deployedpromise)
- [\_runningEvents](ownable.md#_runningevents)
- [\_wrappedEmits](ownable.md#_wrappedemits)
- [address](ownable.md#address)
- [callStatic](ownable.md#callstatic)
- [deployTransaction](ownable.md#deploytransaction)
- [estimateGas](ownable.md#estimategas)
- [filters](ownable.md#filters)
- [functions](ownable.md#functions)
- [interface](ownable.md#interface)
- [populateTransaction](ownable.md#populatetransaction)
- [provider](ownable.md#provider)
- [resolvedAddress](ownable.md#resolvedaddress)
- [signer](ownable.md#signer)

### Methods

- [\_checkRunningEvents](ownable.md#_checkrunningevents)
- [\_deployed](ownable.md#_deployed)
- [\_wrapEvent](ownable.md#_wrapevent)
- [attach](ownable.md#attach)
- [connect](ownable.md#connect)
- [deployed](ownable.md#deployed)
- [emit](ownable.md#emit)
- [fallback](ownable.md#fallback)
- [listenerCount](ownable.md#listenercount)
- [listeners](ownable.md#listeners)
- [off](ownable.md#off)
- [on](ownable.md#on)
- [once](ownable.md#once)
- [owner](ownable.md#owner)
- [owner()](ownable.md#owner())
- [queryFilter](ownable.md#queryfilter)
- [removeAllListeners](ownable.md#removealllisteners)
- [removeListener](ownable.md#removelistener)
- [renounceOwnership](ownable.md#renounceownership)
- [renounceOwnership()](ownable.md#renounceownership())
- [transferOwnership](ownable.md#transferownership)
- [transferOwnership(address)](ownable.md#transferownership(address))
- [getContractAddress](ownable.md#getcontractaddress)
- [getInterface](ownable.md#getinterface)
- [isIndexed](ownable.md#isindexed)

## Constructors

### constructor

\+ **new Ownable**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*Ownable*](ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*Ownable*](ownable.md)

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
| `owner` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `renounceOwnership` | (`overrides?`: CallOverrides) => *Promise*<void\> |
| `renounceOwnership()` | (`overrides?`: CallOverrides) => *Promise*<void\> |
| `transferOwnership` | (`_newOwner`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `transferOwnership(address)` | (`_newOwner`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/Ownable.d.ts:147

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
| `owner` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferOwnership` | (`_newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferOwnership(address)` | (`_newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/Ownable.d.ts:181

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `OwnershipRenounced` | (`previousOwner`: *string*) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*], { `previousOwner`: *string*  }\> |
| `OwnershipTransferred` | (`previousOwner`: *string*, `newOwner`: *string*) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*], { `newOwner`: *string* ; `previousOwner`: *string*  }\> |

Overrides: Contract.filters

Defined in: packages/ethereum/types/Ownable.d.ts:167

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferOwnership` | (`_newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferOwnership(address)` | (`_newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/Ownable.d.ts:101

___

### interface

• **interface**: *OwnableInterface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/Ownable.d.ts:99

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferOwnership` | (`_newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferOwnership(address)` | (`_newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/Ownable.d.ts:205

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

▸ **attach**(`addressOrName`: *string*): [*Ownable*](ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/Ownable.d.ts:60

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*Ownable*](ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/Ownable.d.ts:59

___

### deployed

▸ **deployed**(): *Promise*<[*Ownable*](ownable.md)\>

**Returns:** *Promise*<[*Ownable*](ownable.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/Ownable.d.ts:61

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

Defined in: packages/ethereum/types/Ownable.d.ts:63

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/Ownable.d.ts:86

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*Ownable*](ownable.md)

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

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/Ownable.d.ts:66

▸ **off**(`eventName`: *string*, `listener`: Listener): [*Ownable*](ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/Ownable.d.ts:87

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*Ownable*](ownable.md)

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

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/Ownable.d.ts:70

▸ **on**(`eventName`: *string*, `listener`: Listener): [*Ownable*](ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/Ownable.d.ts:88

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*Ownable*](ownable.md)

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

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/Ownable.d.ts:74

▸ **once**(`eventName`: *string*, `listener`: Listener): [*Ownable*](ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/Ownable.d.ts:89

___

### owner

▸ **owner**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/Ownable.d.ts:133

___

### owner()

▸ **owner()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/Ownable.d.ts:133

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

Defined in: packages/ethereum/types/Ownable.d.ts:93

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*Ownable*](ownable.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/Ownable.d.ts:82

▸ **removeAllListeners**(`eventName?`: *string*): [*Ownable*](ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/Ownable.d.ts:91

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*Ownable*](ownable.md)

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

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/Ownable.d.ts:78

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*Ownable*](ownable.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*Ownable*](ownable.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/Ownable.d.ts:90

___

### renounceOwnership

▸ **renounceOwnership**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/Ownable.d.ts:125

___

### renounceOwnership()

▸ **renounceOwnership()**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/Ownable.d.ts:127

___

### transferOwnership

▸ **transferOwnership**(`_newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_newOwner` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/Ownable.d.ts:137

___

### transferOwnership(address)

▸ **transferOwnership(address)**(`_newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_newOwner` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/Ownable.d.ts:140

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
