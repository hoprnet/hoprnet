[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC1820Implementer

# Class: IERC1820Implementer

## Hierarchy

- `BaseContract`

  ↳ **`IERC1820Implementer`**

## Table of contents

### Constructors

- [constructor](IERC1820Implementer.md#constructor)

### Properties

- [\_deployedPromise](IERC1820Implementer.md#_deployedpromise)
- [\_runningEvents](IERC1820Implementer.md#_runningevents)
- [\_wrappedEmits](IERC1820Implementer.md#_wrappedemits)
- [address](IERC1820Implementer.md#address)
- [callStatic](IERC1820Implementer.md#callstatic)
- [deployTransaction](IERC1820Implementer.md#deploytransaction)
- [estimateGas](IERC1820Implementer.md#estimategas)
- [filters](IERC1820Implementer.md#filters)
- [functions](IERC1820Implementer.md#functions)
- [interface](IERC1820Implementer.md#interface)
- [populateTransaction](IERC1820Implementer.md#populatetransaction)
- [provider](IERC1820Implementer.md#provider)
- [resolvedAddress](IERC1820Implementer.md#resolvedaddress)
- [signer](IERC1820Implementer.md#signer)

### Methods

- [\_checkRunningEvents](IERC1820Implementer.md#_checkrunningevents)
- [\_deployed](IERC1820Implementer.md#_deployed)
- [\_wrapEvent](IERC1820Implementer.md#_wrapevent)
- [attach](IERC1820Implementer.md#attach)
- [canImplementInterfaceForAddress](IERC1820Implementer.md#canimplementinterfaceforaddress)
- [connect](IERC1820Implementer.md#connect)
- [deployed](IERC1820Implementer.md#deployed)
- [emit](IERC1820Implementer.md#emit)
- [fallback](IERC1820Implementer.md#fallback)
- [listenerCount](IERC1820Implementer.md#listenercount)
- [listeners](IERC1820Implementer.md#listeners)
- [off](IERC1820Implementer.md#off)
- [on](IERC1820Implementer.md#on)
- [once](IERC1820Implementer.md#once)
- [queryFilter](IERC1820Implementer.md#queryfilter)
- [removeAllListeners](IERC1820Implementer.md#removealllisteners)
- [removeListener](IERC1820Implementer.md#removelistener)
- [getContractAddress](IERC1820Implementer.md#getcontractaddress)
- [getInterface](IERC1820Implementer.md#getinterface)
- [isIndexed](IERC1820Implementer.md#isindexed)

## Constructors

### constructor

• **new IERC1820Implementer**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |
| `contractInterface` | `ContractInterface` |
| `signerOrProvider?` | `Signer` \| `Provider` |

#### Inherited from

BaseContract.constructor

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:105

## Properties

### \_deployedPromise

• **\_deployedPromise**: `Promise`<`Contract`\>

#### Inherited from

BaseContract.\_deployedPromise

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:98

___

### \_runningEvents

• **\_runningEvents**: `Object`

#### Index signature

▪ [eventTag: `string`]: `RunningEvent`

#### Inherited from

BaseContract.\_runningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:99

___

### \_wrappedEmits

• **\_wrappedEmits**: `Object`

#### Index signature

▪ [eventTag: `string`]: (...`args`: `any`[]) => `void`

#### Inherited from

BaseContract.\_wrappedEmits

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:102

___

### address

• `Readonly` **address**: `string`

#### Inherited from

BaseContract.address

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:77

___

### callStatic

• **callStatic**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:96

___

### deployTransaction

• `Readonly` **deployTransaction**: `TransactionResponse`

#### Inherited from

BaseContract.deployTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:97

___

### estimateGas

• **estimateGas**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:106

___

### filters

• **filters**: `Object`

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:104

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:82

___

### interface

• **interface**: `IERC1820ImplementerInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:80

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:114

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

BaseContract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:80

___

### resolvedAddress

• `Readonly` **resolvedAddress**: `Promise`<`string`\>

#### Inherited from

BaseContract.resolvedAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

BaseContract.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:79

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

node_modules/@ethersproject/contracts/lib/index.d.ts:119

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

node_modules/@ethersproject/contracts/lib/index.d.ts:112

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

node_modules/@ethersproject/contracts/lib/index.d.ts:120

___

### attach

▸ **attach**(`addressOrName`): [`IERC1820Implementer`](IERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:41

___

### canImplementInterfaceForAddress

▸ **canImplementInterfaceForAddress**(`interfaceHash`, `account`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:90

___

### connect

▸ **connect**(`signerOrProvider`): [`IERC1820Implementer`](IERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:40

___

### deployed

▸ **deployed**(): `Promise`<[`IERC1820Implementer`](IERC1820Implementer.md)\>

#### Returns

`Promise`<[`IERC1820Implementer`](IERC1820Implementer.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:42

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

node_modules/@ethersproject/contracts/lib/index.d.ts:125

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

node_modules/@ethersproject/contracts/lib/index.d.ts:113

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

node_modules/@ethersproject/contracts/lib/index.d.ts:126

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

packages/ethereum/types/IERC1820Implementer.d.ts:44

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

packages/ethereum/types/IERC1820Implementer.d.ts:67

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC1820Implementer`](IERC1820Implementer.md)

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

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:47

▸ **off**(`eventName`, `listener`): [`IERC1820Implementer`](IERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:68

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC1820Implementer`](IERC1820Implementer.md)

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

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:51

▸ **on**(`eventName`, `listener`): [`IERC1820Implementer`](IERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:69

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC1820Implementer`](IERC1820Implementer.md)

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

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:55

▸ **once**(`eventName`, `listener`): [`IERC1820Implementer`](IERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:70

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

packages/ethereum/types/IERC1820Implementer.d.ts:74

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`IERC1820Implementer`](IERC1820Implementer.md)

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

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:63

▸ **removeAllListeners**(`eventName?`): [`IERC1820Implementer`](IERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:72

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC1820Implementer`](IERC1820Implementer.md)

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

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:59

▸ **removeListener**(`eventName`, `listener`): [`IERC1820Implementer`](IERC1820Implementer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC1820Implementer`](IERC1820Implementer.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/IERC1820Implementer.d.ts:71

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

node_modules/@ethersproject/contracts/lib/index.d.ts:106

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

node_modules/@ethersproject/contracts/lib/index.d.ts:110

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

node_modules/@ethersproject/contracts/lib/index.d.ts:116
