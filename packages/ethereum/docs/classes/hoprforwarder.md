[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprForwarder

# Class: HoprForwarder

## Hierarchy

- `Contract`

  ↳ **`HoprForwarder`**

## Table of contents

### Constructors

- [constructor](hoprforwarder.md#constructor)

### Properties

- [\_deployedPromise](hoprforwarder.md#_deployedpromise)
- [\_runningEvents](hoprforwarder.md#_runningevents)
- [\_wrappedEmits](hoprforwarder.md#_wrappedemits)
- [address](hoprforwarder.md#address)
- [callStatic](hoprforwarder.md#callstatic)
- [deployTransaction](hoprforwarder.md#deploytransaction)
- [estimateGas](hoprforwarder.md#estimategas)
- [filters](hoprforwarder.md#filters)
- [functions](hoprforwarder.md#functions)
- [interface](hoprforwarder.md#interface)
- [populateTransaction](hoprforwarder.md#populatetransaction)
- [provider](hoprforwarder.md#provider)
- [resolvedAddress](hoprforwarder.md#resolvedaddress)
- [signer](hoprforwarder.md#signer)

### Methods

- [ERC1820\_REGISTRY](hoprforwarder.md#erc1820_registry)
- [ERC1820\_REGISTRY()](hoprforwarder.md#erc1820_registry())
- [HOPR\_TOKEN](hoprforwarder.md#hopr_token)
- [HOPR\_TOKEN()](hoprforwarder.md#hopr_token())
- [MULTISIG](hoprforwarder.md#multisig)
- [MULTISIG()](hoprforwarder.md#multisig())
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH](hoprforwarder.md#tokens_recipient_interface_hash)
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH()](hoprforwarder.md#tokens_recipient_interface_hash())
- [\_checkRunningEvents](hoprforwarder.md#_checkrunningevents)
- [\_deployed](hoprforwarder.md#_deployed)
- [\_wrapEvent](hoprforwarder.md#_wrapevent)
- [attach](hoprforwarder.md#attach)
- [connect](hoprforwarder.md#connect)
- [deployed](hoprforwarder.md#deployed)
- [emit](hoprforwarder.md#emit)
- [fallback](hoprforwarder.md#fallback)
- [listenerCount](hoprforwarder.md#listenercount)
- [listeners](hoprforwarder.md#listeners)
- [off](hoprforwarder.md#off)
- [on](hoprforwarder.md#on)
- [once](hoprforwarder.md#once)
- [queryFilter](hoprforwarder.md#queryfilter)
- [recoverTokens](hoprforwarder.md#recovertokens)
- [recoverTokens(address)](hoprforwarder.md#recovertokens(address))
- [removeAllListeners](hoprforwarder.md#removealllisteners)
- [removeListener](hoprforwarder.md#removelistener)
- [tokensReceived](hoprforwarder.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](hoprforwarder.md#tokensreceived(address,address,address,uint256,bytes,bytes))
- [getContractAddress](hoprforwarder.md#getcontractaddress)
- [getInterface](hoprforwarder.md#getinterface)
- [isIndexed](hoprforwarder.md#isindexed)

## Constructors

### constructor

• **new HoprForwarder**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `ERC1820_REGISTRY` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `ERC1820_REGISTRY()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `HOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `HOPR_TOKEN()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `MULTISIG` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `MULTISIG()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `recoverTokens` | (`token`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `recoverTokens(address)` | (`token`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:219

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
| `ERC1820_REGISTRY` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `ERC1820_REGISTRY()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `HOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `HOPR_TOKEN()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `MULTISIG` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `MULTISIG()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `recoverTokens` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `recoverTokens(address)` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:268

___

### filters

• **filters**: `Object`

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:266

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `ERC1820_REGISTRY` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `ERC1820_REGISTRY()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `HOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `HOPR_TOKEN()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `MULTISIG` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `MULTISIG()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `recoverTokens` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `recoverTokens(address)` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:119

___

### interface

• **interface**: `HoprForwarderInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:117

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `ERC1820_REGISTRY` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `ERC1820_REGISTRY()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `HOPR_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `HOPR_TOKEN()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `MULTISIG` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `MULTISIG()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `recoverTokens` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `recoverTokens(address)` | (`token`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:320

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

### ERC1820\_REGISTRY

▸ **ERC1820_REGISTRY**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:171

___

### ERC1820\_REGISTRY()

▸ **ERC1820_REGISTRY()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:171

___

### HOPR\_TOKEN

▸ **HOPR_TOKEN**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:175

___

### HOPR\_TOKEN()

▸ **HOPR_TOKEN()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:175

___

### MULTISIG

▸ **MULTISIG**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:179

___

### MULTISIG()

▸ **MULTISIG()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:179

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

packages/ethereum/types/HoprForwarder.d.ts:183

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH()

▸ **TOKENS_RECIPIENT_INTERFACE_HASH()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:183

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

▸ **attach**(`addressOrName`): [`HoprForwarder`](hoprforwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:78

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprForwarder`](hoprforwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:77

___

### deployed

▸ **deployed**(): `Promise`<[`HoprForwarder`](hoprforwarder.md)\>

#### Returns

`Promise`<[`HoprForwarder`](hoprforwarder.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:79

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

packages/ethereum/types/HoprForwarder.d.ts:81

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

packages/ethereum/types/HoprForwarder.d.ts:104

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprForwarder`](hoprforwarder.md)

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

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:84

▸ **off**(`eventName`, `listener`): [`HoprForwarder`](hoprforwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:105

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprForwarder`](hoprforwarder.md)

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

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:88

▸ **on**(`eventName`, `listener`): [`HoprForwarder`](hoprforwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:106

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprForwarder`](hoprforwarder.md)

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

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:92

▸ **once**(`eventName`, `listener`): [`HoprForwarder`](hoprforwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:107

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

packages/ethereum/types/HoprForwarder.d.ts:111

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

packages/ethereum/types/HoprForwarder.d.ts:189

___

### recoverTokens(address)

▸ **recoverTokens(address)**(`token`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `token` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:192

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`HoprForwarder`](hoprforwarder.md)

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

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:100

▸ **removeAllListeners**(`eventName?`): [`HoprForwarder`](hoprforwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:109

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprForwarder`](hoprforwarder.md)

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

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:96

▸ **removeListener**(`eventName`, `listener`): [`HoprForwarder`](hoprforwarder.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprForwarder`](hoprforwarder.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprForwarder.d.ts:108

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

packages/ethereum/types/HoprForwarder.d.ts:199

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

packages/ethereum/types/HoprForwarder.d.ts:207

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
