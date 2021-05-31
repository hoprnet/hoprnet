[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC20

# Class: ERC20

## Hierarchy

- *Contract*

  ↳ **ERC20**

## Table of contents

### Constructors

- [constructor](erc20.md#constructor)

### Properties

- [\_deployedPromise](erc20.md#_deployedpromise)
- [\_runningEvents](erc20.md#_runningevents)
- [\_wrappedEmits](erc20.md#_wrappedemits)
- [address](erc20.md#address)
- [callStatic](erc20.md#callstatic)
- [deployTransaction](erc20.md#deploytransaction)
- [estimateGas](erc20.md#estimategas)
- [filters](erc20.md#filters)
- [functions](erc20.md#functions)
- [interface](erc20.md#interface)
- [populateTransaction](erc20.md#populatetransaction)
- [provider](erc20.md#provider)
- [resolvedAddress](erc20.md#resolvedaddress)
- [signer](erc20.md#signer)

### Methods

- [\_checkRunningEvents](erc20.md#_checkrunningevents)
- [\_deployed](erc20.md#_deployed)
- [\_wrapEvent](erc20.md#_wrapevent)
- [allowance](erc20.md#allowance)
- [allowance(address,address)](erc20.md#allowance(address,address))
- [approve](erc20.md#approve)
- [approve(address,uint256)](erc20.md#approve(address,uint256))
- [attach](erc20.md#attach)
- [balanceOf](erc20.md#balanceof)
- [balanceOf(address)](erc20.md#balanceof(address))
- [connect](erc20.md#connect)
- [deployed](erc20.md#deployed)
- [emit](erc20.md#emit)
- [fallback](erc20.md#fallback)
- [listenerCount](erc20.md#listenercount)
- [listeners](erc20.md#listeners)
- [off](erc20.md#off)
- [on](erc20.md#on)
- [once](erc20.md#once)
- [queryFilter](erc20.md#queryfilter)
- [removeAllListeners](erc20.md#removealllisteners)
- [removeListener](erc20.md#removelistener)
- [totalSupply](erc20.md#totalsupply)
- [totalSupply()](erc20.md#totalsupply())
- [transfer](erc20.md#transfer)
- [transfer(address,uint256)](erc20.md#transfer(address,uint256))
- [transferFrom](erc20.md#transferfrom)
- [transferFrom(address,address,uint256)](erc20.md#transferfrom(address,address,uint256))
- [getContractAddress](erc20.md#getcontractaddress)
- [getInterface](erc20.md#getinterface)
- [isIndexed](erc20.md#isindexed)

## Constructors

### constructor

\+ **new ERC20**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*ERC20*](erc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*ERC20*](erc20.md)

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
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/ERC20.d.ts:243

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
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/ERC20.d.ts:326

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner`: *string*, `spender`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `owner`: *string* ; `spender`: *string* ; `value`: *BigNumber*  }\> |
| `Transfer` | (`from`: *string*, `to`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `from`: *string* ; `to`: *string* ; `value`: *BigNumber*  }\> |

Overrides: Contract.filters

Defined in: packages/ethereum/types/ERC20.d.ts:306

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
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/ERC20.d.ts:119

___

### interface

• **interface**: *ERC20Interface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/ERC20.d.ts:117

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
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/ERC20.d.ts:389

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

Defined in: packages/ethereum/types/ERC20.d.ts:231

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

Defined in: packages/ethereum/types/ERC20.d.ts:235

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

Defined in: packages/ethereum/types/ERC20.d.ts:182

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

Defined in: packages/ethereum/types/ERC20.d.ts:186

___

### attach

▸ **attach**(`addressOrName`: *string*): [*ERC20*](erc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/ERC20.d.ts:78

___

### balanceOf

▸ **balanceOf**(`_who`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_who` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/ERC20.d.ts:212

___

### balanceOf(address)

▸ **balanceOf(address)**(`_who`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_who` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/ERC20.d.ts:212

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*ERC20*](erc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/ERC20.d.ts:77

___

### deployed

▸ **deployed**(): *Promise*<[*ERC20*](erc20.md)\>

**Returns:** *Promise*<[*ERC20*](erc20.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/ERC20.d.ts:79

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

Defined in: packages/ethereum/types/ERC20.d.ts:81

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/ERC20.d.ts:104

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ERC20*](erc20.md)

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

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/ERC20.d.ts:84

▸ **off**(`eventName`: *string*, `listener`: Listener): [*ERC20*](erc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/ERC20.d.ts:105

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ERC20*](erc20.md)

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

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/ERC20.d.ts:88

▸ **on**(`eventName`: *string*, `listener`: Listener): [*ERC20*](erc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/ERC20.d.ts:106

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ERC20*](erc20.md)

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

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/ERC20.d.ts:92

▸ **once**(`eventName`: *string*, `listener`: Listener): [*ERC20*](erc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/ERC20.d.ts:107

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

Defined in: packages/ethereum/types/ERC20.d.ts:111

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*ERC20*](erc20.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/ERC20.d.ts:100

▸ **removeAllListeners**(`eventName?`: *string*): [*ERC20*](erc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/ERC20.d.ts:109

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ERC20*](erc20.md)

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

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/ERC20.d.ts:96

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*ERC20*](erc20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ERC20*](erc20.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/ERC20.d.ts:108

___

### totalSupply

▸ **totalSupply**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/ERC20.d.ts:194

___

### totalSupply()

▸ **totalSupply()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/ERC20.d.ts:194

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

Defined in: packages/ethereum/types/ERC20.d.ts:219

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

Defined in: packages/ethereum/types/ERC20.d.ts:223

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

Defined in: packages/ethereum/types/ERC20.d.ts:198

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

Defined in: packages/ethereum/types/ERC20.d.ts:203

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
