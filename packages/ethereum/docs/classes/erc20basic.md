[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC20Basic

# Class: ERC20Basic

## Hierarchy

- `Contract`

  ↳ **`ERC20Basic`**

## Table of contents

### Constructors

- [constructor](erc20basic.md#constructor)

### Properties

- [\_deployedPromise](erc20basic.md#_deployedpromise)
- [\_runningEvents](erc20basic.md#_runningevents)
- [\_wrappedEmits](erc20basic.md#_wrappedemits)
- [address](erc20basic.md#address)
- [callStatic](erc20basic.md#callstatic)
- [deployTransaction](erc20basic.md#deploytransaction)
- [estimateGas](erc20basic.md#estimategas)
- [filters](erc20basic.md#filters)
- [functions](erc20basic.md#functions)
- [interface](erc20basic.md#interface)
- [populateTransaction](erc20basic.md#populatetransaction)
- [provider](erc20basic.md#provider)
- [resolvedAddress](erc20basic.md#resolvedaddress)
- [signer](erc20basic.md#signer)

### Methods

- [\_checkRunningEvents](erc20basic.md#_checkrunningevents)
- [\_deployed](erc20basic.md#_deployed)
- [\_wrapEvent](erc20basic.md#_wrapevent)
- [attach](erc20basic.md#attach)
- [balanceOf](erc20basic.md#balanceof)
- [balanceOf(address)](erc20basic.md#balanceof(address))
- [connect](erc20basic.md#connect)
- [deployed](erc20basic.md#deployed)
- [emit](erc20basic.md#emit)
- [fallback](erc20basic.md#fallback)
- [listenerCount](erc20basic.md#listenercount)
- [listeners](erc20basic.md#listeners)
- [off](erc20basic.md#off)
- [on](erc20basic.md#on)
- [once](erc20basic.md#once)
- [queryFilter](erc20basic.md#queryfilter)
- [removeAllListeners](erc20basic.md#removealllisteners)
- [removeListener](erc20basic.md#removelistener)
- [totalSupply](erc20basic.md#totalsupply)
- [totalSupply()](erc20basic.md#totalsupply())
- [transfer](erc20basic.md#transfer)
- [transfer(address,uint256)](erc20basic.md#transfer(address,uint256))
- [getContractAddress](erc20basic.md#getcontractaddress)
- [getInterface](erc20basic.md#getinterface)
- [isIndexed](erc20basic.md#isindexed)

## Constructors

### constructor

• **new ERC20Basic**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `balanceOf` | (`_who`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOf(address)` | (`_who`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transfer(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:144

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
| `balanceOf` | (`_who`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOf(address)` | (`_who`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transfer(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:180

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Transfer` | (`from`: `string`, `to`: `string`, `value`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:169

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `balanceOf` | (`_who`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `balanceOf(address)` | (`_who`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transfer(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:96

___

### interface

• **interface**: `ERC20BasicInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:94

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `balanceOf` | (`_who`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf(address)` | (`_who`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transfer(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:205

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

▸ **attach**(`addressOrName`): [`ERC20Basic`](erc20basic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:55

___

### balanceOf

▸ **balanceOf**(`_who`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_who` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:125

___

### balanceOf(address)

▸ **balanceOf(address)**(`_who`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_who` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:125

___

### connect

▸ **connect**(`signerOrProvider`): [`ERC20Basic`](erc20basic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:54

___

### deployed

▸ **deployed**(): `Promise`<[`ERC20Basic`](erc20basic.md)\>

#### Returns

`Promise`<[`ERC20Basic`](erc20basic.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:56

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

packages/ethereum/types/ERC20Basic.d.ts:58

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

packages/ethereum/types/ERC20Basic.d.ts:81

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC20Basic`](erc20basic.md)

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

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:61

▸ **off**(`eventName`, `listener`): [`ERC20Basic`](erc20basic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:82

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC20Basic`](erc20basic.md)

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

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:65

▸ **on**(`eventName`, `listener`): [`ERC20Basic`](erc20basic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:83

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC20Basic`](erc20basic.md)

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

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:69

▸ **once**(`eventName`, `listener`): [`ERC20Basic`](erc20basic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:84

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

packages/ethereum/types/ERC20Basic.d.ts:88

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`ERC20Basic`](erc20basic.md)

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

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:77

▸ **removeAllListeners**(`eventName?`): [`ERC20Basic`](erc20basic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:86

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC20Basic`](erc20basic.md)

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

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:73

▸ **removeListener**(`eventName`, `listener`): [`ERC20Basic`](erc20basic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC20Basic`](erc20basic.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:85

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

packages/ethereum/types/ERC20Basic.d.ts:121

___

### totalSupply()

▸ **totalSupply()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:121

___

### transfer

▸ **transfer**(`_to`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:132

___

### transfer(address,uint256)

▸ **transfer(address,uint256)**(`_to`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC20Basic.d.ts:136

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
