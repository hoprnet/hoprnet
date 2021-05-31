[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / DetailedERC20

# Class: DetailedERC20

## Hierarchy

- *Contract*

  ↳ **DetailedERC20**

## Table of contents

### Constructors

- [constructor](detailederc20.md#constructor)

### Properties

- [\_deployedPromise](detailederc20.md#_deployedpromise)
- [\_runningEvents](detailederc20.md#_runningevents)
- [\_wrappedEmits](detailederc20.md#_wrappedemits)
- [address](detailederc20.md#address)
- [callStatic](detailederc20.md#callstatic)
- [deployTransaction](detailederc20.md#deploytransaction)
- [estimateGas](detailederc20.md#estimategas)
- [filters](detailederc20.md#filters)
- [functions](detailederc20.md#functions)
- [interface](detailederc20.md#interface)
- [populateTransaction](detailederc20.md#populatetransaction)
- [provider](detailederc20.md#provider)
- [resolvedAddress](detailederc20.md#resolvedaddress)
- [signer](detailederc20.md#signer)

### Methods

- [\_checkRunningEvents](detailederc20.md#_checkrunningevents)
- [\_deployed](detailederc20.md#_deployed)
- [\_wrapEvent](detailederc20.md#_wrapevent)
- [allowance](detailederc20.md#allowance)
- [allowance(address,address)](detailederc20.md#allowance(address,address))
- [approve](detailederc20.md#approve)
- [approve(address,uint256)](detailederc20.md#approve(address,uint256))
- [attach](detailederc20.md#attach)
- [balanceOf](detailederc20.md#balanceof)
- [balanceOf(address)](detailederc20.md#balanceof(address))
- [connect](detailederc20.md#connect)
- [decimals](detailederc20.md#decimals)
- [decimals()](detailederc20.md#decimals())
- [deployed](detailederc20.md#deployed)
- [emit](detailederc20.md#emit)
- [fallback](detailederc20.md#fallback)
- [listenerCount](detailederc20.md#listenercount)
- [listeners](detailederc20.md#listeners)
- [name](detailederc20.md#name)
- [name()](detailederc20.md#name())
- [off](detailederc20.md#off)
- [on](detailederc20.md#on)
- [once](detailederc20.md#once)
- [queryFilter](detailederc20.md#queryfilter)
- [removeAllListeners](detailederc20.md#removealllisteners)
- [removeListener](detailederc20.md#removelistener)
- [symbol](detailederc20.md#symbol)
- [symbol()](detailederc20.md#symbol())
- [totalSupply](detailederc20.md#totalsupply)
- [totalSupply()](detailederc20.md#totalsupply())
- [transfer](detailederc20.md#transfer)
- [transfer(address,uint256)](detailederc20.md#transfer(address,uint256))
- [transferFrom](detailederc20.md#transferfrom)
- [transferFrom(address,address,uint256)](detailederc20.md#transferfrom(address,address,uint256))
- [getContractAddress](detailederc20.md#getcontractaddress)
- [getInterface](detailederc20.md#getinterface)
- [isIndexed](detailederc20.md#isindexed)

## Constructors

### constructor

\+ **new DetailedERC20**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*DetailedERC20*](detailederc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*DetailedERC20*](detailederc20.md)

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
| `allowance` | (`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `allowance(address,address)` | (`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `approve` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `approve(address,uint256)` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `balanceOf` | (`_who`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOf(address)` | (`_who`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `decimals` | (`overrides?`: CallOverrides) => *Promise*<number\> |
| `decimals()` | (`overrides?`: CallOverrides) => *Promise*<number\> |
| `name` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `name()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `symbol` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `symbol()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/DetailedERC20.d.ts:276

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
| `allowance` | (`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `allowance(address,address)` | (`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `approve` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `approve(address,uint256)` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `balanceOf` | (`_who`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOf(address)` | (`_who`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `decimals` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `decimals()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `name` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `name()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `symbol` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `symbol()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/DetailedERC20.d.ts:371

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner`: *string*, `spender`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `owner`: *string* ; `spender`: *string* ; `value`: *BigNumber*  }\> |
| `Transfer` | (`from`: *string*, `to`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `from`: *string* ; `to`: *string* ; `value`: *BigNumber*  }\> |

Overrides: Contract.filters

Defined in: packages/ethereum/types/DetailedERC20.d.ts:351

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowance` | (`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `allowance(address,address)` | (`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `approve` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `approve(address,uint256)` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `balanceOf` | (`_who`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `balanceOf(address)` | (`_who`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `decimals` | (`overrides?`: CallOverrides) => *Promise*<[*number*]\> |
| `decimals()` | (`overrides?`: CallOverrides) => *Promise*<[*number*]\> |
| `name` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `name()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `symbol` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `symbol()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/DetailedERC20.d.ts:128

___

### interface

• **interface**: *DetailedERC20Interface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/DetailedERC20.d.ts:126

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowance` | (`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `allowance(address,address)` | (`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `approve` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `approve(address,uint256)` | (`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `balanceOf` | (`_who`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `balanceOf(address)` | (`_who`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `decimals` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `decimals()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `name` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `name()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `symbol` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `symbol()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/DetailedERC20.d.ts:446

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

### allowance

▸ **allowance**(`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `_spender` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:264

___

### allowance(address,address)

▸ **allowance(address,address)**(`_owner`: *string*, `_spender`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `_spender` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:268

___

### approve

▸ **approve**(`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:207

___

### approve(address,uint256)

▸ **approve(address,uint256)**(`_spender`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:211

___

### attach

▸ **attach**(`addressOrName`: *string*): [*DetailedERC20*](detailederc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/DetailedERC20.d.ts:87

___

### balanceOf

▸ **balanceOf**(`_who`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_who` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:241

___

### balanceOf(address)

▸ **balanceOf(address)**(`_who`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_who` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:241

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*DetailedERC20*](detailederc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/DetailedERC20.d.ts:86

___

### decimals

▸ **decimals**(`overrides?`: CallOverrides): *Promise*<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<number\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:237

___

### decimals()

▸ **decimals()**(`overrides?`: CallOverrides): *Promise*<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<number\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:237

___

### deployed

▸ **deployed**(): *Promise*<[*DetailedERC20*](detailederc20.md)\>

**Returns:** *Promise*<[*DetailedERC20*](detailederc20.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/DetailedERC20.d.ts:88

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

Defined in: packages/ethereum/types/DetailedERC20.d.ts:90

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/DetailedERC20.d.ts:113

___

### name

▸ **name**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:203

___

### name()

▸ **name()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:203

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*DetailedERC20*](detailederc20.md)

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

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/DetailedERC20.d.ts:93

▸ **off**(`eventName`: *string*, `listener`: Listener): [*DetailedERC20*](detailederc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/DetailedERC20.d.ts:114

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*DetailedERC20*](detailederc20.md)

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

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/DetailedERC20.d.ts:97

▸ **on**(`eventName`: *string*, `listener`: Listener): [*DetailedERC20*](detailederc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/DetailedERC20.d.ts:115

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*DetailedERC20*](detailederc20.md)

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

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/DetailedERC20.d.ts:101

▸ **once**(`eventName`: *string*, `listener`: Listener): [*DetailedERC20*](detailederc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/DetailedERC20.d.ts:116

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

Defined in: packages/ethereum/types/DetailedERC20.d.ts:120

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*DetailedERC20*](detailederc20.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/DetailedERC20.d.ts:109

▸ **removeAllListeners**(`eventName?`: *string*): [*DetailedERC20*](detailederc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/DetailedERC20.d.ts:118

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*DetailedERC20*](detailederc20.md)

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

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/DetailedERC20.d.ts:105

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*DetailedERC20*](detailederc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*DetailedERC20*](detailederc20.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/DetailedERC20.d.ts:117

___

### symbol

▸ **symbol**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:248

___

### symbol()

▸ **symbol()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:248

___

### totalSupply

▸ **totalSupply**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:219

___

### totalSupply()

▸ **totalSupply()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:219

___

### transfer

▸ **transfer**(`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:252

___

### transfer(address,uint256)

▸ **transfer(address,uint256)**(`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:256

___

### transferFrom

▸ **transferFrom**(`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_from` | *string* |
| `_to` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:223

___

### transferFrom(address,address,uint256)

▸ **transferFrom(address,address,uint256)**(`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_from` | *string* |
| `_to` | *string* |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/DetailedERC20.d.ts:228

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
