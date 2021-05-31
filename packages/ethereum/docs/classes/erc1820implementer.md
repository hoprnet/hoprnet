[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC1820Implementer

# Class: ERC1820Implementer

## Hierarchy

- *Contract*

  ↳ **ERC1820Implementer**

## Table of contents

### Constructors

- [constructor](erc1820implementer.md#constructor)

### Properties

- [\_deployedPromise](erc1820implementer.md#_deployedpromise)
- [\_runningEvents](erc1820implementer.md#_runningevents)
- [\_wrappedEmits](erc1820implementer.md#_wrappedemits)
- [address](erc1820implementer.md#address)
- [callStatic](erc1820implementer.md#callstatic)
- [deployTransaction](erc1820implementer.md#deploytransaction)
- [estimateGas](erc1820implementer.md#estimategas)
- [filters](erc1820implementer.md#filters)
- [functions](erc1820implementer.md#functions)
- [interface](erc1820implementer.md#interface)
- [populateTransaction](erc1820implementer.md#populatetransaction)
- [provider](erc1820implementer.md#provider)
- [resolvedAddress](erc1820implementer.md#resolvedaddress)
- [signer](erc1820implementer.md#signer)

### Methods

- [\_checkRunningEvents](erc1820implementer.md#_checkrunningevents)
- [\_deployed](erc1820implementer.md#_deployed)
- [\_wrapEvent](erc1820implementer.md#_wrapevent)
- [attach](erc1820implementer.md#attach)
- [canImplementInterfaceForAddress](erc1820implementer.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](erc1820implementer.md#canimplementinterfaceforaddress(bytes32,address))
- [connect](erc1820implementer.md#connect)
- [deployed](erc1820implementer.md#deployed)
- [emit](erc1820implementer.md#emit)
- [fallback](erc1820implementer.md#fallback)
- [listenerCount](erc1820implementer.md#listenercount)
- [listeners](erc1820implementer.md#listeners)
- [off](erc1820implementer.md#off)
- [on](erc1820implementer.md#on)
- [once](erc1820implementer.md#once)
- [queryFilter](erc1820implementer.md#queryfilter)
- [removeAllListeners](erc1820implementer.md#removealllisteners)
- [removeListener](erc1820implementer.md#removelistener)
- [getContractAddress](erc1820implementer.md#getcontractaddress)
- [getInterface](erc1820implementer.md#getinterface)
- [isIndexed](erc1820implementer.md#isindexed)

## Constructors

### constructor

\+ **new ERC1820Implementer**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Signer* \| *Provider*): [*ERC1820Implementer*](erc1820implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Signer* \| *Provider* |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

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
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<string\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<string\> |

Overrides: Contract.callStatic

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:108

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
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:124

___

### filters

• **filters**: *object*

#### Type declaration

Overrides: Contract.filters

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:122

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |

Overrides: Contract.functions

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:82

___

### interface

• **interface**: *ERC1820ImplementerInterface*

Overrides: Contract.interface

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:80

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:138

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

▸ **attach**(`addressOrName`: *string*): [*ERC1820Implementer*](erc1820implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.attach

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:41

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

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:96

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

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:100

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Signer* \| *Provider*): [*ERC1820Implementer*](erc1820implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Signer* \| *Provider* |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.connect

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:40

___

### deployed

▸ **deployed**(): *Promise*<[*ERC1820Implementer*](erc1820implementer.md)\>

**Returns:** *Promise*<[*ERC1820Implementer*](erc1820implementer.md)\>

Overrides: Contract.deployed

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:42

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

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:44

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:67

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ERC1820Implementer*](erc1820implementer.md)

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

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:47

▸ **off**(`eventName`: *string*, `listener`: Listener): [*ERC1820Implementer*](erc1820implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.off

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:68

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ERC1820Implementer*](erc1820implementer.md)

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

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:51

▸ **on**(`eventName`: *string*, `listener`: Listener): [*ERC1820Implementer*](erc1820implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.on

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:69

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ERC1820Implementer*](erc1820implementer.md)

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

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:55

▸ **once**(`eventName`: *string*, `listener`: Listener): [*ERC1820Implementer*](erc1820implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.once

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:70

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

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:74

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*ERC1820Implementer*](erc1820implementer.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:63

▸ **removeAllListeners**(`eventName?`: *string*): [*ERC1820Implementer*](erc1820implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.removeAllListeners

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:72

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*ERC1820Implementer*](erc1820implementer.md)

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

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:59

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*ERC1820Implementer*](erc1820implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*ERC1820Implementer*](erc1820implementer.md)

Overrides: Contract.removeListener

Defined in: packages/ethereum/types/ERC1820Implementer.d.ts:71

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
