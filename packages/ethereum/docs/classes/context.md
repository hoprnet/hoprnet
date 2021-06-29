[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / Context

# Class: Context

## Hierarchy

- `Contract`

  ↳ **`Context`**

## Table of contents

### Constructors

- [constructor](context.md#constructor)

### Properties

- [\_deployedPromise](context.md#_deployedpromise)
- [\_runningEvents](context.md#_runningevents)
- [\_wrappedEmits](context.md#_wrappedemits)
- [address](context.md#address)
- [callStatic](context.md#callstatic)
- [deployTransaction](context.md#deploytransaction)
- [estimateGas](context.md#estimategas)
- [filters](context.md#filters)
- [functions](context.md#functions)
- [interface](context.md#interface)
- [populateTransaction](context.md#populatetransaction)
- [provider](context.md#provider)
- [resolvedAddress](context.md#resolvedaddress)
- [signer](context.md#signer)

### Methods

- [\_checkRunningEvents](context.md#_checkrunningevents)
- [\_deployed](context.md#_deployed)
- [\_wrapEvent](context.md#_wrapevent)
- [attach](context.md#attach)
- [connect](context.md#connect)
- [deployed](context.md#deployed)
- [emit](context.md#emit)
- [fallback](context.md#fallback)
- [listenerCount](context.md#listenercount)
- [listeners](context.md#listeners)
- [off](context.md#off)
- [on](context.md#on)
- [once](context.md#once)
- [queryFilter](context.md#queryfilter)
- [removeAllListeners](context.md#removealllisteners)
- [removeListener](context.md#removelistener)
- [getContractAddress](context.md#getcontractaddress)
- [getInterface](context.md#getinterface)
- [isIndexed](context.md#isindexed)

## Constructors

### constructor

• **new Context**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/Context.d.ts:71

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

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/Context.d.ts:75

___

### filters

• **filters**: `Object`

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/Context.d.ts:73

___

### functions

• **functions**: `Object`

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/Context.d.ts:69

___

### interface

• **interface**: `ContextInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/Context.d.ts:67

___

### populateTransaction

• **populateTransaction**: `Object`

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/Context.d.ts:77

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

### attach

▸ **attach**(`addressOrName`): [`Context`](context.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`Context`](context.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/Context.d.ts:28

___

### connect

▸ **connect**(`signerOrProvider`): [`Context`](context.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`Context`](context.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/Context.d.ts:27

___

### deployed

▸ **deployed**(): `Promise`<[`Context`](context.md)\>

#### Returns

`Promise`<[`Context`](context.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/Context.d.ts:29

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

packages/ethereum/types/Context.d.ts:31

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

packages/ethereum/types/Context.d.ts:54

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`Context`](context.md)

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

[`Context`](context.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/Context.d.ts:34

▸ **off**(`eventName`, `listener`): [`Context`](context.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`Context`](context.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/Context.d.ts:55

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`Context`](context.md)

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

[`Context`](context.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/Context.d.ts:38

▸ **on**(`eventName`, `listener`): [`Context`](context.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`Context`](context.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/Context.d.ts:56

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`Context`](context.md)

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

[`Context`](context.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/Context.d.ts:42

▸ **once**(`eventName`, `listener`): [`Context`](context.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`Context`](context.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/Context.d.ts:57

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

packages/ethereum/types/Context.d.ts:61

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`Context`](context.md)

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

[`Context`](context.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/Context.d.ts:50

▸ **removeAllListeners**(`eventName?`): [`Context`](context.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`Context`](context.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/Context.d.ts:59

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`Context`](context.md)

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

[`Context`](context.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/Context.d.ts:46

▸ **removeListener**(`eventName`, `listener`): [`Context`](context.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`Context`](context.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/Context.d.ts:58

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
