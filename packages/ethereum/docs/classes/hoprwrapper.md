[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprWrapper

# Class: HoprWrapper

## Hierarchy

- *Contract*

  ↳ **HoprWrapper**

## Table of contents

### Constructors

- [constructor](hoprwrapper.md#constructor)

### Properties

- [\_deployedPromise](hoprwrapper.md#_deployedpromise)
- [\_runningEvents](hoprwrapper.md#_runningevents)
- [\_wrappedEmits](hoprwrapper.md#_wrappedemits)
- [address](hoprwrapper.md#address)
- [callStatic](hoprwrapper.md#callstatic)
- [deployTransaction](hoprwrapper.md#deploytransaction)
- [estimateGas](hoprwrapper.md#estimategas)
- [filters](hoprwrapper.md#filters)
- [functions](hoprwrapper.md#functions)
- [interface](hoprwrapper.md#interface)
- [populateTransaction](hoprwrapper.md#populatetransaction)
- [provider](hoprwrapper.md#provider)
- [resolvedAddress](hoprwrapper.md#resolvedaddress)
- [signer](hoprwrapper.md#signer)

### Methods

- [TOKENS\_RECIPIENT\_INTERFACE\_HASH](hoprwrapper.md#tokens_recipient_interface_hash)
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH()](hoprwrapper.md#tokens_recipient_interface_hash())
- [\_checkRunningEvents](hoprwrapper.md#_checkrunningevents)
- [\_deployed](hoprwrapper.md#_deployed)
- [\_wrapEvent](hoprwrapper.md#_wrapevent)
- [attach](hoprwrapper.md#attach)
- [canImplementInterfaceForAddress](hoprwrapper.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](hoprwrapper.md#canimplementinterfaceforaddress(bytes32,address))
- [connect](hoprwrapper.md#connect)
- [deployed](hoprwrapper.md#deployed)
- [emit](hoprwrapper.md#emit)
- [fallback](hoprwrapper.md#fallback)
- [listenerCount](hoprwrapper.md#listenercount)
- [listeners](hoprwrapper.md#listeners)
- [off](hoprwrapper.md#off)
- [on](hoprwrapper.md#on)
- [onTokenTransfer](hoprwrapper.md#ontokentransfer)
- [onTokenTransfer(address,uint256,bytes)](hoprwrapper.md#ontokentransfer(address,uint256,bytes))
- [once](hoprwrapper.md#once)
- [owner](hoprwrapper.md#owner)
- [owner()](hoprwrapper.md#owner())
- [queryFilter](hoprwrapper.md#queryfilter)
- [recoverTokens](hoprwrapper.md#recovertokens)
- [recoverTokens()](hoprwrapper.md#recovertokens())
- [removeAllListeners](hoprwrapper.md#removealllisteners)
- [removeListener](hoprwrapper.md#removelistener)
- [renounceOwnership](hoprwrapper.md#renounceownership)
- [renounceOwnership()](hoprwrapper.md#renounceownership())
- [tokensReceived](hoprwrapper.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](hoprwrapper.md#tokensreceived(address,address,address,uint256,bytes,bytes))
- [transferOwnership](hoprwrapper.md#transferownership)
- [transferOwnership(address)](hoprwrapper.md#transferownership(address))
- [wxHOPR](hoprwrapper.md#wxhopr)
- [wxHOPR()](hoprwrapper.md#wxhopr())
- [xHOPR](hoprwrapper.md#xhopr)
- [xHOPR()](hoprwrapper.md#xhopr())
- [xHoprAmount](hoprwrapper.md#xhopramount)
- [xHoprAmount()](hoprwrapper.md#xhopramount())
- [getContractAddress](hoprwrapper.md#getcontractaddress)
- [getInterface](hoprwrapper.md#getinterface)
- [isIndexed](hoprwrapper.md#isindexed)

## Constructors

### constructor

\+ **new HoprWrapper**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*HoprWrapper*](hoprwrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

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
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<string\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<string\> |
| `onTokenTransfer` | (`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `onTokenTransfer(address,uint256,bytes)` | (`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `recoverTokens` | (`overrides?`: CallOverrides) => *Promise*<void\> |
| `recoverTokens()` | (`overrides?`: CallOverrides) => *Promise*<void\> |
| `renounceOwnership` | (`overrides?`: CallOverrides) => *Promise*<void\> |
| `renounceOwnership()` | (`overrides?`: CallOverrides) => *Promise*<void\> |
| `tokensReceived` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `transferOwnership` | (`newOwner`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `transferOwnership(address)` | (`newOwner`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `wxHOPR` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `wxHOPR()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `xHOPR` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `xHOPR()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `xHoprAmount` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `xHoprAmount()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/HoprWrapper.d.ts:355

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
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `onTokenTransfer` | (`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `onTokenTransfer(address,uint256,bytes)` | (`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `recoverTokens` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `recoverTokens()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `tokensReceived` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferOwnership` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferOwnership(address)` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `wxHOPR` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `wxHOPR()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `xHOPR` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `xHOPR()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `xHoprAmount` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `xHoprAmount()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/HoprWrapper.d.ts:469

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `OwnershipTransferred` | (`previousOwner`: *string*, `newOwner`: *string*) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*], { `newOwner`: *string* ; `previousOwner`: *string*  }\> |
| `Unwrapped` | (`account`: *string*, `amount`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *BigNumber*], { `account`: *string* ; `amount`: *BigNumber*  }\> |
| `Wrapped` | (`account`: *string*, `amount`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *BigNumber*], { `account`: *string* ; `amount`: *BigNumber*  }\> |

Overrides: Contract.filters

Defined in: packages/ethereum/types/HoprWrapper.d.ts:443

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `onTokenTransfer` | (`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `onTokenTransfer(address,uint256,bytes)` | (`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `recoverTokens` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `recoverTokens()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `tokensReceived` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferOwnership` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferOwnership(address)` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `wxHOPR` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `wxHOPR()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `xHOPR` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `xHOPR()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `xHoprAmount` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `xHoprAmount()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/HoprWrapper.d.ts:163

___

### interface

• **interface**: *HoprWrapperInterface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/HoprWrapper.d.ts:161

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `onTokenTransfer` | (`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `onTokenTransfer(address,uint256,bytes)` | (`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `owner` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `owner()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `recoverTokens` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `recoverTokens()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `renounceOwnership` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `renounceOwnership()` | (`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `tokensReceived` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferOwnership` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferOwnership(address)` | (`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `wxHOPR` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `wxHOPR()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `xHOPR` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `xHOPR()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `xHoprAmount` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `xHoprAmount()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/HoprWrapper.d.ts:567

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

### TOKENS\_RECIPIENT\_INTERFACE\_HASH

▸ **TOKENS_RECIPIENT_INTERFACE_HASH**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:261

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH()

▸ **TOKENS_RECIPIENT_INTERFACE_HASH()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:261

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

### attach

▸ **attach**(`addressOrName`: *string*): [*HoprWrapper*](hoprwrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/HoprWrapper.d.ts:122

___

### canImplementInterfaceForAddress

▸ **canImplementInterfaceForAddress**(`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | BytesLike |
| `account` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:267

___

### canImplementInterfaceForAddress(bytes32,address)

▸ **canImplementInterfaceForAddress(bytes32,address)**(`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | BytesLike |
| `account` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:271

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*HoprWrapper*](hoprwrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/HoprWrapper.d.ts:121

___

### deployed

▸ **deployed**(): *Promise*<[*HoprWrapper*](hoprwrapper.md)\>

**Returns:** *Promise*<[*HoprWrapper*](hoprwrapper.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/HoprWrapper.d.ts:123

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

Defined in: packages/ethereum/types/HoprWrapper.d.ts:125

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/HoprWrapper.d.ts:148

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprWrapper*](hoprwrapper.md)

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

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/HoprWrapper.d.ts:128

▸ **off**(`eventName`: *string*, `listener`: Listener): [*HoprWrapper*](hoprwrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/HoprWrapper.d.ts:149

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprWrapper*](hoprwrapper.md)

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

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/HoprWrapper.d.ts:132

▸ **on**(`eventName`: *string*, `listener`: Listener): [*HoprWrapper*](hoprwrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/HoprWrapper.d.ts:150

___

### onTokenTransfer

▸ **onTokenTransfer**(`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `from` | *string* |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:279

___

### onTokenTransfer(address,uint256,bytes)

▸ **onTokenTransfer(address,uint256,bytes)**(`from`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `from` | *string* |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:284

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprWrapper*](hoprwrapper.md)

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

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/HoprWrapper.d.ts:136

▸ **once**(`eventName`: *string*, `listener`: Listener): [*HoprWrapper*](hoprwrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/HoprWrapper.d.ts:151

___

### owner

▸ **owner**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:293

___

### owner()

▸ **owner()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:293

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

Defined in: packages/ethereum/types/HoprWrapper.d.ts:155

___

### recoverTokens

▸ **recoverTokens**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:297

___

### recoverTokens()

▸ **recoverTokens()**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:299

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*HoprWrapper*](hoprwrapper.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/HoprWrapper.d.ts:144

▸ **removeAllListeners**(`eventName?`: *string*): [*HoprWrapper*](hoprwrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/HoprWrapper.d.ts:153

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprWrapper*](hoprwrapper.md)

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

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/HoprWrapper.d.ts:140

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*HoprWrapper*](hoprwrapper.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprWrapper*](hoprwrapper.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/HoprWrapper.d.ts:152

___

### renounceOwnership

▸ **renounceOwnership**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:305

___

### renounceOwnership()

▸ **renounceOwnership()**(`overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:307

___

### tokensReceived

▸ **tokensReceived**(`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `from` | *string* |
| `to` | *string* |
| `amount` | BigNumberish |
| `userData` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:313

___

### tokensReceived(address,address,address,uint256,bytes,bytes)

▸ **tokensReceived(address,address,address,uint256,bytes,bytes)**(`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `from` | *string* |
| `to` | *string* |
| `amount` | BigNumberish |
| `userData` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:321

___

### transferOwnership

▸ **transferOwnership**(`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newOwner` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:333

___

### transferOwnership(address)

▸ **transferOwnership(address)**(`newOwner`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newOwner` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:336

___

### wxHOPR

▸ **wxHOPR**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:343

___

### wxHOPR()

▸ **wxHOPR()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:343

___

### xHOPR

▸ **xHOPR**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:347

___

### xHOPR()

▸ **xHOPR()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:347

___

### xHoprAmount

▸ **xHoprAmount**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:351

___

### xHoprAmount()

▸ **xHoprAmount()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/HoprWrapper.d.ts:351

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
