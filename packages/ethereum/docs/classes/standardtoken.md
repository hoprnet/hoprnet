[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / StandardToken

# Class: StandardToken

## Hierarchy

- *Contract*

  ↳ **StandardToken**

## Table of contents

### Constructors

- [constructor](standardtoken.md#constructor)

### Properties

- [\_deployedPromise](standardtoken.md#_deployedpromise)
- [\_runningEvents](standardtoken.md#_runningevents)
- [\_wrappedEmits](standardtoken.md#_wrappedemits)
- [address](standardtoken.md#address)
- [callStatic](standardtoken.md#callstatic)
- [deployTransaction](standardtoken.md#deploytransaction)
- [estimateGas](standardtoken.md#estimategas)
- [filters](standardtoken.md#filters)
- [functions](standardtoken.md#functions)
- [interface](standardtoken.md#interface)
- [populateTransaction](standardtoken.md#populatetransaction)
- [provider](standardtoken.md#provider)
- [resolvedAddress](standardtoken.md#resolvedaddress)
- [signer](standardtoken.md#signer)

### Methods

- [\_checkRunningEvents](standardtoken.md#_checkrunningevents)
- [\_deployed](standardtoken.md#_deployed)
- [\_wrapEvent](standardtoken.md#_wrapevent)
- [allowance](standardtoken.md#allowance)
- [allowance(address,address)](standardtoken.md#allowance(address,address))
- [approve](standardtoken.md#approve)
- [approve(address,uint256)](standardtoken.md#approve(address,uint256))
- [attach](standardtoken.md#attach)
- [balanceOf](standardtoken.md#balanceof)
- [balanceOf(address)](standardtoken.md#balanceof(address))
- [connect](standardtoken.md#connect)
- [decreaseApproval](standardtoken.md#decreaseapproval)
- [decreaseApproval(address,uint256)](standardtoken.md#decreaseapproval(address,uint256))
- [deployed](standardtoken.md#deployed)
- [emit](standardtoken.md#emit)
- [fallback](standardtoken.md#fallback)
- [increaseApproval](standardtoken.md#increaseapproval)
- [increaseApproval(address,uint256)](standardtoken.md#increaseapproval(address,uint256))
- [listenerCount](standardtoken.md#listenercount)
- [listeners](standardtoken.md#listeners)
- [off](standardtoken.md#off)
- [on](standardtoken.md#on)
- [once](standardtoken.md#once)
- [queryFilter](standardtoken.md#queryfilter)
- [removeAllListeners](standardtoken.md#removealllisteners)
- [removeListener](standardtoken.md#removelistener)
- [totalSupply](standardtoken.md#totalsupply)
- [totalSupply()](standardtoken.md#totalsupply())
- [transfer](standardtoken.md#transfer)
- [transfer(address,uint256)](standardtoken.md#transfer(address,uint256))
- [transferFrom](standardtoken.md#transferfrom)
- [transferFrom(address,address,uint256)](standardtoken.md#transferfrom(address,address,uint256))
- [getContractAddress](standardtoken.md#getcontractaddress)
- [getInterface](standardtoken.md#getinterface)
- [isIndexed](standardtoken.md#isindexed)

## Constructors

### constructor

\+ **new StandardToken**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*StandardToken*](standardtoken.md)

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
| `balanceOf` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOf(address)` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `decreaseApproval` | (`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `decreaseApproval(address,uint256)` | (`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `increaseApproval` | (`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `increaseApproval(address,uint256)` | (`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/StandardToken.d.ts:309

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
| `balanceOf` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOf(address)` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `decreaseApproval` | (`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `decreaseApproval(address,uint256)` | (`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `increaseApproval` | (`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `increaseApproval(address,uint256)` | (`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/StandardToken.d.ts:416

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner`: *string*, `spender`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `owner`: *string* ; `spender`: *string* ; `value`: *BigNumber*  }\> |
| `Transfer` | (`from`: *string*, `to`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `from`: *string* ; `to`: *string* ; `value`: *BigNumber*  }\> |

Overrides: Contract.filters

Defined in: packages/ethereum/types/StandardToken.d.ts:396

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
| `balanceOf` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `balanceOf(address)` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `decreaseApproval` | (`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `decreaseApproval(address,uint256)` | (`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `increaseApproval` | (`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `increaseApproval(address,uint256)` | (`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/StandardToken.d.ts:137

___

### interface

• **interface**: *StandardTokenInterface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/StandardToken.d.ts:135

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
| `balanceOf` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `balanceOf(address)` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `decreaseApproval` | (`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `decreaseApproval(address,uint256)` | (`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `increaseApproval` | (`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `increaseApproval(address,uint256)` | (`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom(address,address,uint256)` | (`_from`: *string*, `_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/StandardToken.d.ts:503

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

Defined in: packages/ethereum/types/StandardToken.d.ts:297

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

Defined in: packages/ethereum/types/StandardToken.d.ts:301

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

Defined in: packages/ethereum/types/StandardToken.d.ts:224

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

Defined in: packages/ethereum/types/StandardToken.d.ts:228

___

### attach

▸ **attach**(`addressOrName`: *string*): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/StandardToken.d.ts:96

___

### balanceOf

▸ **balanceOf**(`_owner`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/StandardToken.d.ts:266

___

### balanceOf(address)

▸ **balanceOf(address)**(`_owner`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/StandardToken.d.ts:266

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/StandardToken.d.ts:95

___

### decreaseApproval

▸ **decreaseApproval**(`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | *string* |
| `_subtractedValue` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/StandardToken.d.ts:254

___

### decreaseApproval(address,uint256)

▸ **decreaseApproval(address,uint256)**(`_spender`: *string*, `_subtractedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | *string* |
| `_subtractedValue` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/StandardToken.d.ts:258

___

### deployed

▸ **deployed**(): *Promise*<[*StandardToken*](standardtoken.md)\>

**Returns:** *Promise*<[*StandardToken*](standardtoken.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/StandardToken.d.ts:97

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

### increaseApproval

▸ **increaseApproval**(`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | *string* |
| `_addedValue` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/StandardToken.d.ts:285

___

### increaseApproval(address,uint256)

▸ **increaseApproval(address,uint256)**(`_spender`: *string*, `_addedValue`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | *string* |
| `_addedValue` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/StandardToken.d.ts:289

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

Defined in: packages/ethereum/types/StandardToken.d.ts:99

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/StandardToken.d.ts:122

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*StandardToken*](standardtoken.md)

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

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/StandardToken.d.ts:102

▸ **off**(`eventName`: *string*, `listener`: Listener): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/StandardToken.d.ts:123

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*StandardToken*](standardtoken.md)

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

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/StandardToken.d.ts:106

▸ **on**(`eventName`: *string*, `listener`: Listener): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/StandardToken.d.ts:124

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*StandardToken*](standardtoken.md)

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

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/StandardToken.d.ts:110

▸ **once**(`eventName`: *string*, `listener`: Listener): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/StandardToken.d.ts:125

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

Defined in: packages/ethereum/types/StandardToken.d.ts:129

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*StandardToken*](standardtoken.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/StandardToken.d.ts:118

▸ **removeAllListeners**(`eventName?`: *string*): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/StandardToken.d.ts:127

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*StandardToken*](standardtoken.md)

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

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/StandardToken.d.ts:114

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*StandardToken*](standardtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*StandardToken*](standardtoken.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/StandardToken.d.ts:126

___

### totalSupply

▸ **totalSupply**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/StandardToken.d.ts:236

___

### totalSupply()

▸ **totalSupply()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/StandardToken.d.ts:236

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

Defined in: packages/ethereum/types/StandardToken.d.ts:273

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

Defined in: packages/ethereum/types/StandardToken.d.ts:277

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

Defined in: packages/ethereum/types/StandardToken.d.ts:240

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

Defined in: packages/ethereum/types/StandardToken.d.ts:245

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
