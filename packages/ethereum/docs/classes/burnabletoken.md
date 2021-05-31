[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / BurnableToken

# Class: BurnableToken

## Hierarchy

- *Contract*

  ↳ **BurnableToken**

## Table of contents

### Constructors

- [constructor](burnabletoken.md#constructor)

### Properties

- [\_deployedPromise](burnabletoken.md#_deployedpromise)
- [\_runningEvents](burnabletoken.md#_runningevents)
- [\_wrappedEmits](burnabletoken.md#_wrappedemits)
- [address](burnabletoken.md#address)
- [callStatic](burnabletoken.md#callstatic)
- [deployTransaction](burnabletoken.md#deploytransaction)
- [estimateGas](burnabletoken.md#estimategas)
- [filters](burnabletoken.md#filters)
- [functions](burnabletoken.md#functions)
- [interface](burnabletoken.md#interface)
- [populateTransaction](burnabletoken.md#populatetransaction)
- [provider](burnabletoken.md#provider)
- [resolvedAddress](burnabletoken.md#resolvedaddress)
- [signer](burnabletoken.md#signer)

### Methods

- [\_checkRunningEvents](burnabletoken.md#_checkrunningevents)
- [\_deployed](burnabletoken.md#_deployed)
- [\_wrapEvent](burnabletoken.md#_wrapevent)
- [attach](burnabletoken.md#attach)
- [balanceOf](burnabletoken.md#balanceof)
- [balanceOf(address)](burnabletoken.md#balanceof(address))
- [burn](burnabletoken.md#burn)
- [burn(uint256)](burnabletoken.md#burn(uint256))
- [connect](burnabletoken.md#connect)
- [deployed](burnabletoken.md#deployed)
- [emit](burnabletoken.md#emit)
- [fallback](burnabletoken.md#fallback)
- [listenerCount](burnabletoken.md#listenercount)
- [listeners](burnabletoken.md#listeners)
- [off](burnabletoken.md#off)
- [on](burnabletoken.md#on)
- [once](burnabletoken.md#once)
- [queryFilter](burnabletoken.md#queryfilter)
- [removeAllListeners](burnabletoken.md#removealllisteners)
- [removeListener](burnabletoken.md#removelistener)
- [totalSupply](burnabletoken.md#totalsupply)
- [totalSupply()](burnabletoken.md#totalsupply())
- [transfer](burnabletoken.md#transfer)
- [transfer(address,uint256)](burnabletoken.md#transfer(address,uint256))
- [getContractAddress](burnabletoken.md#getcontractaddress)
- [getInterface](burnabletoken.md#getinterface)
- [isIndexed](burnabletoken.md#isindexed)

## Constructors

### constructor

\+ **new BurnableToken**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*BurnableToken*](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*BurnableToken*](burnabletoken.md)

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
| `balanceOf` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOf(address)` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `burn` | (`_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |
| `burn(uint256)` | (`_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/BurnableToken.d.ts:169

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
| `balanceOf` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOf(address)` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `burn` | (`_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `burn(uint256)` | (`_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/BurnableToken.d.ts:220

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Burn` | (`burner`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *BigNumber*], { `burner`: *string* ; `value`: *BigNumber*  }\> |
| `Transfer` | (`from`: *string*, `to`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `from`: *string* ; `to`: *string* ; `value`: *BigNumber*  }\> |

Overrides: Contract.filters

Defined in: packages/ethereum/types/BurnableToken.d.ts:201

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `balanceOf` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `balanceOf(address)` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `burn` | (`_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `burn(uint256)` | (`_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/BurnableToken.d.ts:101

___

### interface

• **interface**: *BurnableTokenInterface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/BurnableToken.d.ts:99

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `balanceOf` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `balanceOf(address)` | (`_owner`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `burn` | (`_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `burn(uint256)` | (`_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `transfer` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transfer(address,uint256)` | (`_to`: *string*, `_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/BurnableToken.d.ts:255

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

▸ **attach**(`addressOrName`: *string*): [*BurnableToken*](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/BurnableToken.d.ts:60

___

### balanceOf

▸ **balanceOf**(`_owner`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/BurnableToken.d.ts:150

___

### balanceOf(address)

▸ **balanceOf(address)**(`_owner`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/BurnableToken.d.ts:150

___

### burn

▸ **burn**(`_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/BurnableToken.d.ts:140

___

### burn(uint256)

▸ **burn(uint256)**(`_value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/ethereum/types/BurnableToken.d.ts:143

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*BurnableToken*](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/BurnableToken.d.ts:59

___

### deployed

▸ **deployed**(): *Promise*<[*BurnableToken*](burnabletoken.md)\>

**Returns:** *Promise*<[*BurnableToken*](burnabletoken.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/BurnableToken.d.ts:61

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

Defined in: packages/ethereum/types/BurnableToken.d.ts:63

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/BurnableToken.d.ts:86

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*BurnableToken*](burnabletoken.md)

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

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/BurnableToken.d.ts:66

▸ **off**(`eventName`: *string*, `listener`: Listener): [*BurnableToken*](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/BurnableToken.d.ts:87

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*BurnableToken*](burnabletoken.md)

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

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/BurnableToken.d.ts:70

▸ **on**(`eventName`: *string*, `listener`: Listener): [*BurnableToken*](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/BurnableToken.d.ts:88

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*BurnableToken*](burnabletoken.md)

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

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/BurnableToken.d.ts:74

▸ **once**(`eventName`: *string*, `listener`: Listener): [*BurnableToken*](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/BurnableToken.d.ts:89

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

Defined in: packages/ethereum/types/BurnableToken.d.ts:93

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*BurnableToken*](burnabletoken.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/BurnableToken.d.ts:82

▸ **removeAllListeners**(`eventName?`: *string*): [*BurnableToken*](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/BurnableToken.d.ts:91

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*BurnableToken*](burnabletoken.md)

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

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/BurnableToken.d.ts:78

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*BurnableToken*](burnabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*BurnableToken*](burnabletoken.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/BurnableToken.d.ts:90

___

### totalSupply

▸ **totalSupply**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/BurnableToken.d.ts:136

___

### totalSupply()

▸ **totalSupply()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/ethereum/types/BurnableToken.d.ts:136

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

Defined in: packages/ethereum/types/BurnableToken.d.ts:157

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

Defined in: packages/ethereum/types/BurnableToken.d.ts:161

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
