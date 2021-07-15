[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprDistributor

# Class: HoprDistributor

## Hierarchy

- `Contract`

  ↳ **`HoprDistributor`**

## Table of contents

### Constructors

- [constructor](hoprdistributor.md#constructor)

### Properties

- [\_deployedPromise](hoprdistributor.md#_deployedpromise)
- [\_runningEvents](hoprdistributor.md#_runningevents)
- [\_wrappedEmits](hoprdistributor.md#_wrappedemits)
- [address](hoprdistributor.md#address)
- [callStatic](hoprdistributor.md#callstatic)
- [deployTransaction](hoprdistributor.md#deploytransaction)
- [estimateGas](hoprdistributor.md#estimategas)
- [filters](hoprdistributor.md#filters)
- [functions](hoprdistributor.md#functions)
- [interface](hoprdistributor.md#interface)
- [populateTransaction](hoprdistributor.md#populatetransaction)
- [provider](hoprdistributor.md#provider)
- [resolvedAddress](hoprdistributor.md#resolvedaddress)
- [signer](hoprdistributor.md#signer)

### Methods

- [MULTIPLIER](hoprdistributor.md#multiplier)
- [MULTIPLIER()](hoprdistributor.md#multiplier())
- [\_checkRunningEvents](hoprdistributor.md#_checkrunningevents)
- [\_deployed](hoprdistributor.md#_deployed)
- [\_wrapEvent](hoprdistributor.md#_wrapevent)
- [addAllocations](hoprdistributor.md#addallocations)
- [addAllocations(address[],uint128[],string)](hoprdistributor.md#addallocations(address[],uint128[],string))
- [addSchedule](hoprdistributor.md#addschedule)
- [addSchedule(uint128[],uint128[],string)](hoprdistributor.md#addschedule(uint128[],uint128[],string))
- [allocations](hoprdistributor.md#allocations)
- [allocations(address,string)](hoprdistributor.md#allocations(address,string))
- [attach](hoprdistributor.md#attach)
- [claim](hoprdistributor.md#claim)
- [claim(string)](hoprdistributor.md#claim(string))
- [claimFor](hoprdistributor.md#claimfor)
- [claimFor(address,string)](hoprdistributor.md#claimfor(address,string))
- [connect](hoprdistributor.md#connect)
- [deployed](hoprdistributor.md#deployed)
- [emit](hoprdistributor.md#emit)
- [fallback](hoprdistributor.md#fallback)
- [getClaimable](hoprdistributor.md#getclaimable)
- [getClaimable(address,string)](hoprdistributor.md#getclaimable(address,string))
- [getSchedule](hoprdistributor.md#getschedule)
- [getSchedule(string)](hoprdistributor.md#getschedule(string))
- [listenerCount](hoprdistributor.md#listenercount)
- [listeners](hoprdistributor.md#listeners)
- [maxMintAmount](hoprdistributor.md#maxmintamount)
- [maxMintAmount()](hoprdistributor.md#maxmintamount())
- [off](hoprdistributor.md#off)
- [on](hoprdistributor.md#on)
- [once](hoprdistributor.md#once)
- [owner](hoprdistributor.md#owner)
- [owner()](hoprdistributor.md#owner())
- [queryFilter](hoprdistributor.md#queryfilter)
- [removeAllListeners](hoprdistributor.md#removealllisteners)
- [removeListener](hoprdistributor.md#removelistener)
- [renounceOwnership](hoprdistributor.md#renounceownership)
- [renounceOwnership()](hoprdistributor.md#renounceownership())
- [revokeAccount](hoprdistributor.md#revokeaccount)
- [revokeAccount(address,string)](hoprdistributor.md#revokeaccount(address,string))
- [startTime](hoprdistributor.md#starttime)
- [startTime()](hoprdistributor.md#starttime())
- [token](hoprdistributor.md#token)
- [token()](hoprdistributor.md#token())
- [totalMinted](hoprdistributor.md#totalminted)
- [totalMinted()](hoprdistributor.md#totalminted())
- [totalToBeMinted](hoprdistributor.md#totaltobeminted)
- [totalToBeMinted()](hoprdistributor.md#totaltobeminted())
- [transferOwnership](hoprdistributor.md#transferownership)
- [transferOwnership(address)](hoprdistributor.md#transferownership(address))
- [updateStartTime](hoprdistributor.md#updatestarttime)
- [updateStartTime(uint128)](hoprdistributor.md#updatestarttime(uint128))
- [getContractAddress](hoprdistributor.md#getcontractaddress)
- [getInterface](hoprdistributor.md#getinterface)
- [isIndexed](hoprdistributor.md#isindexed)

## Constructors

### constructor

• **new HoprDistributor**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |
| `contractInterface` | `ContractInterface` |
| `signerOrProvider?` | `Signer` \| `Provider` |

#### Inherited from

Contract.constructor

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:102

## Properties

### \_deployedPromise

• **\_deployedPromise**: `Promise`<`Contract`\>

#### Inherited from

Contract.\_deployedPromise

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### \_runningEvents

• **\_runningEvents**: `Object`

#### Index signature

▪ [eventTag: `string`]: `RunningEvent`

#### Inherited from

Contract.\_runningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:97

___

### \_wrappedEmits

• **\_wrappedEmits**: `Object`

#### Index signature

▪ [eventTag: `string`]: (...`args`: `any`[]) => `void`

#### Inherited from

Contract.\_wrappedEmits

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:100

___

### address

• `Readonly` **address**: `string`

#### Inherited from

Contract.address

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:75

___

### callStatic

• **callStatic**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `MULTIPLIER` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `MULTIPLIER()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `addAllocations` | (`accounts`: `string`[], `amounts`: `BigNumberish`[], `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: `string`[], `amounts`: `BigNumberish`[], `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `addSchedule` | (`durations`: `BigNumberish`[], `percents`: `BigNumberish`[], `name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `addSchedule(uint128[],uint128[],string)` | (`durations`: `BigNumberish`[], `percents`: `BigNumberish`[], `name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `allocations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `boolean`] & { `amount`: `BigNumber` ; `claimed`: `BigNumber` ; `lastClaim`: `BigNumber` ; `revoked`: `boolean`  }\> |
| `allocations(address,string)` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `boolean`] & { `amount`: `BigNumber` ; `claimed`: `BigNumber` ; `lastClaim`: `BigNumber` ; `revoked`: `boolean`  }\> |
| `claim` | (`scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `claim(string)` | (`scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `claimFor` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `claimFor(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `getClaimable` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getClaimable(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getSchedule` | (`name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`[], `BigNumber`[]]\> |
| `getSchedule(string)` | (`name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`[], `BigNumber`[]]\> |
| `maxMintAmount` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `maxMintAmount()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `renounceOwnership` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceOwnership()` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeAccount` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeAccount(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `startTime` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `startTime()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `totalMinted` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalMinted()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalToBeMinted` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalToBeMinted()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `transferOwnership(address)` | (`newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `updateStartTime` | (`_startTime`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `updateStartTime(uint128)` | (`_startTime`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:547

___

### deployTransaction

• `Readonly` **deployTransaction**: `TransactionResponse`

#### Inherited from

Contract.deployTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:95

___

### estimateGas

• **estimateGas**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `MULTIPLIER` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `MULTIPLIER()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `addAllocations` | (`accounts`: `string`[], `amounts`: `BigNumberish`[], `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: `string`[], `amounts`: `BigNumberish`[], `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `addSchedule` | (`durations`: `BigNumberish`[], `percents`: `BigNumberish`[], `name`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `addSchedule(uint128[],uint128[],string)` | (`durations`: `BigNumberish`[], `percents`: `BigNumberish`[], `name`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `allocations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allocations(address,string)` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `claim` | (`scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `claim(string)` | (`scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `claimFor` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `claimFor(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `getClaimable` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getClaimable(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getSchedule` | (`name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getSchedule(string)` | (`name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `maxMintAmount` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `maxMintAmount()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceOwnership()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeAccount` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeAccount(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `startTime` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `startTime()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalMinted` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalMinted()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalToBeMinted` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalToBeMinted()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferOwnership(address)` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `updateStartTime` | (`_startTime`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `updateStartTime(uint128)` | (`_startTime`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:745

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `AllocationAdded` | (`account`: `string`, `amount`: ``null``, `scheduleName`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `BigNumber`, `string`], `Object`\> |
| `Claimed` | (`account`: `string`, `amount`: ``null``, `scheduleName`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `BigNumber`, `string`], `Object`\> |
| `OwnershipTransferred` | (`previousOwner`: `string`, `newOwner`: `string`) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`], `Object`\> |
| `ScheduleAdded` | (`durations`: ``null``, `percents`: ``null``, `name`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`BigNumber`[], `BigNumber`[], `string`], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:708

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `MULTIPLIER` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `MULTIPLIER()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `addAllocations` | (`accounts`: `string`[], `amounts`: `BigNumberish`[], `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: `string`[], `amounts`: `BigNumberish`[], `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `addSchedule` | (`durations`: `BigNumberish`[], `percents`: `BigNumberish`[], `name`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `addSchedule(uint128[],uint128[],string)` | (`durations`: `BigNumberish`[], `percents`: `BigNumberish`[], `name`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `allocations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `boolean`] & { `amount`: `BigNumber` ; `claimed`: `BigNumber` ; `lastClaim`: `BigNumber` ; `revoked`: `boolean`  }\> |
| `allocations(address,string)` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `boolean`] & { `amount`: `BigNumber` ; `claimed`: `BigNumber` ; `lastClaim`: `BigNumber` ; `revoked`: `boolean`  }\> |
| `claim` | (`scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `claim(string)` | (`scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `claimFor` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `claimFor(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `getClaimable` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `getClaimable(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `getSchedule` | (`name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`[], `BigNumber`[]]\> |
| `getSchedule(string)` | (`name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`[], `BigNumber`[]]\> |
| `maxMintAmount` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `maxMintAmount()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceOwnership()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeAccount` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeAccount(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `startTime` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `startTime()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `totalMinted` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalMinted()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalToBeMinted` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalToBeMinted()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferOwnership(address)` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `updateStartTime` | (`_startTime`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `updateStartTime(uint128)` | (`_startTime`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:213

___

### interface

• **interface**: `HoprDistributorInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:211

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `MULTIPLIER` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `MULTIPLIER()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `addAllocations` | (`accounts`: `string`[], `amounts`: `BigNumberish`[], `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: `string`[], `amounts`: `BigNumberish`[], `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `addSchedule` | (`durations`: `BigNumberish`[], `percents`: `BigNumberish`[], `name`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `addSchedule(uint128[],uint128[],string)` | (`durations`: `BigNumberish`[], `percents`: `BigNumberish`[], `name`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `allocations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allocations(address,string)` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `claim` | (`scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `claim(string)` | (`scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `claimFor` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `claimFor(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `getClaimable` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getClaimable(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getSchedule` | (`name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getSchedule(string)` | (`name`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `maxMintAmount` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `maxMintAmount()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeAccount` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeAccount(address,string)` | (`account`: `string`, `scheduleName`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `startTime` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `startTime()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalMinted` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalMinted()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalToBeMinted` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalToBeMinted()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership(address)` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `updateStartTime` | (`_startTime`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `updateStartTime(uint128)` | (`_startTime`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:896

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

Contract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:78

___

### resolvedAddress

• `Readonly` **resolvedAddress**: `Promise`<`string`\>

#### Inherited from

Contract.resolvedAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:94

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

Contract.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:77

## Methods

### MULTIPLIER

▸ **MULTIPLIER**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:381

___

### MULTIPLIER()

▸ **MULTIPLIER()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:381

___

### \_checkRunningEvents

▸ **_checkRunningEvents**(`runningEvent`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | `RunningEvent` |

#### Returns

`void`

#### Inherited from

Contract.\_checkRunningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:117

___

### \_deployed

▸ **_deployed**(`blockTag?`): `Promise`<`Contract`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockTag?` | `BlockTag` |

#### Returns

`Promise`<`Contract`\>

#### Inherited from

Contract.\_deployed

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:110

___

### \_wrapEvent

▸ **_wrapEvent**(`runningEvent`, `log`, `listener`): `Event`

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | `RunningEvent` |
| `log` | `Log` |
| `listener` | `Listener` |

#### Returns

`Event`

#### Inherited from

Contract.\_wrapEvent

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:118

___

### addAllocations

▸ **addAllocations**(`accounts`, `amounts`, `scheduleName`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accounts` | `string`[] |
| `amounts` | `BigNumberish`[] |
| `scheduleName` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:385

___

### addAllocations(address[],uint128[],string)

▸ **addAllocations(address[],uint128[],string)**(`accounts`, `amounts`, `scheduleName`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accounts` | `string`[] |
| `amounts` | `BigNumberish`[] |
| `scheduleName` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:390

___

### addSchedule

▸ **addSchedule**(`durations`, `percents`, `name`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `durations` | `BigNumberish`[] |
| `percents` | `BigNumberish`[] |
| `name` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:399

___

### addSchedule(uint128[],uint128[],string)

▸ **addSchedule(uint128[],uint128[],string)**(`durations`, `percents`, `name`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `durations` | `BigNumberish`[] |
| `percents` | `BigNumberish`[] |
| `name` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:404

___

### allocations

▸ **allocations**(`arg0`, `arg1`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `boolean`] & { `amount`: `BigNumber` ; `claimed`: `BigNumber` ; `lastClaim`: `BigNumber` ; `revoked`: `boolean`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `arg1` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `boolean`] & { `amount`: `BigNumber` ; `claimed`: `BigNumber` ; `lastClaim`: `BigNumber` ; `revoked`: `boolean`  }\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:413

___

### allocations(address,string)

▸ **allocations(address,string)**(`arg0`, `arg1`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `boolean`] & { `amount`: `BigNumber` ; `claimed`: `BigNumber` ; `lastClaim`: `BigNumber` ; `revoked`: `boolean`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `arg1` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `boolean`] & { `amount`: `BigNumber` ; `claimed`: `BigNumber` ; `lastClaim`: `BigNumber` ; `revoked`: `boolean`  }\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:424

___

### attach

▸ **attach**(`addressOrName`): [`HoprDistributor`](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:172

___

### claim

▸ **claim**(`scheduleName`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `scheduleName` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:439

___

### claim(string)

▸ **claim(string)**(`scheduleName`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `scheduleName` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:442

___

### claimFor

▸ **claimFor**(`account`, `scheduleName`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `scheduleName` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:449

___

### claimFor(address,string)

▸ **claimFor(address,string)**(`account`, `scheduleName`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `scheduleName` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:453

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprDistributor`](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:171

___

### deployed

▸ **deployed**(): `Promise`<[`HoprDistributor`](hoprdistributor.md)\>

#### Returns

`Promise`<[`HoprDistributor`](hoprdistributor.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:173

___

### emit

▸ **emit**(`eventName`, ...`args`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `EventFilter` |
| `...args` | `any`[] |

#### Returns

`boolean`

#### Inherited from

Contract.emit

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:123

___

### fallback

▸ **fallback**(`overrides?`): `Promise`<`TransactionResponse`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `TransactionRequest` |

#### Returns

`Promise`<`TransactionResponse`\>

#### Inherited from

Contract.fallback

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:111

___

### getClaimable

▸ **getClaimable**(`account`, `scheduleName`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `scheduleName` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:461

___

### getClaimable(address,string)

▸ **getClaimable(address,string)**(`account`, `scheduleName`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `scheduleName` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:465

___

### getSchedule

▸ **getSchedule**(`name`, `overrides?`): `Promise`<[`BigNumber`[], `BigNumber`[]]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `name` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`[], `BigNumber`[]]\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:473

___

### getSchedule(string)

▸ **getSchedule(string)**(`name`, `overrides?`): `Promise`<[`BigNumber`[], `BigNumber`[]]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `name` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`[], `BigNumber`[]]\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:476

___

### listenerCount

▸ **listenerCount**(`eventName?`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` \| `EventFilter` |

#### Returns

`number`

#### Inherited from

Contract.listenerCount

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:124

___

### listeners

▸ **listeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter?`): [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\>[]

#### Overrides

Contract.listeners

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:175

▸ **listeners**(`eventName?`): `Listener`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

`Listener`[]

#### Overrides

Contract.listeners

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:198

___

### maxMintAmount

▸ **maxMintAmount**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:483

___

### maxMintAmount()

▸ **maxMintAmount()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:483

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprDistributor`](hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:178

▸ **off**(`eventName`, `listener`): [`HoprDistributor`](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:199

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprDistributor`](hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:182

▸ **on**(`eventName`, `listener`): [`HoprDistributor`](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:200

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprDistributor`](hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:186

▸ **once**(`eventName`, `listener`): [`HoprDistributor`](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:201

___

### owner

▸ **owner**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:487

___

### owner()

▸ **owner()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:487

___

### queryFilter

▸ **queryFilter**<`EventArgsArray`, `EventArgsObject`\>(`event`, `fromBlockOrBlockhash?`, `toBlock?`): `Promise`<[`TypedEvent`](../interfaces/typedevent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `fromBlockOrBlockhash?` | `string` \| `number` |
| `toBlock?` | `string` \| `number` |

#### Returns

`Promise`<[`TypedEvent`](../interfaces/typedevent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Overrides

Contract.queryFilter

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:205

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`HoprDistributor`](hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:194

▸ **removeAllListeners**(`eventName?`): [`HoprDistributor`](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:203

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprDistributor`](hoprdistributor.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:190

▸ **removeListener**(`eventName`, `listener`): [`HoprDistributor`](hoprdistributor.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprDistributor`](hoprdistributor.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:202

___

### renounceOwnership

▸ **renounceOwnership**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:491

___

### renounceOwnership()

▸ **renounceOwnership()**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:493

___

### revokeAccount

▸ **revokeAccount**(`account`, `scheduleName`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `scheduleName` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:499

___

### revokeAccount(address,string)

▸ **revokeAccount(address,string)**(`account`, `scheduleName`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `scheduleName` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:503

___

### startTime

▸ **startTime**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:511

___

### startTime()

▸ **startTime()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:511

___

### token

▸ **token**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:515

___

### token()

▸ **token()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:515

___

### totalMinted

▸ **totalMinted**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:519

___

### totalMinted()

▸ **totalMinted()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:519

___

### totalToBeMinted

▸ **totalToBeMinted**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:523

___

### totalToBeMinted()

▸ **totalToBeMinted()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:523

___

### transferOwnership

▸ **transferOwnership**(`newOwner`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newOwner` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:527

___

### transferOwnership(address)

▸ **transferOwnership(address)**(`newOwner`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newOwner` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:530

___

### updateStartTime

▸ **updateStartTime**(`_startTime`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_startTime` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:537

___

### updateStartTime(uint128)

▸ **updateStartTime(uint128)**(`_startTime`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_startTime` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprDistributor.d.ts:540

___

### getContractAddress

▸ `Static` **getContractAddress**(`transaction`): `string`

#### Parameters

| Name | Type |
| :------ | :------ |
| `transaction` | `Object` |
| `transaction.from` | `string` |
| `transaction.nonce` | `BigNumberish` |

#### Returns

`string`

#### Inherited from

Contract.getContractAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:104

___

### getInterface

▸ `Static` **getInterface**(`contractInterface`): `Interface`

#### Parameters

| Name | Type |
| :------ | :------ |
| `contractInterface` | `ContractInterface` |

#### Returns

`Interface`

#### Inherited from

Contract.getInterface

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:108

___

### isIndexed

▸ `Static` **isIndexed**(`value`): value is Indexed

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | `any` |

#### Returns

value is Indexed

#### Inherited from

Contract.isIndexed

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:114
