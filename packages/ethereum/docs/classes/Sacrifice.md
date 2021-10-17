[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / Sacrifice

# Class: Sacrifice

## Hierarchy

- `BaseContract`

  ↳ **`Sacrifice`**

## Table of contents

### Constructors

- [constructor](Sacrifice.md#constructor)

### Properties

- [\_deployedPromise](Sacrifice.md#_deployedpromise)
- [\_runningEvents](Sacrifice.md#_runningevents)
- [\_wrappedEmits](Sacrifice.md#_wrappedemits)
- [address](Sacrifice.md#address)
- [callStatic](Sacrifice.md#callstatic)
- [deployTransaction](Sacrifice.md#deploytransaction)
- [estimateGas](Sacrifice.md#estimategas)
- [filters](Sacrifice.md#filters)
- [functions](Sacrifice.md#functions)
- [interface](Sacrifice.md#interface)
- [populateTransaction](Sacrifice.md#populatetransaction)
- [provider](Sacrifice.md#provider)
- [resolvedAddress](Sacrifice.md#resolvedaddress)
- [signer](Sacrifice.md#signer)

### Methods

- [\_checkRunningEvents](Sacrifice.md#_checkrunningevents)
- [\_deployed](Sacrifice.md#_deployed)
- [\_wrapEvent](Sacrifice.md#_wrapevent)
- [attach](Sacrifice.md#attach)
- [connect](Sacrifice.md#connect)
- [deployed](Sacrifice.md#deployed)
- [emit](Sacrifice.md#emit)
- [fallback](Sacrifice.md#fallback)
- [listenerCount](Sacrifice.md#listenercount)
- [listeners](Sacrifice.md#listeners)
- [off](Sacrifice.md#off)
- [on](Sacrifice.md#on)
- [once](Sacrifice.md#once)
- [queryFilter](Sacrifice.md#queryfilter)
- [removeAllListeners](Sacrifice.md#removealllisteners)
- [removeListener](Sacrifice.md#removelistener)
- [getContractAddress](Sacrifice.md#getcontractaddress)
- [getInterface](Sacrifice.md#getinterface)
- [isIndexed](Sacrifice.md#isindexed)

## Constructors

### constructor

• **new Sacrifice**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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

packages/ethereum/types/Sacrifice.d.ts:71

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

packages/ethereum/types/Sacrifice.d.ts:75

___

### filters

• **filters**: `Object`

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:73

___

### functions

• **functions**: `Object`

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:69

___

### interface

• **interface**: `SacrificeInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:67

___

### populateTransaction

• **populateTransaction**: `Object`

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:77

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

▸ **attach**(`addressOrName`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:28

___

### connect

▸ **connect**(`signerOrProvider`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:27

___

### deployed

▸ **deployed**(): `Promise`<[`Sacrifice`](Sacrifice.md)\>

#### Returns

`Promise`<[`Sacrifice`](Sacrifice.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:29

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

packages/ethereum/types/Sacrifice.d.ts:31

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

packages/ethereum/types/Sacrifice.d.ts:54

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`Sacrifice`](Sacrifice.md)

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

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:34

▸ **off**(`eventName`, `listener`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:55

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`Sacrifice`](Sacrifice.md)

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

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:38

▸ **on**(`eventName`, `listener`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:56

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`Sacrifice`](Sacrifice.md)

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

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:42

▸ **once**(`eventName`, `listener`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:57

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

packages/ethereum/types/Sacrifice.d.ts:61

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`Sacrifice`](Sacrifice.md)

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

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:50

▸ **removeAllListeners**(`eventName?`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:59

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`Sacrifice`](Sacrifice.md)

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

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:46

▸ **removeListener**(`eventName`, `listener`): [`Sacrifice`](Sacrifice.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`Sacrifice`](Sacrifice.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/Sacrifice.d.ts:58

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
