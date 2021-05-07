[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprDistributor](../modules/contracts_hoprdistributor.md) / HoprDistributor

# Class: HoprDistributor

[contracts/HoprDistributor](../modules/contracts_hoprdistributor.md).HoprDistributor

## Hierarchy

- *Contract*

  ↳ **HoprDistributor**

## Table of contents

### Constructors

- [constructor](contracts_hoprdistributor.hoprdistributor.md#constructor)

### Properties

- [\_deployedPromise](contracts_hoprdistributor.hoprdistributor.md#_deployedpromise)
- [\_runningEvents](contracts_hoprdistributor.hoprdistributor.md#_runningevents)
- [\_wrappedEmits](contracts_hoprdistributor.hoprdistributor.md#_wrappedemits)
- [address](contracts_hoprdistributor.hoprdistributor.md#address)
- [callStatic](contracts_hoprdistributor.hoprdistributor.md#callstatic)
- [deployTransaction](contracts_hoprdistributor.hoprdistributor.md#deploytransaction)
- [estimateGas](contracts_hoprdistributor.hoprdistributor.md#estimategas)
- [filters](contracts_hoprdistributor.hoprdistributor.md#filters)
- [functions](contracts_hoprdistributor.hoprdistributor.md#functions)
- [interface](contracts_hoprdistributor.hoprdistributor.md#interface)
- [populateTransaction](contracts_hoprdistributor.hoprdistributor.md#populatetransaction)
- [provider](contracts_hoprdistributor.hoprdistributor.md#provider)
- [resolvedAddress](contracts_hoprdistributor.hoprdistributor.md#resolvedaddress)
- [signer](contracts_hoprdistributor.hoprdistributor.md#signer)

### Methods

- [MULTIPLIER](contracts_hoprdistributor.hoprdistributor.md#multiplier)
- [MULTIPLIER()](contracts_hoprdistributor.hoprdistributor.md#multiplier())
- [\_checkRunningEvents](contracts_hoprdistributor.hoprdistributor.md#_checkrunningevents)
- [\_deployed](contracts_hoprdistributor.hoprdistributor.md#_deployed)
- [\_wrapEvent](contracts_hoprdistributor.hoprdistributor.md#_wrapevent)
- [addAllocations](contracts_hoprdistributor.hoprdistributor.md#addallocations)
- [addAllocations(address[],uint128[],string)](contracts_hoprdistributor.hoprdistributor.md#addallocations(address[],uint128[],string))
- [addSchedule](contracts_hoprdistributor.hoprdistributor.md#addschedule)
- [addSchedule(uint128[],uint128[],string)](contracts_hoprdistributor.hoprdistributor.md#addschedule(uint128[],uint128[],string))
- [allocations](contracts_hoprdistributor.hoprdistributor.md#allocations)
- [allocations(address,string)](contracts_hoprdistributor.hoprdistributor.md#allocations(address,string))
- [attach](contracts_hoprdistributor.hoprdistributor.md#attach)
- [claim](contracts_hoprdistributor.hoprdistributor.md#claim)
- [claim(string)](contracts_hoprdistributor.hoprdistributor.md#claim(string))
- [claimFor](contracts_hoprdistributor.hoprdistributor.md#claimfor)
- [claimFor(address,string)](contracts_hoprdistributor.hoprdistributor.md#claimfor(address,string))
- [connect](contracts_hoprdistributor.hoprdistributor.md#connect)
- [deployed](contracts_hoprdistributor.hoprdistributor.md#deployed)
- [emit](contracts_hoprdistributor.hoprdistributor.md#emit)
- [fallback](contracts_hoprdistributor.hoprdistributor.md#fallback)
- [getClaimable](contracts_hoprdistributor.hoprdistributor.md#getclaimable)
- [getClaimable(address,string)](contracts_hoprdistributor.hoprdistributor.md#getclaimable(address,string))
- [getSchedule](contracts_hoprdistributor.hoprdistributor.md#getschedule)
- [getSchedule(string)](contracts_hoprdistributor.hoprdistributor.md#getschedule(string))
- [listenerCount](contracts_hoprdistributor.hoprdistributor.md#listenercount)
- [listeners](contracts_hoprdistributor.hoprdistributor.md#listeners)
- [maxMintAmount](contracts_hoprdistributor.hoprdistributor.md#maxmintamount)
- [maxMintAmount()](contracts_hoprdistributor.hoprdistributor.md#maxmintamount())
- [off](contracts_hoprdistributor.hoprdistributor.md#off)
- [on](contracts_hoprdistributor.hoprdistributor.md#on)
- [once](contracts_hoprdistributor.hoprdistributor.md#once)
- [owner](contracts_hoprdistributor.hoprdistributor.md#owner)
- [owner()](contracts_hoprdistributor.hoprdistributor.md#owner())
- [queryFilter](contracts_hoprdistributor.hoprdistributor.md#queryfilter)
- [removeAllListeners](contracts_hoprdistributor.hoprdistributor.md#removealllisteners)
- [removeListener](contracts_hoprdistributor.hoprdistributor.md#removelistener)
- [renounceOwnership](contracts_hoprdistributor.hoprdistributor.md#renounceownership)
- [renounceOwnership()](contracts_hoprdistributor.hoprdistributor.md#renounceownership())
- [revokeAccount](contracts_hoprdistributor.hoprdistributor.md#revokeaccount)
- [revokeAccount(address,string)](contracts_hoprdistributor.hoprdistributor.md#revokeaccount(address,string))
- [startTime](contracts_hoprdistributor.hoprdistributor.md#starttime)
- [startTime()](contracts_hoprdistributor.hoprdistributor.md#starttime())
- [token](contracts_hoprdistributor.hoprdistributor.md#token)
- [token()](contracts_hoprdistributor.hoprdistributor.md#token())
- [totalMinted](contracts_hoprdistributor.hoprdistributor.md#totalminted)
- [totalMinted()](contracts_hoprdistributor.hoprdistributor.md#totalminted())
- [totalToBeMinted](contracts_hoprdistributor.hoprdistributor.md#totaltobeminted)
- [totalToBeMinted()](contracts_hoprdistributor.hoprdistributor.md#totaltobeminted())
- [transferOwnership](contracts_hoprdistributor.hoprdistributor.md#transferownership)
- [transferOwnership(address)](contracts_hoprdistributor.hoprdistributor.md#transferownership(address))
- [updateStartTime](contracts_hoprdistributor.hoprdistributor.md#updatestarttime)
- [updateStartTime(uint128)](contracts_hoprdistributor.hoprdistributor.md#updatestarttime(uint128))
- [getContractAddress](contracts_hoprdistributor.hoprdistributor.md#getcontractaddress)
- [getInterface](contracts_hoprdistributor.hoprdistributor.md#getinterface)
- [isIndexed](contracts_hoprdistributor.hoprdistributor.md#isindexed)

## Constructors

### constructor

\+ **new HoprDistributor**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Provider* \| *Signer*): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Provider* \| *Signer* |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

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
| `MULTIPLIER` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `MULTIPLIER()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `addAllocations` | (`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `addSchedule` | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `addSchedule(uint128[],uint128[],string)` | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `allocations` | (`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: *BigNumber* ; `claimed`: *BigNumber* ; `lastClaim`: *BigNumber* ; `revoked`: *boolean*  }\> |
| `allocations(address,string)` | (`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: *BigNumber* ; `claimed`: *BigNumber* ; `lastClaim`: *BigNumber* ; `revoked`: *boolean*  }\> |
| `claim` | (`scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `claim(string)` | (`scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `claimFor` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `claimFor(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `getClaimable` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getClaimable(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getSchedule` | (`name`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*[], *BigNumber*[]]\> |
| `getSchedule(string)` | (`name`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*[], *BigNumber*[]]\> |
| `maxMintAmount` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `maxMintAmount()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `renounceOwnership` | (`overrides?`: CallOverrides) => *Promise*<void\> |
| `renounceOwnership()` | (`overrides?`: CallOverrides) => *Promise*<void\> |
| `revokeAccount` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `revokeAccount(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `startTime` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `startTime()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `token` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `token()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `totalMinted` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalMinted()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalToBeMinted` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalToBeMinted()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transferOwnership` | (`newOwner`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `transferOwnership(address)` | (`newOwner`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `updateStartTime` | (`_startTime`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |
| `updateStartTime(uint128)` | (`_startTime`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |

Overrides: Contract.callStatic

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:547

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
| `MULTIPLIER` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `MULTIPLIER()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `addAllocations` | (`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `addSchedule` | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `addSchedule(uint128[],uint128[],string)` | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `allocations` | (`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `allocations(address,string)` | (`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `claim` | (`scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `claim(string)` | (`scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `claimFor` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `claimFor(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `getClaimable` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getClaimable(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getSchedule` | (`name`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getSchedule(string)` | (`name`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `maxMintAmount` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `maxMintAmount()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `revokeAccount` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `revokeAccount(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `startTime` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `startTime()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `token` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `token()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalMinted` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalMinted()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalToBeMinted` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalToBeMinted()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transferOwnership` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferOwnership(address)` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `updateStartTime` | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `updateStartTime(uint128)` | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:745

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `AllocationAdded` | (`account`: *string*, `amount`: ``null``, `scheduleName`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *BigNumber*, *string*], { `account`: *string* ; `amount`: *BigNumber* ; `scheduleName`: *string*  }\> |
| `Claimed` | (`account`: *string*, `amount`: ``null``, `scheduleName`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *BigNumber*, *string*], { `account`: *string* ; `amount`: *BigNumber* ; `scheduleName`: *string*  }\> |
| `OwnershipTransferred` | (`previousOwner`: *string*, `newOwner`: *string*) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `newOwner`: *string* ; `previousOwner`: *string*  }\> |
| `ScheduleAdded` | (`durations`: ``null``, `percents`: ``null``, `name`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*BigNumber*[], *BigNumber*[], *string*], { `durations`: *BigNumber*[] ; `name`: *string* ; `percents`: *BigNumber*[]  }\> |

Overrides: Contract.filters

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:708

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `MULTIPLIER` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `MULTIPLIER()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `addAllocations` | (`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `addSchedule` | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `addSchedule(uint128[],uint128[],string)` | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `allocations` | (`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: *BigNumber* ; `claimed`: *BigNumber* ; `lastClaim`: *BigNumber* ; `revoked`: *boolean*  }\> |
| `allocations(address,string)` | (`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: *BigNumber* ; `claimed`: *BigNumber* ; `lastClaim`: *BigNumber* ; `revoked`: *boolean*  }\> |
| `claim` | (`scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `claim(string)` | (`scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `claimFor` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `claimFor(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `getClaimable` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `getClaimable(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `getSchedule` | (`name`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*[], *BigNumber*[]]\> |
| `getSchedule(string)` | (`name`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*[], *BigNumber*[]]\> |
| `maxMintAmount` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `maxMintAmount()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `revokeAccount` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `revokeAccount(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `startTime` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `startTime()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `token` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `token()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `totalMinted` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalMinted()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalToBeMinted` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalToBeMinted()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `transferOwnership` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferOwnership(address)` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `updateStartTime` | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `updateStartTime(uint128)` | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:213

___

### interface

• **interface**: [*HoprDistributorInterface*](../interfaces/contracts_hoprdistributor.hoprdistributorinterface.md)

Overrides: Contract.interface

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:211

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `MULTIPLIER` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `MULTIPLIER()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `addAllocations` | (`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `addSchedule` | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `addSchedule(uint128[],uint128[],string)` | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `allocations` | (`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `allocations(address,string)` | (`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `claim` | (`scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `claim(string)` | (`scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `claimFor` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `claimFor(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `getClaimable` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getClaimable(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getSchedule` | (`name`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getSchedule(string)` | (`name`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `maxMintAmount` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `maxMintAmount()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `revokeAccount` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `revokeAccount(address,string)` | (`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `startTime` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `startTime()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `token` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `token()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalMinted` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalMinted()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalToBeMinted` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalToBeMinted()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `transferOwnership` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferOwnership(address)` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `updateStartTime` | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `updateStartTime(uint128)` | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:896

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

### MULTIPLIER

▸ **MULTIPLIER**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:381

___

### MULTIPLIER()

▸ **MULTIPLIER()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:381

___

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

### addAllocations

▸ **addAllocations**(`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accounts` | *string*[] |
| `amounts` | BigNumberish[] |
| `scheduleName` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:385

___

### addAllocations(address[],uint128[],string)

▸ **addAllocations(address[],uint128[],string)**(`accounts`: *string*[], `amounts`: BigNumberish[], `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accounts` | *string*[] |
| `amounts` | BigNumberish[] |
| `scheduleName` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:390

___

### addSchedule

▸ **addSchedule**(`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `durations` | BigNumberish[] |
| `percents` | BigNumberish[] |
| `name` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:399

___

### addSchedule(uint128[],uint128[],string)

▸ **addSchedule(uint128[],uint128[],string)**(`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `durations` | BigNumberish[] |
| `percents` | BigNumberish[] |
| `name` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:404

___

### allocations

▸ **allocations**(`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides): *Promise*<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: *BigNumber* ; `claimed`: *BigNumber* ; `lastClaim`: *BigNumber* ; `revoked`: *boolean*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | *string* |
| `arg1` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: *BigNumber* ; `claimed`: *BigNumber* ; `lastClaim`: *BigNumber* ; `revoked`: *boolean*  }\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:413

___

### allocations(address,string)

▸ **allocations(address,string)**(`arg0`: *string*, `arg1`: *string*, `overrides?`: CallOverrides): *Promise*<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: *BigNumber* ; `claimed`: *BigNumber* ; `lastClaim`: *BigNumber* ; `revoked`: *boolean*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | *string* |
| `arg1` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: *BigNumber* ; `claimed`: *BigNumber* ; `lastClaim`: *BigNumber* ; `revoked`: *boolean*  }\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:424

___

### attach

▸ **attach**(`addressOrName`: *string*): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.attach

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:172

___

### claim

▸ **claim**(`scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `scheduleName` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:439

___

### claim(string)

▸ **claim(string)**(`scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `scheduleName` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:442

___

### claimFor

▸ **claimFor**(`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `scheduleName` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:449

___

### claimFor(address,string)

▸ **claimFor(address,string)**(`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `scheduleName` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:453

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Provider* \| *Signer*): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Provider* \| *Signer* |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.connect

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:171

___

### deployed

▸ **deployed**(): *Promise*<[*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)\>

**Returns:** *Promise*<[*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)\>

Overrides: Contract.deployed

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:173

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

### getClaimable

▸ **getClaimable**(`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `scheduleName` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:461

___

### getClaimable(address,string)

▸ **getClaimable(address,string)**(`account`: *string*, `scheduleName`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `scheduleName` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:465

___

### getSchedule

▸ **getSchedule**(`name`: *string*, `overrides?`: CallOverrides): *Promise*<[*BigNumber*[], *BigNumber*[]]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `name` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*[], *BigNumber*[]]\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:473

___

### getSchedule(string)

▸ **getSchedule(string)**(`name`: *string*, `overrides?`: CallOverrides): *Promise*<[*BigNumber*[], *BigNumber*[]]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `name` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*[], *BigNumber*[]]\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:476

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

▸ **listeners**<EventArgsArray, EventArgsObject\>(`eventFilter?`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:175

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:198

___

### maxMintAmount

▸ **maxMintAmount**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:483

___

### maxMintAmount()

▸ **maxMintAmount()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:483

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:178

▸ **off**(`eventName`: *string*, `listener`: Listener): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:199

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:182

▸ **on**(`eventName`: *string*, `listener`: Listener): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:200

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:186

▸ **once**(`eventName`: *string*, `listener`: Listener): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:201

___

### owner

▸ **owner**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:487

___

### owner()

▸ **owner()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:487

___

### queryFilter

▸ **queryFilter**<EventArgsArray, EventArgsObject\>(`event`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `fromBlockOrBlockhash?`: *string* \| *number*, `toBlock?`: *string* \| *number*): *Promise*<[*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `fromBlockOrBlockhash?` | *string* \| *number* |
| `toBlock?` | *string* \| *number* |

**Returns:** *Promise*<[*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

Overrides: Contract.queryFilter

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:205

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:194

▸ **removeAllListeners**(`eventName?`: *string*): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:203

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:190

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprDistributor*](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:202

___

### renounceOwnership

▸ **renounceOwnership**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:491

___

### renounceOwnership()

▸ **renounceOwnership()**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:493

___

### revokeAccount

▸ **revokeAccount**(`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `scheduleName` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:499

___

### revokeAccount(address,string)

▸ **revokeAccount(address,string)**(`account`: *string*, `scheduleName`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `scheduleName` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:503

___

### startTime

▸ **startTime**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:511

___

### startTime()

▸ **startTime()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:511

___

### token

▸ **token**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:515

___

### token()

▸ **token()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:515

___

### totalMinted

▸ **totalMinted**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:519

___

### totalMinted()

▸ **totalMinted()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:519

___

### totalToBeMinted

▸ **totalToBeMinted**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:523

___

### totalToBeMinted()

▸ **totalToBeMinted()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:523

___

### transferOwnership

▸ **transferOwnership**(`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newOwner` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:527

___

### transferOwnership(address)

▸ **transferOwnership(address)**(`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newOwner` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:530

___

### updateStartTime

▸ **updateStartTime**(`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_startTime` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:537

___

### updateStartTime(uint128)

▸ **updateStartTime(uint128)**(`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_startTime` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:540

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
