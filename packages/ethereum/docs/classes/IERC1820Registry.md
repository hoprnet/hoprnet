[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC1820Registry

# Class: IERC1820Registry

## Hierarchy

- `BaseContract`

  ↳ **`IERC1820Registry`**

## Table of contents

### Constructors

- [constructor](IERC1820Registry.md#constructor)

### Properties

- [\_deployedPromise](IERC1820Registry.md#_deployedpromise)
- [\_runningEvents](IERC1820Registry.md#_runningevents)
- [\_wrappedEmits](IERC1820Registry.md#_wrappedemits)
- [address](IERC1820Registry.md#address)
- [callStatic](IERC1820Registry.md#callstatic)
- [deployTransaction](IERC1820Registry.md#deploytransaction)
- [estimateGas](IERC1820Registry.md#estimategas)
- [filters](IERC1820Registry.md#filters)
- [functions](IERC1820Registry.md#functions)
- [interface](IERC1820Registry.md#interface)
- [populateTransaction](IERC1820Registry.md#populatetransaction)
- [provider](IERC1820Registry.md#provider)
- [resolvedAddress](IERC1820Registry.md#resolvedaddress)
- [signer](IERC1820Registry.md#signer)

### Methods

- [\_checkRunningEvents](IERC1820Registry.md#_checkrunningevents)
- [\_deployed](IERC1820Registry.md#_deployed)
- [\_wrapEvent](IERC1820Registry.md#_wrapevent)
- [attach](IERC1820Registry.md#attach)
- [connect](IERC1820Registry.md#connect)
- [deployed](IERC1820Registry.md#deployed)
- [emit](IERC1820Registry.md#emit)
- [fallback](IERC1820Registry.md#fallback)
- [getInterfaceImplementer](IERC1820Registry.md#getinterfaceimplementer)
- [getManager](IERC1820Registry.md#getmanager)
- [implementsERC165Interface](IERC1820Registry.md#implementserc165interface)
- [implementsERC165InterfaceNoCache](IERC1820Registry.md#implementserc165interfacenocache)
- [interfaceHash](IERC1820Registry.md#interfacehash)
- [listenerCount](IERC1820Registry.md#listenercount)
- [listeners](IERC1820Registry.md#listeners)
- [off](IERC1820Registry.md#off)
- [on](IERC1820Registry.md#on)
- [once](IERC1820Registry.md#once)
- [queryFilter](IERC1820Registry.md#queryfilter)
- [removeAllListeners](IERC1820Registry.md#removealllisteners)
- [removeListener](IERC1820Registry.md#removelistener)
- [setInterfaceImplementer](IERC1820Registry.md#setinterfaceimplementer)
- [setManager](IERC1820Registry.md#setmanager)
- [updateERC165Cache](IERC1820Registry.md#updateerc165cache)
- [getContractAddress](IERC1820Registry.md#getcontractaddress)
- [getInterface](IERC1820Registry.md#getinterface)
- [isIndexed](IERC1820Registry.md#isindexed)

## Constructors

### constructor

• **new IERC1820Registry**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `getInterfaceImplementer` | (`account`: `string`, `_interfaceHash`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getManager` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `implementsERC165Interface` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `implementsERC165InterfaceNoCache` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `interfaceHash` | (`interfaceName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `setInterfaceImplementer` | (`account`: `string`, `_interfaceHash`: `BytesLike`, `implementer`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `setManager` | (`account`: `string`, `newManager`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `updateERC165Cache` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:233

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
| `getInterfaceImplementer` | (`account`: `string`, `_interfaceHash`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getManager` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `implementsERC165Interface` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `implementsERC165InterfaceNoCache` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `interfaceHash` | (`interfaceName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `setInterfaceImplementer` | (`account`: `string`, `_interfaceHash`: `BytesLike`, `implementer`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `setManager` | (`account`: `string`, `newManager`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `updateERC165Cache` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:298

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `InterfaceImplementerSet` | (`account?`: `string`, `interfaceHash?`: `BytesLike`, `implementer?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], `Object`\> |
| `ManagerChanged` | (`account?`: `string`, `newManager?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], `Object`\> |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:279

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `getInterfaceImplementer` | (`account`: `string`, `_interfaceHash`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getManager` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `implementsERC165Interface` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `implementsERC165InterfaceNoCache` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `interfaceHash` | (`interfaceName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `setInterfaceImplementer` | (`account`: `string`, `_interfaceHash`: `BytesLike`, `implementer`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `setManager` | (`account`: `string`, `newManager`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `updateERC165Cache` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:143

___

### interface

• **interface**: `IERC1820RegistryInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:141

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `getInterfaceImplementer` | (`account`: `string`, `_interfaceHash`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getManager` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `implementsERC165Interface` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `implementsERC165InterfaceNoCache` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `interfaceHash` | (`interfaceName`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `setInterfaceImplementer` | (`account`: `string`, `_interfaceHash`: `BytesLike`, `implementer`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `setManager` | (`account`: `string`, `newManager`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `updateERC165Cache` | (`account`: `string`, `interfaceId`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:344

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

▸ **attach**(`addressOrName`): [`IERC1820Registry`](IERC1820Registry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:102

___

### connect

▸ **connect**(`signerOrProvider`): [`IERC1820Registry`](IERC1820Registry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:101

___

### deployed

▸ **deployed**(): `Promise`<[`IERC1820Registry`](IERC1820Registry.md)\>

#### Returns

`Promise`<[`IERC1820Registry`](IERC1820Registry.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:103

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

### getInterfaceImplementer

▸ **getInterfaceImplementer**(`account`, `_interfaceHash`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `_interfaceHash` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:189

___

### getManager

▸ **getManager**(`account`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:195

___

### implementsERC165Interface

▸ **implementsERC165Interface**(`account`, `interfaceId`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `interfaceId` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:197

___

### implementsERC165InterfaceNoCache

▸ **implementsERC165InterfaceNoCache**(`account`, `interfaceId`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `interfaceId` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:203

___

### interfaceHash

▸ **interfaceHash**(`interfaceName`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceName` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:209

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

▸ **listeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter?`): [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\>[]

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

[`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\>[]

#### Overrides

BaseContract.listeners

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:105

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

packages/ethereum/types/IERC1820Registry.d.ts:128

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC1820Registry`](IERC1820Registry.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:108

▸ **off**(`eventName`, `listener`): [`IERC1820Registry`](IERC1820Registry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:129

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC1820Registry`](IERC1820Registry.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:112

▸ **on**(`eventName`, `listener`): [`IERC1820Registry`](IERC1820Registry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:130

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC1820Registry`](IERC1820Registry.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:116

▸ **once**(`eventName`, `listener`): [`IERC1820Registry`](IERC1820Registry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:131

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

packages/ethereum/types/IERC1820Registry.d.ts:135

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`IERC1820Registry`](IERC1820Registry.md)

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

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:124

▸ **removeAllListeners**(`eventName?`): [`IERC1820Registry`](IERC1820Registry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:133

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC1820Registry`](IERC1820Registry.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:120

▸ **removeListener**(`eventName`, `listener`): [`IERC1820Registry`](IERC1820Registry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC1820Registry`](IERC1820Registry.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:132

___

### setInterfaceImplementer

▸ **setInterfaceImplementer**(`account`, `_interfaceHash`, `implementer`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `_interfaceHash` | `BytesLike` |
| `implementer` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:214

___

### setManager

▸ **setManager**(`account`, `newManager`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `newManager` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:221

___

### updateERC165Cache

▸ **updateERC165Cache**(`account`, `interfaceId`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `interfaceId` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/IERC1820Registry.d.ts:227

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
