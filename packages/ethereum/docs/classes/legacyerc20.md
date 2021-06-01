[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / LegacyERC20

# Class: LegacyERC20

## Hierarchy

- *Contract*

  ↳ **LegacyERC20**

## Table of contents

### Constructors

- [constructor](legacyerc20.md#constructor)

### Properties

- [\_deployedPromise](legacyerc20.md#_deployedpromise)
- [\_runningEvents](legacyerc20.md#_runningevents)
- [\_wrappedEmits](legacyerc20.md#_wrappedemits)
- [address](legacyerc20.md#address)
- [callStatic](legacyerc20.md#callstatic)
- [deployTransaction](legacyerc20.md#deploytransaction)
- [estimateGas](legacyerc20.md#estimategas)
- [filters](legacyerc20.md#filters)
- [functions](legacyerc20.md#functions)
- [interface](legacyerc20.md#interface)
- [populateTransaction](legacyerc20.md#populatetransaction)
- [provider](legacyerc20.md#provider)
- [resolvedAddress](legacyerc20.md#resolvedaddress)
- [signer](legacyerc20.md#signer)

### Methods

- [\_checkRunningEvents](legacyerc20.md#_checkrunningevents)
- [\_deployed](legacyerc20.md#_deployed)
- [\_wrapEvent](legacyerc20.md#_wrapevent)
- [attach](legacyerc20.md#attach)
- [connect](legacyerc20.md#connect)
- [deployed](legacyerc20.md#deployed)
- [emit](legacyerc20.md#emit)
- [fallback](legacyerc20.md#fallback)
- [listenerCount](legacyerc20.md#listenercount)
- [listeners](legacyerc20.md#listeners)
- [off](legacyerc20.md#off)
- [on](legacyerc20.md#on)
- [once](legacyerc20.md#once)
- [queryFilter](legacyerc20.md#queryfilter)
- [removeAllListeners](legacyerc20.md#removealllisteners)
- [removeListener](legacyerc20.md#removelistener)
- [transfer](legacyerc20.md#transfer)
- [transfer(address,uint256)](legacyerc20.md#transfer(address,uint256))
- [transferFrom](legacyerc20.md#transferfrom)
- [transferFrom(address,address,uint256)](legacyerc20.md#transferfrom(address,address,uint256))
- [getContractAddress](legacyerc20.md#getcontractaddress)
- [getInterface](legacyerc20.md#getinterface)
- [isIndexed](legacyerc20.md#isindexed)

## Constructors

### constructor

\+ **new LegacyERC20**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*LegacyERC20*](legacyerc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*LegacyERC20*](legacyerc20.md)

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
| `transfer` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |
| `transfer(address,uint256)` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |
| `transferFrom` | (`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |
| `transferFrom(address,address,uint256)` | (`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/LegacyERC20.d.ts:143

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
| `transfer` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transfer(address,uint256)` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom` | (`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom(address,address,uint256)` | (`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/LegacyERC20.d.ts:173

___

### filters

• **filters**: *object*

#### Type declaration

Overrides: Contract.filters

Defined in: packages/ethereum/types/LegacyERC20.d.ts:171

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `transfer` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transfer(address,uint256)` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom` | (`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom(address,address,uint256)` | (`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/LegacyERC20.d.ts:89

___

### interface

• **interface**: *LegacyERC20Interface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/LegacyERC20.d.ts:87

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `transfer` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transfer(address,uint256)` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom` | (`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom(address,address,uint256)` | (`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/LegacyERC20.d.ts:201

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

▸ **attach**(`addressOrName`: *string*): [*LegacyERC20*](legacyerc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/LegacyERC20.d.ts:48

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*LegacyERC20*](legacyerc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/LegacyERC20.d.ts:47

___

### deployed

▸ **deployed**(): *Promise*<[*LegacyERC20*](legacyerc20.md)\>

**Returns:** *Promise*<[*LegacyERC20*](legacyerc20.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/LegacyERC20.d.ts:49

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

Defined in: packages/ethereum/types/LegacyERC20.d.ts:51

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/LegacyERC20.d.ts:74

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*LegacyERC20*](legacyerc20.md)

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

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/LegacyERC20.d.ts:54

▸ **off**(`eventName`: *string*, `listener`: Listener): [*LegacyERC20*](legacyerc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/LegacyERC20.d.ts:75

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*LegacyERC20*](legacyerc20.md)

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

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/LegacyERC20.d.ts:58

▸ **on**(`eventName`: *string*, `listener`: Listener): [*LegacyERC20*](legacyerc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/LegacyERC20.d.ts:76

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*LegacyERC20*](legacyerc20.md)

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

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/LegacyERC20.d.ts:62

▸ **once**(`eventName`: *string*, `listener`: Listener): [*LegacyERC20*](legacyerc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/LegacyERC20.d.ts:77

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

Defined in: packages/ethereum/types/LegacyERC20.d.ts:81

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*LegacyERC20*](legacyerc20.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/LegacyERC20.d.ts:70

▸ **removeAllListeners**(`eventName?`: *string*): [*LegacyERC20*](legacyerc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/LegacyERC20.d.ts:79

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*LegacyERC20*](legacyerc20.md)

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

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/LegacyERC20.d.ts:66

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*LegacyERC20*](legacyerc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*LegacyERC20*](legacyerc20.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/LegacyERC20.d.ts:78

___

### transfer

▸ **transfer**(`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/LegacyERC20.d.ts:131

___

### transfer(address,uint256)

▸ **transfer(address,uint256)**(`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/LegacyERC20.d.ts:135

___

### transferFrom

▸ **transferFrom**(`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `_spender` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/LegacyERC20.d.ts:117

___

### transferFrom(address,address,uint256)

▸ **transferFrom(address,address,uint256)**(`_owner`: *string*, `_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `_spender` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/LegacyERC20.d.ts:122

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
