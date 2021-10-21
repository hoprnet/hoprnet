[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / IERC20

# Class: IERC20

## Hierarchy

- `BaseContract`

  ↳ **`IERC20`**

## Table of contents

### Constructors

- [constructor](IERC20.md#constructor)

### Properties

- [\_deployedPromise](IERC20.md#_deployedpromise)
- [\_runningEvents](IERC20.md#_runningevents)
- [\_wrappedEmits](IERC20.md#_wrappedemits)
- [address](IERC20.md#address)
- [callStatic](IERC20.md#callstatic)
- [deployTransaction](IERC20.md#deploytransaction)
- [estimateGas](IERC20.md#estimategas)
- [filters](IERC20.md#filters)
- [functions](IERC20.md#functions)
- [interface](IERC20.md#interface)
- [populateTransaction](IERC20.md#populatetransaction)
- [provider](IERC20.md#provider)
- [resolvedAddress](IERC20.md#resolvedaddress)
- [signer](IERC20.md#signer)

### Methods

- [\_checkRunningEvents](IERC20.md#_checkrunningevents)
- [\_deployed](IERC20.md#_deployed)
- [\_wrapEvent](IERC20.md#_wrapevent)
- [allowance](IERC20.md#allowance)
- [approve](IERC20.md#approve)
- [attach](IERC20.md#attach)
- [balanceOf](IERC20.md#balanceof)
- [connect](IERC20.md#connect)
- [deployed](IERC20.md#deployed)
- [emit](IERC20.md#emit)
- [fallback](IERC20.md#fallback)
- [listenerCount](IERC20.md#listenercount)
- [listeners](IERC20.md#listeners)
- [off](IERC20.md#off)
- [on](IERC20.md#on)
- [once](IERC20.md#once)
- [queryFilter](IERC20.md#queryfilter)
- [removeAllListeners](IERC20.md#removealllisteners)
- [removeListener](IERC20.md#removelistener)
- [totalSupply](IERC20.md#totalsupply)
- [transfer](IERC20.md#transfer)
- [transferFrom](IERC20.md#transferfrom)
- [getContractAddress](IERC20.md#getcontractaddress)
- [getInterface](IERC20.md#getinterface)
- [isIndexed](IERC20.md#isindexed)

## Constructors

### constructor

• **new IERC20**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `allowance` | (`owner`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`spender`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `balanceOf` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/types/IERC20.d.ts:191

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
| `allowance` | (`owner`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`spender`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `balanceOf` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/types/IERC20.d.ts:260

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner?`: `string`, `spender?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `Approval(address,address,uint256)` | (`owner?`: `string`, `spender?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `Transfer` | (`from?`: `string`, `to?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `Transfer(address,address,uint256)` | (`from?`: `string`, `to?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/types/IERC20.d.ts:222

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowance` | (`owner`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `approve` | (`spender`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `balanceOf` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/types/IERC20.d.ts:131

___

### interface

• **interface**: `IERC20Interface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/types/IERC20.d.ts:129

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowance` | (`owner`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `approve` | (`spender`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/types/IERC20.d.ts:291

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

### allowance

▸ **allowance**(`owner`, `spender`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `owner` | `string` |
| `spender` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/IERC20.d.ts:162

___

### approve

▸ **approve**(`spender`, `amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | `string` |
| `amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/IERC20.d.ts:168

___

### attach

▸ **attach**(`addressOrName`): [`IERC20`](IERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/types/IERC20.d.ts:90

___

### balanceOf

▸ **balanceOf**(`account`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/IERC20.d.ts:174

___

### connect

▸ **connect**(`signerOrProvider`): [`IERC20`](IERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/types/IERC20.d.ts:89

___

### deployed

▸ **deployed**(): `Promise`<[`IERC20`](IERC20.md)\>

#### Returns

`Promise`<[`IERC20`](IERC20.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/types/IERC20.d.ts:91

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

packages/ethereum/types/IERC20.d.ts:93

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

packages/ethereum/types/IERC20.d.ts:116

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC20`](IERC20.md)

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

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/IERC20.d.ts:96

▸ **off**(`eventName`, `listener`): [`IERC20`](IERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/IERC20.d.ts:117

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC20`](IERC20.md)

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

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/IERC20.d.ts:100

▸ **on**(`eventName`, `listener`): [`IERC20`](IERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/IERC20.d.ts:118

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC20`](IERC20.md)

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

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/IERC20.d.ts:104

▸ **once**(`eventName`, `listener`): [`IERC20`](IERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/IERC20.d.ts:119

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

packages/ethereum/types/IERC20.d.ts:123

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`IERC20`](IERC20.md)

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

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/IERC20.d.ts:112

▸ **removeAllListeners**(`eventName?`): [`IERC20`](IERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/IERC20.d.ts:121

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`IERC20`](IERC20.md)

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

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/IERC20.d.ts:108

▸ **removeListener**(`eventName`, `listener`): [`IERC20`](IERC20.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`IERC20`](IERC20.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/IERC20.d.ts:120

___

### totalSupply

▸ **totalSupply**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/IERC20.d.ts:176

___

### transfer

▸ **transfer**(`recipient`, `amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/IERC20.d.ts:178

___

### transferFrom

▸ **transferFrom**(`sender`, `recipient`, `amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `sender` | `string` |
| `recipient` | `string` |
| `amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/IERC20.d.ts:184

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
