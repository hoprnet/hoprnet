[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprDistributor](../modules/contracts_hoprdistributor.md) / HoprDistributor

# Class: HoprDistributor

[contracts/HoprDistributor](../modules/contracts_hoprdistributor.md).HoprDistributor

## Hierarchy

- _Contract_

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
- [MULTIPLIER()](<contracts_hoprdistributor.hoprdistributor.md#multiplier()>)
- [\_checkRunningEvents](contracts_hoprdistributor.hoprdistributor.md#_checkrunningevents)
- [\_deployed](contracts_hoprdistributor.hoprdistributor.md#_deployed)
- [\_wrapEvent](contracts_hoprdistributor.hoprdistributor.md#_wrapevent)
- [addAllocations](contracts_hoprdistributor.hoprdistributor.md#addallocations)
- [addAllocations(address[],uint128[],string)](<contracts_hoprdistributor.hoprdistributor.md#addallocations(address[],uint128[],string)>)
- [addSchedule](contracts_hoprdistributor.hoprdistributor.md#addschedule)
- [addSchedule(uint128[],uint128[],string)](<contracts_hoprdistributor.hoprdistributor.md#addschedule(uint128[],uint128[],string)>)
- [allocations](contracts_hoprdistributor.hoprdistributor.md#allocations)
- [allocations(address,string)](<contracts_hoprdistributor.hoprdistributor.md#allocations(address,string)>)
- [attach](contracts_hoprdistributor.hoprdistributor.md#attach)
- [claim](contracts_hoprdistributor.hoprdistributor.md#claim)
- [claim(string)](<contracts_hoprdistributor.hoprdistributor.md#claim(string)>)
- [claimFor](contracts_hoprdistributor.hoprdistributor.md#claimfor)
- [claimFor(address,string)](<contracts_hoprdistributor.hoprdistributor.md#claimfor(address,string)>)
- [connect](contracts_hoprdistributor.hoprdistributor.md#connect)
- [deployed](contracts_hoprdistributor.hoprdistributor.md#deployed)
- [emit](contracts_hoprdistributor.hoprdistributor.md#emit)
- [fallback](contracts_hoprdistributor.hoprdistributor.md#fallback)
- [getClaimable](contracts_hoprdistributor.hoprdistributor.md#getclaimable)
- [getClaimable(address,string)](<contracts_hoprdistributor.hoprdistributor.md#getclaimable(address,string)>)
- [getSchedule](contracts_hoprdistributor.hoprdistributor.md#getschedule)
- [getSchedule(string)](<contracts_hoprdistributor.hoprdistributor.md#getschedule(string)>)
- [listenerCount](contracts_hoprdistributor.hoprdistributor.md#listenercount)
- [listeners](contracts_hoprdistributor.hoprdistributor.md#listeners)
- [maxMintAmount](contracts_hoprdistributor.hoprdistributor.md#maxmintamount)
- [maxMintAmount()](<contracts_hoprdistributor.hoprdistributor.md#maxmintamount()>)
- [off](contracts_hoprdistributor.hoprdistributor.md#off)
- [on](contracts_hoprdistributor.hoprdistributor.md#on)
- [once](contracts_hoprdistributor.hoprdistributor.md#once)
- [owner](contracts_hoprdistributor.hoprdistributor.md#owner)
- [owner()](<contracts_hoprdistributor.hoprdistributor.md#owner()>)
- [queryFilter](contracts_hoprdistributor.hoprdistributor.md#queryfilter)
- [removeAllListeners](contracts_hoprdistributor.hoprdistributor.md#removealllisteners)
- [removeListener](contracts_hoprdistributor.hoprdistributor.md#removelistener)
- [renounceOwnership](contracts_hoprdistributor.hoprdistributor.md#renounceownership)
- [renounceOwnership()](<contracts_hoprdistributor.hoprdistributor.md#renounceownership()>)
- [revokeAccount](contracts_hoprdistributor.hoprdistributor.md#revokeaccount)
- [revokeAccount(address,string)](<contracts_hoprdistributor.hoprdistributor.md#revokeaccount(address,string)>)
- [startTime](contracts_hoprdistributor.hoprdistributor.md#starttime)
- [startTime()](<contracts_hoprdistributor.hoprdistributor.md#starttime()>)
- [token](contracts_hoprdistributor.hoprdistributor.md#token)
- [token()](<contracts_hoprdistributor.hoprdistributor.md#token()>)
- [totalMinted](contracts_hoprdistributor.hoprdistributor.md#totalminted)
- [totalMinted()](<contracts_hoprdistributor.hoprdistributor.md#totalminted()>)
- [totalToBeMinted](contracts_hoprdistributor.hoprdistributor.md#totaltobeminted)
- [totalToBeMinted()](<contracts_hoprdistributor.hoprdistributor.md#totaltobeminted()>)
- [transferOwnership](contracts_hoprdistributor.hoprdistributor.md#transferownership)
- [transferOwnership(address)](<contracts_hoprdistributor.hoprdistributor.md#transferownership(address)>)
- [updateStartTime](contracts_hoprdistributor.hoprdistributor.md#updatestarttime)
- [updateStartTime(uint128)](<contracts_hoprdistributor.hoprdistributor.md#updatestarttime(uint128)>)
- [getContractAddress](contracts_hoprdistributor.hoprdistributor.md#getcontractaddress)
- [getInterface](contracts_hoprdistributor.hoprdistributor.md#getinterface)
- [isIndexed](contracts_hoprdistributor.hoprdistributor.md#isindexed)

## Constructors

### constructor

\+ **new HoprDistributor**(`addressOrName`: _string_, `contractInterface`: ContractInterface, `signerOrProvider?`: _Provider_ \| _Signer_): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name                | Type                   |
| :------------------ | :--------------------- |
| `addressOrName`     | _string_               |
| `contractInterface` | ContractInterface      |
| `signerOrProvider?` | _Provider_ \| _Signer_ |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Inherited from: Contract.constructor

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:98

## Properties

### \_deployedPromise

• **\_deployedPromise**: _Promise_<Contract\>

Inherited from: Contract.\_deployedPromise

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:92

---

### \_runningEvents

• **\_runningEvents**: _object_

#### Type declaration

Inherited from: Contract.\_runningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:93

---

### \_wrappedEmits

• **\_wrappedEmits**: _object_

#### Type declaration

Inherited from: Contract.\_wrappedEmits

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:96

---

### address

• `Readonly` **address**: _string_

Inherited from: Contract.address

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:71

---

### callStatic

• **callStatic**: _object_

#### Type declaration

| Name                                         | Type                                                                                                                                                                                                                                       |
| :------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MULTIPLIER`                                 | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `MULTIPLIER()`                               | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `addAllocations`                             | (`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                             |
| `addAllocations(address[],uint128[],string)` | (`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                             |
| `addSchedule`                                | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                               |
| `addSchedule(uint128[],uint128[],string)`    | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                               |
| `allocations`                                | (`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: _BigNumber_ ; `claimed`: _BigNumber_ ; `lastClaim`: _BigNumber_ ; `revoked`: _boolean_ }\> |
| `allocations(address,string)`                | (`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: _BigNumber_ ; `claimed`: _BigNumber_ ; `lastClaim`: _BigNumber_ ; `revoked`: _boolean_ }\> |
| `claim`                                      | (`scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                |
| `claim(string)`                              | (`scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                |
| `claimFor`                                   | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                           |
| `claimFor(address,string)`                   | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                           |
| `getClaimable`                               | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                      |
| `getClaimable(address,string)`               | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                      |
| `getSchedule`                                | (`name`: _string_, `overrides?`: CallOverrides) => _Promise_<[_BigNumber_[], _BigNumber_[]]\>                                                                                                                                              |
| `getSchedule(string)`                        | (`name`: _string_, `overrides?`: CallOverrides) => _Promise_<[_BigNumber_[], _BigNumber_[]]\>                                                                                                                                              |
| `maxMintAmount`                              | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `maxMintAmount()`                            | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `owner`                                      | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                        |
| `owner()`                                    | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                        |
| `renounceOwnership`                          | (`overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                          |
| `renounceOwnership()`                        | (`overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                          |
| `revokeAccount`                              | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                           |
| `revokeAccount(address,string)`              | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                           |
| `startTime`                                  | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `startTime()`                                | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `token`                                      | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                        |
| `token()`                                    | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                        |
| `totalMinted`                                | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `totalMinted()`                              | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `totalToBeMinted`                            | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `totalToBeMinted()`                          | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                     |
| `transferOwnership`                          | (`newOwner`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                    |
| `transferOwnership(address)`                 | (`newOwner`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                    |
| `updateStartTime`                            | (`_startTime`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                              |
| `updateStartTime(uint128)`                   | (`_startTime`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                              |

Overrides: Contract.callStatic

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:547

---

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

---

### estimateGas

• **estimateGas**: _object_

#### Type declaration

| Name                                         | Type                                                                                                                                                                          |
| :------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MULTIPLIER`                                 | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `MULTIPLIER()`                               | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `addAllocations`                             | (`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `addSchedule`                                | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>   |
| `addSchedule(uint128[],uint128[],string)`    | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>   |
| `allocations`                                | (`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                    |
| `allocations(address,string)`                | (`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                    |
| `claim`                                      | (`scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                    |
| `claim(string)`                              | (`scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                    |
| `claimFor`                                   | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                               |
| `claimFor(address,string)`                   | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                               |
| `getClaimable`                               | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                         |
| `getClaimable(address,string)`               | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                         |
| `getSchedule`                                | (`name`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                      |
| `getSchedule(string)`                        | (`name`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                      |
| `maxMintAmount`                              | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `maxMintAmount()`                            | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `owner`                                      | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `owner()`                                    | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `renounceOwnership`                          | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                              |
| `renounceOwnership()`                        | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                              |
| `revokeAccount`                              | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                               |
| `revokeAccount(address,string)`              | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                               |
| `startTime`                                  | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `startTime()`                                | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `token`                                      | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `token()`                                    | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `totalMinted`                                | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `totalMinted()`                              | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `totalToBeMinted`                            | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `totalToBeMinted()`                          | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                        |
| `transferOwnership`                          | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                        |
| `transferOwnership(address)`                 | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                        |
| `updateStartTime`                            | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                  |
| `updateStartTime(uint128)`                   | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                  |

Overrides: Contract.estimateGas

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:745

---

### filters

• **filters**: _object_

#### Type declaration

| Name                   | Type                                                                                                                                                                                                                                                             |
| :--------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AllocationAdded`      | (`account`: _string_, `amount`: `null`, `scheduleName`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *BigNumber*, *string*], { `account`: _string_ ; `amount`: _BigNumber_ ; `scheduleName`: _string_ }\>     |
| `Claimed`              | (`account`: _string_, `amount`: `null`, `scheduleName`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *BigNumber*, *string*], { `account`: _string_ ; `amount`: _BigNumber_ ; `scheduleName`: _string_ }\>     |
| `OwnershipTransferred` | (`previousOwner`: _string_, `newOwner`: _string_) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `newOwner`: _string_ ; `previousOwner`: _string_ }\>                                                      |
| `ScheduleAdded`        | (`durations`: `null`, `percents`: `null`, `name`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[_BigNumber_[], _BigNumber_[], _string_], { `durations`: _BigNumber_[] ; `name`: _string_ ; `percents`: _BigNumber_[] }\> |

Overrides: Contract.filters

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:708

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                         | Type                                                                                                                                                                                                                                       |
| :------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MULTIPLIER`                                 | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `MULTIPLIER()`                               | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `addAllocations`                             | (`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                    |
| `addAllocations(address[],uint128[],string)` | (`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                    |
| `addSchedule`                                | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                      |
| `addSchedule(uint128[],uint128[],string)`    | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                      |
| `allocations`                                | (`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: _BigNumber_ ; `claimed`: _BigNumber_ ; `lastClaim`: _BigNumber_ ; `revoked`: _boolean_ }\> |
| `allocations(address,string)`                | (`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: _BigNumber_ ; `claimed`: _BigNumber_ ; `lastClaim`: _BigNumber_ ; `revoked`: _boolean_ }\> |
| `claim`                                      | (`scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                       |
| `claim(string)`                              | (`scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                       |
| `claimFor`                                   | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                  |
| `claimFor(address,string)`                   | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                  |
| `getClaimable`                               | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                  |
| `getClaimable(address,string)`               | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                  |
| `getSchedule`                                | (`name`: _string_, `overrides?`: CallOverrides) => _Promise_<[_BigNumber_[], _BigNumber_[]]\>                                                                                                                                              |
| `getSchedule(string)`                        | (`name`: _string_, `overrides?`: CallOverrides) => _Promise_<[_BigNumber_[], _BigNumber_[]]\>                                                                                                                                              |
| `maxMintAmount`                              | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `maxMintAmount()`                            | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `owner`                                      | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                    |
| `owner()`                                    | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                    |
| `renounceOwnership`                          | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                 |
| `renounceOwnership()`                        | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                 |
| `revokeAccount`                              | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                  |
| `revokeAccount(address,string)`              | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                  |
| `startTime`                                  | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `startTime()`                                | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `token`                                      | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                    |
| `token()`                                    | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                    |
| `totalMinted`                                | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `totalMinted()`                              | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `totalToBeMinted`                            | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `totalToBeMinted()`                          | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                 |
| `transferOwnership`                          | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                           |
| `transferOwnership(address)`                 | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                           |
| `updateStartTime`                            | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                     |
| `updateStartTime(uint128)`                   | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                     |

Overrides: Contract.functions

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:213

---

### interface

• **interface**: [_HoprDistributorInterface_](../interfaces/contracts_hoprdistributor.hoprdistributorinterface.md)

Overrides: Contract.interface

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:211

---

### populateTransaction

• **populateTransaction**: _object_

#### Type declaration

| Name                                         | Type                                                                                                                                                                                     |
| :------------------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MULTIPLIER`                                 | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `MULTIPLIER()`                               | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `addAllocations`                             | (`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `addAllocations(address[],uint128[],string)` | (`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `addSchedule`                                | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>   |
| `addSchedule(uint128[],uint128[],string)`    | (`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>   |
| `allocations`                                | (`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                    |
| `allocations(address,string)`                | (`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                    |
| `claim`                                      | (`scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                    |
| `claim(string)`                              | (`scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                    |
| `claimFor`                                   | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                               |
| `claimFor(address,string)`                   | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                               |
| `getClaimable`                               | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                         |
| `getClaimable(address,string)`               | (`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                         |
| `getSchedule`                                | (`name`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                      |
| `getSchedule(string)`                        | (`name`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                      |
| `maxMintAmount`                              | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `maxMintAmount()`                            | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `owner`                                      | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `owner()`                                    | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `renounceOwnership`                          | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                              |
| `renounceOwnership()`                        | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                              |
| `revokeAccount`                              | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                               |
| `revokeAccount(address,string)`              | (`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                               |
| `startTime`                                  | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `startTime()`                                | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `token`                                      | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `token()`                                    | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `totalMinted`                                | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `totalMinted()`                              | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `totalToBeMinted`                            | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `totalToBeMinted()`                          | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                        |
| `transferOwnership`                          | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                        |
| `transferOwnership(address)`                 | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                        |
| `updateStartTime`                            | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                  |
| `updateStartTime(uint128)`                   | (`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                  |

Overrides: Contract.populateTransaction

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:896

---

### provider

• `Readonly` **provider**: _Provider_

Inherited from: Contract.provider

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:74

---

### resolvedAddress

• `Readonly` **resolvedAddress**: _Promise_<string\>

Inherited from: Contract.resolvedAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:90

---

### signer

• `Readonly` **signer**: _Signer_

Inherited from: Contract.signer

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:73

## Methods

### MULTIPLIER

▸ **MULTIPLIER**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:381

---

### MULTIPLIER()

▸ **MULTIPLIER()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:381

---

### \_checkRunningEvents

▸ **\_checkRunningEvents**(`runningEvent`: _RunningEvent_): _void_

#### Parameters

| Name           | Type           |
| :------------- | :------------- |
| `runningEvent` | _RunningEvent_ |

**Returns:** _void_

Inherited from: Contract.\_checkRunningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:113

---

### \_deployed

▸ **\_deployed**(`blockTag?`: BlockTag): _Promise_<Contract\>

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `blockTag?` | BlockTag |

**Returns:** _Promise_<Contract\>

Inherited from: Contract.\_deployed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:106

---

### \_wrapEvent

▸ **\_wrapEvent**(`runningEvent`: _RunningEvent_, `log`: Log, `listener`: Listener): Event

#### Parameters

| Name           | Type           |
| :------------- | :------------- |
| `runningEvent` | _RunningEvent_ |
| `log`          | Log            |
| `listener`     | Listener       |

**Returns:** Event

Inherited from: Contract.\_wrapEvent

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:114

---

### addAllocations

▸ **addAllocations**(`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `accounts`     | _string_[]                                              |
| `amounts`      | BigNumberish[]                                          |
| `scheduleName` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:385

---

### addAllocations(address[],uint128[],string)

▸ **addAllocations(address[],uint128[],string)**(`accounts`: _string_[], `amounts`: BigNumberish[], `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `accounts`     | _string_[]                                              |
| `amounts`      | BigNumberish[]                                          |
| `scheduleName` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:390

---

### addSchedule

▸ **addSchedule**(`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `durations`  | BigNumberish[]                                          |
| `percents`   | BigNumberish[]                                          |
| `name`       | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:399

---

### addSchedule(uint128[],uint128[],string)

▸ **addSchedule(uint128[],uint128[],string)**(`durations`: BigNumberish[], `percents`: BigNumberish[], `name`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `durations`  | BigNumberish[]                                          |
| `percents`   | BigNumberish[]                                          |
| `name`       | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:404

---

### allocations

▸ **allocations**(`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides): _Promise_<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: _BigNumber_ ; `claimed`: _BigNumber_ ; `lastClaim`: _BigNumber_ ; `revoked`: _boolean_ }\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `arg0`       | _string_      |
| `arg1`       | _string_      |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: _BigNumber_ ; `claimed`: _BigNumber_ ; `lastClaim`: _BigNumber_ ; `revoked`: _boolean_ }\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:413

---

### allocations(address,string)

▸ **allocations(address,string)**(`arg0`: _string_, `arg1`: _string_, `overrides?`: CallOverrides): _Promise_<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: _BigNumber_ ; `claimed`: _BigNumber_ ; `lastClaim`: _BigNumber_ ; `revoked`: _boolean_ }\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `arg0`       | _string_      |
| `arg1`       | _string_      |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[*BigNumber*, *BigNumber*, *BigNumber*, *boolean*] & { `amount`: _BigNumber_ ; `claimed`: _BigNumber_ ; `lastClaim`: _BigNumber_ ; `revoked`: _boolean_ }\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:424

---

### attach

▸ **attach**(`addressOrName`: _string_): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name            | Type     |
| :-------------- | :------- |
| `addressOrName` | _string_ |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.attach

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:172

---

### claim

▸ **claim**(`scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `scheduleName` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:439

---

### claim(string)

▸ **claim(string)**(`scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `scheduleName` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:442

---

### claimFor

▸ **claimFor**(`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `account`      | _string_                                                |
| `scheduleName` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:449

---

### claimFor(address,string)

▸ **claimFor(address,string)**(`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `account`      | _string_                                                |
| `scheduleName` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:453

---

### connect

▸ **connect**(`signerOrProvider`: _string_ \| _Provider_ \| _Signer_): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name               | Type                               |
| :----------------- | :--------------------------------- |
| `signerOrProvider` | _string_ \| _Provider_ \| _Signer_ |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.connect

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:171

---

### deployed

▸ **deployed**(): _Promise_<[_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)\>

**Returns:** _Promise_<[_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)\>

Overrides: Contract.deployed

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:173

---

### emit

▸ **emit**(`eventName`: _string_ \| EventFilter, ...`args`: _any_[]): _boolean_

#### Parameters

| Name        | Type                    |
| :---------- | :---------------------- |
| `eventName` | _string_ \| EventFilter |
| `...args`   | _any_[]                 |

**Returns:** _boolean_

Inherited from: Contract.emit

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:119

---

### fallback

▸ **fallback**(`overrides?`: TransactionRequest): _Promise_<TransactionResponse\>

#### Parameters

| Name         | Type               |
| :----------- | :----------------- |
| `overrides?` | TransactionRequest |

**Returns:** _Promise_<TransactionResponse\>

Inherited from: Contract.fallback

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:107

---

### getClaimable

▸ **getClaimable**(`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name           | Type          |
| :------------- | :------------ |
| `account`      | _string_      |
| `scheduleName` | _string_      |
| `overrides?`   | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:461

---

### getClaimable(address,string)

▸ **getClaimable(address,string)**(`account`: _string_, `scheduleName`: _string_, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name           | Type          |
| :------------- | :------------ |
| `account`      | _string_      |
| `scheduleName` | _string_      |
| `overrides?`   | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:465

---

### getSchedule

▸ **getSchedule**(`name`: _string_, `overrides?`: CallOverrides): _Promise_<[_BigNumber_[], _BigNumber_[]]\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `name`       | _string_      |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[_BigNumber_[], _BigNumber_[]]\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:473

---

### getSchedule(string)

▸ **getSchedule(string)**(`name`: _string_, `overrides?`: CallOverrides): _Promise_<[_BigNumber_[], _BigNumber_[]]\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `name`       | _string_      |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[_BigNumber_[], _BigNumber_[]]\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:476

---

### listenerCount

▸ **listenerCount**(`eventName?`: _string_ \| EventFilter): _number_

#### Parameters

| Name         | Type                    |
| :----------- | :---------------------- |
| `eventName?` | _string_ \| EventFilter |

**Returns:** _number_

Inherited from: Contract.listenerCount

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:120

---

### listeners

▸ **listeners**<EventArgsArray, EventArgsObject\>(`eventFilter?`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name           | Type                                                                                                        |
| :------------- | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter?` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:175

▸ **listeners**(`eventName?`: _string_): Listener[]

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:198

---

### maxMintAmount

▸ **maxMintAmount**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:483

---

### maxMintAmount()

▸ **maxMintAmount()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:483

---

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:178

▸ **off**(`eventName`: _string_, `listener`: Listener): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:199

---

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:182

▸ **on**(`eventName`: _string_, `listener`: Listener): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:200

---

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:186

▸ **once**(`eventName`: _string_, `listener`: Listener): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:201

---

### owner

▸ **owner**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:487

---

### owner()

▸ **owner()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:487

---

### queryFilter

▸ **queryFilter**<EventArgsArray, EventArgsObject\>(`event`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `fromBlockOrBlockhash?`: _string_ \| _number_, `toBlock?`: _string_ \| _number_): _Promise_<[_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name                    | Type                                                                                                        |
| :---------------------- | :---------------------------------------------------------------------------------------------------------- |
| `event`                 | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `fromBlockOrBlockhash?` | _string_ \| _number_                                                                                        |
| `toBlock?`              | _string_ \| _number_                                                                                        |

**Returns:** _Promise_<[_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

Overrides: Contract.queryFilter

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:205

---

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:194

▸ **removeAllListeners**(`eventName?`: _string_): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:203

---

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:190

▸ **removeListener**(`eventName`: _string_, `listener`: Listener): [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprDistributor_](contracts_hoprdistributor.hoprdistributor.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:202

---

### renounceOwnership

▸ **renounceOwnership**(`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:491

---

### renounceOwnership()

▸ **renounceOwnership()**(`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:493

---

### revokeAccount

▸ **revokeAccount**(`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `account`      | _string_                                                |
| `scheduleName` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:499

---

### revokeAccount(address,string)

▸ **revokeAccount(address,string)**(`account`: _string_, `scheduleName`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `account`      | _string_                                                |
| `scheduleName` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:503

---

### startTime

▸ **startTime**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:511

---

### startTime()

▸ **startTime()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:511

---

### token

▸ **token**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:515

---

### token()

▸ **token()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:515

---

### totalMinted

▸ **totalMinted**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:519

---

### totalMinted()

▸ **totalMinted()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:519

---

### totalToBeMinted

▸ **totalToBeMinted**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:523

---

### totalToBeMinted()

▸ **totalToBeMinted()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:523

---

### transferOwnership

▸ **transferOwnership**(`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `newOwner`   | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:527

---

### transferOwnership(address)

▸ **transferOwnership(address)**(`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `newOwner`   | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:530

---

### updateStartTime

▸ **updateStartTime**(`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `_startTime` | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:537

---

### updateStartTime(uint128)

▸ **updateStartTime(uint128)**(`_startTime`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `_startTime` | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprDistributor.d.ts:540

---

### getContractAddress

▸ `Static` **getContractAddress**(`transaction`: { `from`: _string_ ; `nonce`: BigNumberish }): _string_

#### Parameters

| Name                | Type         |
| :------------------ | :----------- |
| `transaction`       | _object_     |
| `transaction.from`  | _string_     |
| `transaction.nonce` | BigNumberish |

**Returns:** _string_

Inherited from: Contract.getContractAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:100

---

### getInterface

▸ `Static` **getInterface**(`contractInterface`: ContractInterface): _Interface_

#### Parameters

| Name                | Type              |
| :------------------ | :---------------- |
| `contractInterface` | ContractInterface |

**Returns:** _Interface_

Inherited from: Contract.getInterface

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:104

---

### isIndexed

▸ `Static` **isIndexed**(`value`: _any_): value is Indexed

#### Parameters

| Name    | Type  |
| :------ | :---- |
| `value` | _any_ |

**Returns:** value is Indexed

Inherited from: Contract.isIndexed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:110
