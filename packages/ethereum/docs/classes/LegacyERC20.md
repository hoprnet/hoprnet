[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / LegacyERC20

# Class: LegacyERC20

## Hierarchy

- `BaseContract`

  ↳ **`LegacyERC20`**

## Table of contents

### Constructors

- [constructor](LegacyERC20.md#constructor)

### Properties

- [\_deployedPromise](LegacyERC20.md#_deployedpromise)
- [\_runningEvents](LegacyERC20.md#_runningevents)
- [\_wrappedEmits](LegacyERC20.md#_wrappedemits)
- [address](LegacyERC20.md#address)
- [callStatic](LegacyERC20.md#callstatic)
- [deployTransaction](LegacyERC20.md#deploytransaction)
- [estimateGas](LegacyERC20.md#estimategas)
- [filters](LegacyERC20.md#filters)
- [functions](LegacyERC20.md#functions)
- [interface](LegacyERC20.md#interface)
- [populateTransaction](LegacyERC20.md#populatetransaction)
- [provider](LegacyERC20.md#provider)
- [resolvedAddress](LegacyERC20.md#resolvedaddress)
- [signer](LegacyERC20.md#signer)

### Methods

- [\_checkRunningEvents](LegacyERC20.md#_checkrunningevents)
- [\_deployed](LegacyERC20.md#_deployed)
- [\_wrapEvent](LegacyERC20.md#_wrapevent)
- [attach](LegacyERC20.md#attach)
- [connect](LegacyERC20.md#connect)
- [deployed](LegacyERC20.md#deployed)
- [emit](LegacyERC20.md#emit)
- [fallback](LegacyERC20.md#fallback)
- [listenerCount](LegacyERC20.md#listenercount)
- [listeners](LegacyERC20.md#listeners)
- [off](LegacyERC20.md#off)
- [on](LegacyERC20.md#on)
- [once](LegacyERC20.md#once)
- [queryFilter](LegacyERC20.md#queryfilter)
- [removeAllListeners](LegacyERC20.md#removealllisteners)
- [removeListener](LegacyERC20.md#removelistener)
- [transfer](LegacyERC20.md#transfer)
- [transferFrom](LegacyERC20.md#transferfrom)
- [getContractAddress](LegacyERC20.md#getcontractaddress)
- [getInterface](LegacyERC20.md#getinterface)
- [isIndexed](LegacyERC20.md#isindexed)

## Constructors

### constructor

• **new LegacyERC20**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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

#### Type declaration

| Name | Type |
| :------ | :------ |
| `transfer` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `transferFrom` | (`_owner`: `string`, `_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:117

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

#### Type declaration

| Name | Type |
| :------ | :------ |
| `transfer` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom` | (`_owner`: `string`, `_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:134

___

### filters

• **filters**: `Object`

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:132

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `transfer` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom` | (`_owner`: `string`, `_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:89

___

### interface

• **interface**: `LegacyERC20Interface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:87

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `transfer` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom` | (`_owner`: `string`, `_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:149

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

▸ **attach**(`addressOrName`): [`LegacyERC20`](LegacyERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:48

___

### connect

▸ **connect**(`signerOrProvider`): [`LegacyERC20`](LegacyERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:47

___

### deployed

▸ **deployed**(): `Promise`<[`LegacyERC20`](LegacyERC20.md)\>

#### Returns

`Promise`<[`LegacyERC20`](LegacyERC20.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:49

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

packages/ethereum/types/LegacyERC20.d.ts:51

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

packages/ethereum/types/LegacyERC20.d.ts:74

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`LegacyERC20`](LegacyERC20.md)

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

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:54

▸ **off**(`eventName`, `listener`): [`LegacyERC20`](LegacyERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:75

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`LegacyERC20`](LegacyERC20.md)

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

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:58

▸ **on**(`eventName`, `listener`): [`LegacyERC20`](LegacyERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:76

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`LegacyERC20`](LegacyERC20.md)

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

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:62

▸ **once**(`eventName`, `listener`): [`LegacyERC20`](LegacyERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:77

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

packages/ethereum/types/LegacyERC20.d.ts:81

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`LegacyERC20`](LegacyERC20.md)

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

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:70

▸ **removeAllListeners**(`eventName?`): [`LegacyERC20`](LegacyERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:79

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`LegacyERC20`](LegacyERC20.md)

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

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:66

▸ **removeListener**(`eventName`, `listener`): [`LegacyERC20`](LegacyERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`LegacyERC20`](LegacyERC20.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:78

___

### transfer

▸ **transfer**(`_spender`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:111

___

### transferFrom

▸ **transferFrom**(`_owner`, `_spender`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `_spender` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/LegacyERC20.d.ts:104

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
