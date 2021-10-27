[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprWrapperProxy

# Class: HoprWrapperProxy

## Hierarchy

- `BaseContract`

  ↳ **`HoprWrapperProxy`**

## Table of contents

### Constructors

- [constructor](HoprWrapperProxy.md#constructor)

### Properties

- [\_deployedPromise](HoprWrapperProxy.md#_deployedpromise)
- [\_runningEvents](HoprWrapperProxy.md#_runningevents)
- [\_wrappedEmits](HoprWrapperProxy.md#_wrappedemits)
- [address](HoprWrapperProxy.md#address)
- [callStatic](HoprWrapperProxy.md#callstatic)
- [deployTransaction](HoprWrapperProxy.md#deploytransaction)
- [estimateGas](HoprWrapperProxy.md#estimategas)
- [filters](HoprWrapperProxy.md#filters)
- [functions](HoprWrapperProxy.md#functions)
- [interface](HoprWrapperProxy.md#interface)
- [populateTransaction](HoprWrapperProxy.md#populatetransaction)
- [provider](HoprWrapperProxy.md#provider)
- [resolvedAddress](HoprWrapperProxy.md#resolvedaddress)
- [signer](HoprWrapperProxy.md#signer)

### Methods

- [ERC1820\_REGISTRY](HoprWrapperProxy.md#erc1820_registry)
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH](HoprWrapperProxy.md#tokens_recipient_interface_hash)
- [WRAPPER](HoprWrapperProxy.md#wrapper)
- [WXHOPR\_TOKEN](HoprWrapperProxy.md#wxhopr_token)
- [XDAI\_MULTISIG](HoprWrapperProxy.md#xdai_multisig)
- [XHOPR\_TOKEN](HoprWrapperProxy.md#xhopr_token)
- [\_checkRunningEvents](HoprWrapperProxy.md#_checkrunningevents)
- [\_deployed](HoprWrapperProxy.md#_deployed)
- [\_wrapEvent](HoprWrapperProxy.md#_wrapevent)
- [attach](HoprWrapperProxy.md#attach)
- [connect](HoprWrapperProxy.md#connect)
- [deployed](HoprWrapperProxy.md#deployed)
- [emit](HoprWrapperProxy.md#emit)
- [fallback](HoprWrapperProxy.md#fallback)
- [listenerCount](HoprWrapperProxy.md#listenercount)
- [listeners](HoprWrapperProxy.md#listeners)
- [off](HoprWrapperProxy.md#off)
- [on](HoprWrapperProxy.md#on)
- [onTokenTransfer](HoprWrapperProxy.md#ontokentransfer)
- [once](HoprWrapperProxy.md#once)
- [queryFilter](HoprWrapperProxy.md#queryfilter)
- [recoverTokens](HoprWrapperProxy.md#recovertokens)
- [removeAllListeners](HoprWrapperProxy.md#removealllisteners)
- [removeListener](HoprWrapperProxy.md#removelistener)
- [tokensReceived](HoprWrapperProxy.md#tokensreceived)
- [getContractAddress](HoprWrapperProxy.md#getcontractaddress)
- [getInterface](HoprWrapperProxy.md#getinterface)
- [isIndexed](HoprWrapperProxy.md#isindexed)

## Constructors

### constructor

• **new HoprWrapperProxy**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `ERC1820_REGISTRY` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `WRAPPER` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `WXHOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `XDAI_MULTISIG` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `XHOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `recoverTokens` | (`token`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:235

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
| `ERC1820_REGISTRY` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `WRAPPER` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `WXHOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `XDAI_MULTISIG` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `XHOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `recoverTokens` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:296

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FowardedFrom` | (`from?`: ``null``, `amount?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `BigNumber`], `Object`\> |
| `FowardedFrom(address,uint256)` | (`from?`: ``null``, `amount?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `BigNumber`], `Object`\> |
| `FowardedTo` | (`to?`: ``null``, `amount?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `BigNumber`], `Object`\> |
| `FowardedTo(address,uint256)` | (`to?`: ``null``, `amount?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `BigNumber`], `Object`\> |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:268

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `ERC1820_REGISTRY` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `WRAPPER` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `WXHOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `XDAI_MULTISIG` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `XHOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `recoverTokens` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:163

___

### interface

• **interface**: `HoprWrapperProxyInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:161

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `ERC1820_REGISTRY` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `WRAPPER` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `WXHOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `XDAI_MULTISIG` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `XHOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `recoverTokens` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:334

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

### ERC1820\_REGISTRY

▸ **ERC1820_REGISTRY**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:201

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH

▸ **TOKENS_RECIPIENT_INTERFACE_HASH**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:203

___

### WRAPPER

▸ **WRAPPER**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:205

___

### WXHOPR\_TOKEN

▸ **WXHOPR_TOKEN**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:207

___

### XDAI\_MULTISIG

▸ **XDAI_MULTISIG**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:209

___

### XHOPR\_TOKEN

▸ **XHOPR_TOKEN**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:211

___

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

▸ **attach**(`addressOrName`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:122

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:121

___

### deployed

▸ **deployed**(): `Promise`<[`HoprWrapperProxy`](HoprWrapperProxy.md)\>

#### Returns

`Promise`<[`HoprWrapperProxy`](HoprWrapperProxy.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:123

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

packages/ethereum/types/HoprWrapperProxy.d.ts:125

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

packages/ethereum/types/HoprWrapperProxy.d.ts:148

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

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

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:128

▸ **off**(`eventName`, `listener`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:149

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

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

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:132

▸ **on**(`eventName`, `listener`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:150

___

### onTokenTransfer

▸ **onTokenTransfer**(`_from`, `_value`, `_data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_from` | `string` |
| `_value` | `BigNumberish` |
| `_data` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:213

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

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

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:136

▸ **once**(`eventName`, `listener`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:151

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

packages/ethereum/types/HoprWrapperProxy.d.ts:155

___

### recoverTokens

▸ **recoverTokens**(`token`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `token` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:220

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

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

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:144

▸ **removeAllListeners**(`eventName?`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:153

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

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

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:140

▸ **removeListener**(`eventName`, `listener`): [`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprWrapperProxy`](HoprWrapperProxy.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/HoprWrapperProxy.d.ts:152

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

packages/ethereum/types/HoprWrapperProxy.d.ts:225

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
