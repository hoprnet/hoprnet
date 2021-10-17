[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ReentrancyGuard

# Class: ReentrancyGuard

## Hierarchy

- `BaseContract`

  ↳ **`ReentrancyGuard`**

## Table of contents

### Constructors

- [constructor](ReentrancyGuard.md#constructor)

### Properties

- [\_deployedPromise](ReentrancyGuard.md#_deployedpromise)
- [\_runningEvents](ReentrancyGuard.md#_runningevents)
- [\_wrappedEmits](ReentrancyGuard.md#_wrappedemits)
- [address](ReentrancyGuard.md#address)
- [callStatic](ReentrancyGuard.md#callstatic)
- [deployTransaction](ReentrancyGuard.md#deploytransaction)
- [estimateGas](ReentrancyGuard.md#estimategas)
- [filters](ReentrancyGuard.md#filters)
- [functions](ReentrancyGuard.md#functions)
- [interface](ReentrancyGuard.md#interface)
- [populateTransaction](ReentrancyGuard.md#populatetransaction)
- [provider](ReentrancyGuard.md#provider)
- [resolvedAddress](ReentrancyGuard.md#resolvedaddress)
- [signer](ReentrancyGuard.md#signer)

### Methods

- [\_checkRunningEvents](ReentrancyGuard.md#_checkrunningevents)
- [\_deployed](ReentrancyGuard.md#_deployed)
- [\_wrapEvent](ReentrancyGuard.md#_wrapevent)
- [attach](ReentrancyGuard.md#attach)
- [connect](ReentrancyGuard.md#connect)
- [deployed](ReentrancyGuard.md#deployed)
- [emit](ReentrancyGuard.md#emit)
- [fallback](ReentrancyGuard.md#fallback)
- [listenerCount](ReentrancyGuard.md#listenercount)
- [listeners](ReentrancyGuard.md#listeners)
- [off](ReentrancyGuard.md#off)
- [on](ReentrancyGuard.md#on)
- [once](ReentrancyGuard.md#once)
- [queryFilter](ReentrancyGuard.md#queryfilter)
- [removeAllListeners](ReentrancyGuard.md#removealllisteners)
- [removeListener](ReentrancyGuard.md#removelistener)
- [getContractAddress](ReentrancyGuard.md#getcontractaddress)
- [getInterface](ReentrancyGuard.md#getinterface)
- [isIndexed](ReentrancyGuard.md#isindexed)

## Constructors

### constructor

• **new ReentrancyGuard**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |
| `contractInterface` | `ContractInterface` |
| `signerOrProvider?` | `Signer` \| `Provider` |

#### Inherited from

BaseContract.constructor

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:103

## Properties

### \_deployedPromise

• **\_deployedPromise**: `Promise`<`Contract`\>

#### Inherited from

BaseContract.\_deployedPromise

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### \_runningEvents

• **\_runningEvents**: `Object`

#### Index signature

▪ [eventTag: `string`]: `RunningEvent`

#### Inherited from

BaseContract.\_runningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:97

___

### \_wrappedEmits

• **\_wrappedEmits**: `Object`

#### Index signature

▪ [eventTag: `string`]: (...`args`: `any`[]) => `void`

#### Inherited from

BaseContract.\_wrappedEmits

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:100

___

### address

• `Readonly` **address**: `string`

#### Inherited from

BaseContract.address

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:75

___

### callStatic

• **callStatic**: `Object`

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:71

___

### deployTransaction

• `Readonly` **deployTransaction**: `TransactionResponse`

#### Inherited from

BaseContract.deployTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:95

___

### estimateGas

• **estimateGas**: `Object`

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:75

___

### filters

• **filters**: `Object`

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:73

___

### functions

• **functions**: `Object`

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:69

___

### interface

• **interface**: `ReentrancyGuardInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:67

___

### populateTransaction

• **populateTransaction**: `Object`

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:77

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

BaseContract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:78

___

### resolvedAddress

• `Readonly` **resolvedAddress**: `Promise`<`string`\>

#### Inherited from

BaseContract.resolvedAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:94

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

BaseContract.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:77

## Methods

### \_checkRunningEvents

▸ **_checkRunningEvents**(`runningEvent`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | `RunningEvent` |

#### Returns

`void`

#### Inherited from

BaseContract.\_checkRunningEvents

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

BaseContract.\_deployed

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

BaseContract.\_wrapEvent

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:118

___

### attach

▸ **attach**(`addressOrName`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:28

___

### connect

▸ **connect**(`signerOrProvider`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:27

___

### deployed

▸ **deployed**(): `Promise`<[`ReentrancyGuard`](ReentrancyGuard.md)\>

#### Returns

`Promise`<[`ReentrancyGuard`](ReentrancyGuard.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:29

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

BaseContract.emit

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

BaseContract.fallback

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:111

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

BaseContract.listenerCount

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:124

___

### listeners

▸ **listeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter?`): `TypedListener`<`EventArgsArray`, `EventArgsObject`\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

`TypedListener`<`EventArgsArray`, `EventArgsObject`\>[]

#### Overrides

BaseContract.listeners

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:31

▸ **listeners**(`eventName?`): `Listener`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

`Listener`[]

#### Overrides

BaseContract.listeners

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:54

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | `TypedListener`<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:34

▸ **off**(`eventName`, `listener`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:55

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | `TypedListener`<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:38

▸ **on**(`eventName`, `listener`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:56

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | `TypedListener`<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:42

▸ **once**(`eventName`, `listener`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:57

___

### queryFilter

▸ **queryFilter**<`EventArgsArray`, `EventArgsObject`\>(`event`, `fromBlockOrBlockhash?`, `toBlock?`): `Promise`<[`TypedEvent`](../interfaces/TypedEvent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `fromBlockOrBlockhash?` | `string` \| `number` |
| `toBlock?` | `string` \| `number` |

#### Returns

`Promise`<[`TypedEvent`](../interfaces/TypedEvent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Overrides

BaseContract.queryFilter

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:61

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:50

▸ **removeAllListeners**(`eventName?`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:59

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | `TypedListener`<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:46

▸ **removeListener**(`eventName`, `listener`): [`ReentrancyGuard`](ReentrancyGuard.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ReentrancyGuard`](ReentrancyGuard.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/ReentrancyGuard.d.ts:58

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

BaseContract.getContractAddress

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

BaseContract.getInterface

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

BaseContract.isIndexed

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:114
