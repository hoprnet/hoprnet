[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC777Recipient

# Class: IERC777Recipient

## Hierarchy

- `Contract`

  ↳ **`IERC777Recipient`**

## Table of contents

### Constructors

- [constructor](ierc777recipient.md#constructor)

### Properties

- [\_deployedPromise](ierc777recipient.md#_deployedpromise)
- [\_runningEvents](ierc777recipient.md#_runningevents)
- [\_wrappedEmits](ierc777recipient.md#_wrappedemits)
- [address](ierc777recipient.md#address)
- [callStatic](ierc777recipient.md#callstatic)
- [deployTransaction](ierc777recipient.md#deploytransaction)
- [estimateGas](ierc777recipient.md#estimategas)
- [filters](ierc777recipient.md#filters)
- [functions](ierc777recipient.md#functions)
- [interface](ierc777recipient.md#interface)
- [populateTransaction](ierc777recipient.md#populatetransaction)
- [provider](ierc777recipient.md#provider)
- [resolvedAddress](ierc777recipient.md#resolvedaddress)
- [signer](ierc777recipient.md#signer)

### Methods

- [\_checkRunningEvents](ierc777recipient.md#_checkrunningevents)
- [\_deployed](ierc777recipient.md#_deployed)
- [\_wrapEvent](ierc777recipient.md#_wrapevent)
- [attach](ierc777recipient.md#attach)
- [connect](ierc777recipient.md#connect)
- [deployed](ierc777recipient.md#deployed)
- [emit](ierc777recipient.md#emit)
- [fallback](ierc777recipient.md#fallback)
- [listenerCount](ierc777recipient.md#listenercount)
- [listeners](ierc777recipient.md#listeners)
- [off](ierc777recipient.md#off)
- [on](ierc777recipient.md#on)
- [once](ierc777recipient.md#once)
- [queryFilter](ierc777recipient.md#queryfilter)
- [removeAllListeners](ierc777recipient.md#removealllisteners)
- [removeListener](ierc777recipient.md#removelistener)
- [tokensReceived](ierc777recipient.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](ierc777recipient.md#tokensreceived(address,address,address,uint256,bytes,bytes))
- [getContractAddress](ierc777recipient.md#getcontractaddress)
- [getInterface](ierc777recipient.md#getinterface)
- [isIndexed](ierc777recipient.md#isindexed)

## Constructors

### constructor

• **new IERC777Recipient**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:125

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

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:149

___

### filters

• **filters**: `Object`

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:147

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:83

___

### interface

• **interface**: `IERC777RecipientInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:81

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:171

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

▸ **attach**(`addressOrName`): [`IERC777Recipient`](ierc777recipient.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:42

___

### connect

▸ **connect**(`signerOrProvider`): [`IERC777Recipient`](ierc777recipient.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:41

___

### deployed

▸ **deployed**(): `Promise`<[`IERC777Recipient`](ierc777recipient.md)\>

#### Returns

`Promise`<[`IERC777Recipient`](ierc777recipient.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:43

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

packages/ethereum/types/IERC777Recipient.d.ts:45

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

packages/ethereum/types/IERC777Recipient.d.ts:68

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC777Recipient`](ierc777recipient.md)

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

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:48

▸ **off**(`eventName`, `listener`): [`IERC777Recipient`](ierc777recipient.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:69

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC777Recipient`](ierc777recipient.md)

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

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:52

▸ **on**(`eventName`, `listener`): [`IERC777Recipient`](ierc777recipient.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:70

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC777Recipient`](ierc777recipient.md)

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

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:56

▸ **once**(`eventName`, `listener`): [`IERC777Recipient`](ierc777recipient.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:71

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

packages/ethereum/types/IERC777Recipient.d.ts:75

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`IERC777Recipient`](ierc777recipient.md)

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

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:64

▸ **removeAllListeners**(`eventName?`): [`IERC777Recipient`](ierc777recipient.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:73

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC777Recipient`](ierc777recipient.md)

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

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:60

▸ **removeListener**(`eventName`, `listener`): [`IERC777Recipient`](ierc777recipient.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC777Recipient`](ierc777recipient.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:72

___

### tokensReceived

▸ **tokensReceived**(`operator`, `from`, `to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `from` | `string` |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:105

___

### tokensReceived(address,address,address,uint256,bytes,bytes)

▸ **tokensReceived(address,address,address,uint256,bytes,bytes)**(`operator`, `from`, `to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `from` | `string` |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/IERC777Recipient.d.ts:113

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
