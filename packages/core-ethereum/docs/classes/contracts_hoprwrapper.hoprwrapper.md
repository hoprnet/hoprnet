[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprWrapper](../modules/contracts_hoprwrapper.md) / HoprWrapper

# Class: HoprWrapper

[contracts/HoprWrapper](../modules/contracts_hoprwrapper.md).HoprWrapper

## Hierarchy

- _Contract_

  ↳ **HoprWrapper**

## Table of contents

### Constructors

- [constructor](contracts_hoprwrapper.hoprwrapper.md#constructor)

### Properties

- [\_deployedPromise](contracts_hoprwrapper.hoprwrapper.md#_deployedpromise)
- [\_runningEvents](contracts_hoprwrapper.hoprwrapper.md#_runningevents)
- [\_wrappedEmits](contracts_hoprwrapper.hoprwrapper.md#_wrappedemits)
- [address](contracts_hoprwrapper.hoprwrapper.md#address)
- [callStatic](contracts_hoprwrapper.hoprwrapper.md#callstatic)
- [deployTransaction](contracts_hoprwrapper.hoprwrapper.md#deploytransaction)
- [estimateGas](contracts_hoprwrapper.hoprwrapper.md#estimategas)
- [filters](contracts_hoprwrapper.hoprwrapper.md#filters)
- [functions](contracts_hoprwrapper.hoprwrapper.md#functions)
- [interface](contracts_hoprwrapper.hoprwrapper.md#interface)
- [populateTransaction](contracts_hoprwrapper.hoprwrapper.md#populatetransaction)
- [provider](contracts_hoprwrapper.hoprwrapper.md#provider)
- [resolvedAddress](contracts_hoprwrapper.hoprwrapper.md#resolvedaddress)
- [signer](contracts_hoprwrapper.hoprwrapper.md#signer)

### Methods

- [TOKENS_RECIPIENT_INTERFACE_HASH](contracts_hoprwrapper.hoprwrapper.md#tokens_recipient_interface_hash)
- [TOKENS_RECIPIENT_INTERFACE_HASH()](<contracts_hoprwrapper.hoprwrapper.md#tokens_recipient_interface_hash()>)
- [\_checkRunningEvents](contracts_hoprwrapper.hoprwrapper.md#_checkrunningevents)
- [\_deployed](contracts_hoprwrapper.hoprwrapper.md#_deployed)
- [\_wrapEvent](contracts_hoprwrapper.hoprwrapper.md#_wrapevent)
- [attach](contracts_hoprwrapper.hoprwrapper.md#attach)
- [canImplementInterfaceForAddress](contracts_hoprwrapper.hoprwrapper.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](<contracts_hoprwrapper.hoprwrapper.md#canimplementinterfaceforaddress(bytes32,address)>)
- [connect](contracts_hoprwrapper.hoprwrapper.md#connect)
- [deployed](contracts_hoprwrapper.hoprwrapper.md#deployed)
- [emit](contracts_hoprwrapper.hoprwrapper.md#emit)
- [fallback](contracts_hoprwrapper.hoprwrapper.md#fallback)
- [listenerCount](contracts_hoprwrapper.hoprwrapper.md#listenercount)
- [listeners](contracts_hoprwrapper.hoprwrapper.md#listeners)
- [off](contracts_hoprwrapper.hoprwrapper.md#off)
- [on](contracts_hoprwrapper.hoprwrapper.md#on)
- [onTokenTransfer](contracts_hoprwrapper.hoprwrapper.md#ontokentransfer)
- [onTokenTransfer(address,uint256,bytes)](<contracts_hoprwrapper.hoprwrapper.md#ontokentransfer(address,uint256,bytes)>)
- [once](contracts_hoprwrapper.hoprwrapper.md#once)
- [owner](contracts_hoprwrapper.hoprwrapper.md#owner)
- [owner()](<contracts_hoprwrapper.hoprwrapper.md#owner()>)
- [queryFilter](contracts_hoprwrapper.hoprwrapper.md#queryfilter)
- [recoverTokens](contracts_hoprwrapper.hoprwrapper.md#recovertokens)
- [recoverTokens()](<contracts_hoprwrapper.hoprwrapper.md#recovertokens()>)
- [removeAllListeners](contracts_hoprwrapper.hoprwrapper.md#removealllisteners)
- [removeListener](contracts_hoprwrapper.hoprwrapper.md#removelistener)
- [renounceOwnership](contracts_hoprwrapper.hoprwrapper.md#renounceownership)
- [renounceOwnership()](<contracts_hoprwrapper.hoprwrapper.md#renounceownership()>)
- [tokensReceived](contracts_hoprwrapper.hoprwrapper.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](<contracts_hoprwrapper.hoprwrapper.md#tokensreceived(address,address,address,uint256,bytes,bytes)>)
- [transferOwnership](contracts_hoprwrapper.hoprwrapper.md#transferownership)
- [transferOwnership(address)](<contracts_hoprwrapper.hoprwrapper.md#transferownership(address)>)
- [wxHOPR](contracts_hoprwrapper.hoprwrapper.md#wxhopr)
- [wxHOPR()](<contracts_hoprwrapper.hoprwrapper.md#wxhopr()>)
- [xHOPR](contracts_hoprwrapper.hoprwrapper.md#xhopr)
- [xHOPR()](<contracts_hoprwrapper.hoprwrapper.md#xhopr()>)
- [xHoprAmount](contracts_hoprwrapper.hoprwrapper.md#xhopramount)
- [xHoprAmount()](<contracts_hoprwrapper.hoprwrapper.md#xhopramount()>)
- [getContractAddress](contracts_hoprwrapper.hoprwrapper.md#getcontractaddress)
- [getInterface](contracts_hoprwrapper.hoprwrapper.md#getinterface)
- [isIndexed](contracts_hoprwrapper.hoprwrapper.md#isindexed)

## Constructors

### constructor

\+ **new HoprWrapper**(`addressOrName`: _string_, `contractInterface`: ContractInterface, `signerOrProvider?`: _Provider_ \| _Signer_): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Parameters

| Name                | Type                   |
| :------------------ | :--------------------- |
| `addressOrName`     | _string_               |
| `contractInterface` | ContractInterface      |
| `signerOrProvider?` | _Provider_ \| _Signer_ |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

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

| Name                                                          | Type                                                                                                                                                                                |
| :------------------------------------------------------------ | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                             | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `canImplementInterfaceForAddress`                             | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<string\>                                                                                |
| `canImplementInterfaceForAddress(bytes32,address)`            | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<string\>                                                                                |
| `onTokenTransfer`                                             | (`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                   |
| `onTokenTransfer(address,uint256,bytes)`                      | (`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                   |
| `owner`                                                       | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `owner()`                                                     | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `recoverTokens`                                               | (`overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                   |
| `recoverTokens()`                                             | (`overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                   |
| `renounceOwnership`                                           | (`overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                   |
| `renounceOwnership()`                                         | (`overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                   |
| `tokensReceived`                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\> |
| `transferOwnership`                                           | (`newOwner`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                             |
| `transferOwnership(address)`                                  | (`newOwner`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                             |
| `wxHOPR`                                                      | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `wxHOPR()`                                                    | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `xHOPR`                                                       | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `xHOPR()`                                                     | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `xHoprAmount`                                                 | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                              |
| `xHoprAmount()`                                               | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                              |

Overrides: Contract.callStatic

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:355

---

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

---

### estimateGas

• **estimateGas**: _object_

#### Type declaration

| Name                                                          | Type                                                                                                                                                                                                                               |
| :------------------------------------------------------------ | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                             | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `canImplementInterfaceForAddress`                             | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                            |
| `canImplementInterfaceForAddress(bytes32,address)`            | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                            |
| `onTokenTransfer`                                             | (`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                      |
| `onTokenTransfer(address,uint256,bytes)`                      | (`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                      |
| `owner`                                                       | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `owner()`                                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `recoverTokens`                                               | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                   |
| `recoverTokens()`                                             | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                   |
| `renounceOwnership`                                           | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                   |
| `renounceOwnership()`                                         | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                   |
| `tokensReceived`                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `transferOwnership`                                           | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                             |
| `transferOwnership(address)`                                  | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                             |
| `wxHOPR`                                                      | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `wxHOPR()`                                                    | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `xHOPR`                                                       | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `xHOPR()`                                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `xHoprAmount`                                                 | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `xHoprAmount()`                                               | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |

Overrides: Contract.estimateGas

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:469

---

### filters

• **filters**: _object_

#### Type declaration

| Name                   | Type                                                                                                                                                                                                        |
| :--------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `OwnershipTransferred` | (`previousOwner`: _string_, `newOwner`: _string_) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `newOwner`: _string_ ; `previousOwner`: _string_ }\> |
| `Unwrapped`            | (`account`: _string_, `amount`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *BigNumber*], { `account`: _string_ ; `amount`: _BigNumber_ }\>             |
| `Wrapped`              | (`account`: _string_, `amount`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *BigNumber*], { `account`: _string_ ; `amount`: _BigNumber_ }\>             |

Overrides: Contract.filters

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:443

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                                          | Type                                                                                                                                                                                                                                         |
| :------------------------------------------------------------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                             | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `canImplementInterfaceForAddress`                             | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                     |
| `canImplementInterfaceForAddress(bytes32,address)`            | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                     |
| `onTokenTransfer`                                             | (`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                      |
| `onTokenTransfer(address,uint256,bytes)`                      | (`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                      |
| `owner`                                                       | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `owner()`                                                     | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `recoverTokens`                                               | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                   |
| `recoverTokens()`                                             | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                   |
| `renounceOwnership`                                           | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                   |
| `renounceOwnership()`                                         | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                   |
| `tokensReceived`                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\> |
| `transferOwnership`                                           | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                             |
| `transferOwnership(address)`                                  | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                             |
| `wxHOPR`                                                      | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `wxHOPR()`                                                    | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `xHOPR`                                                       | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `xHOPR()`                                                     | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `xHoprAmount`                                                 | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                   |
| `xHoprAmount()`                                               | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                   |

Overrides: Contract.functions

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:163

---

### interface

• **interface**: [_HoprWrapperInterface_](../interfaces/contracts_hoprwrapper.hoprwrapperinterface.md)

Overrides: Contract.interface

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:161

---

### populateTransaction

• **populateTransaction**: _object_

#### Type declaration

| Name                                                          | Type                                                                                                                                                                                                                                          |
| :------------------------------------------------------------ | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                             | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `canImplementInterfaceForAddress`                             | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                            |
| `canImplementInterfaceForAddress(bytes32,address)`            | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                            |
| `onTokenTransfer`                                             | (`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                      |
| `onTokenTransfer(address,uint256,bytes)`                      | (`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                      |
| `owner`                                                       | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `owner()`                                                     | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `recoverTokens`                                               | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                   |
| `recoverTokens()`                                             | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                   |
| `renounceOwnership`                                           | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                   |
| `renounceOwnership()`                                         | (`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                   |
| `tokensReceived`                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `transferOwnership`                                           | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                             |
| `transferOwnership(address)`                                  | (`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                             |
| `wxHOPR`                                                      | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `wxHOPR()`                                                    | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `xHOPR`                                                       | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `xHOPR()`                                                     | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `xHoprAmount`                                                 | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `xHoprAmount()`                                               | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |

Overrides: Contract.populateTransaction

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:567

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

### TOKENS_RECIPIENT_INTERFACE_HASH

▸ **TOKENS_RECIPIENT_INTERFACE_HASH**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:261

---

### TOKENS_RECIPIENT_INTERFACE_HASH()

▸ **TOKENS_RECIPIENT_INTERFACE_HASH()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:261

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

### attach

▸ **attach**(`addressOrName`: _string_): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Parameters

| Name            | Type     |
| :-------------- | :------- |
| `addressOrName` | _string_ |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.attach

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:122

---

### canImplementInterfaceForAddress

▸ **canImplementInterfaceForAddress**(`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name            | Type          |
| :-------------- | :------------ |
| `interfaceHash` | BytesLike     |
| `account`       | _string_      |
| `overrides?`    | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:267

---

### canImplementInterfaceForAddress(bytes32,address)

▸ **canImplementInterfaceForAddress(bytes32,address)**(`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name            | Type          |
| :-------------- | :------------ |
| `interfaceHash` | BytesLike     |
| `account`       | _string_      |
| `overrides?`    | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:271

---

### connect

▸ **connect**(`signerOrProvider`: _string_ \| _Provider_ \| _Signer_): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Parameters

| Name               | Type                               |
| :----------------- | :--------------------------------- |
| `signerOrProvider` | _string_ \| _Provider_ \| _Signer_ |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.connect

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:121

---

### deployed

▸ **deployed**(): _Promise_<[_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)\>

**Returns:** _Promise_<[_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)\>

Overrides: Contract.deployed

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:123

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

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:125

▸ **listeners**(`eventName?`: _string_): Listener[]

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:148

---

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

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

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:128

▸ **off**(`eventName`: _string_, `listener`: Listener): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:149

---

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

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

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:132

▸ **on**(`eventName`: _string_, `listener`: Listener): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:150

---

### onTokenTransfer

▸ **onTokenTransfer**(`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `from`       | _string_                                                |
| `amount`     | BigNumberish                                            |
| `data`       | BytesLike                                               |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:279

---

### onTokenTransfer(address,uint256,bytes)

▸ **onTokenTransfer(address,uint256,bytes)**(`from`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `from`       | _string_                                                |
| `amount`     | BigNumberish                                            |
| `data`       | BytesLike                                               |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:284

---

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

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

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:136

▸ **once**(`eventName`: _string_, `listener`: Listener): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:151

---

### owner

▸ **owner**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:293

---

### owner()

▸ **owner()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:293

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

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:155

---

### recoverTokens

▸ **recoverTokens**(`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:297

---

### recoverTokens()

▸ **recoverTokens()**(`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:299

---

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:144

▸ **removeAllListeners**(`eventName?`: _string_): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:153

---

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

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

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:140

▸ **removeListener**(`eventName`: _string_, `listener`: Listener): [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprWrapper_](contracts_hoprwrapper.hoprwrapper.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:152

---

### renounceOwnership

▸ **renounceOwnership**(`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:305

---

### renounceOwnership()

▸ **renounceOwnership()**(`overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:307

---

### tokensReceived

▸ **tokensReceived**(`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `operator`     | _string_                                                |
| `from`         | _string_                                                |
| `to`           | _string_                                                |
| `amount`       | BigNumberish                                            |
| `userData`     | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:313

---

### tokensReceived(address,address,address,uint256,bytes,bytes)

▸ **tokensReceived(address,address,address,uint256,bytes,bytes)**(`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `operator`     | _string_                                                |
| `from`         | _string_                                                |
| `to`           | _string_                                                |
| `amount`       | BigNumberish                                            |
| `userData`     | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:321

---

### transferOwnership

▸ **transferOwnership**(`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `newOwner`   | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:333

---

### transferOwnership(address)

▸ **transferOwnership(address)**(`newOwner`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `newOwner`   | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:336

---

### wxHOPR

▸ **wxHOPR**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:343

---

### wxHOPR()

▸ **wxHOPR()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:343

---

### xHOPR

▸ **xHOPR**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:347

---

### xHOPR()

▸ **xHOPR()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:347

---

### xHoprAmount

▸ **xHoprAmount**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:351

---

### xHoprAmount()

▸ **xHoprAmount()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprWrapper.d.ts:351

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
