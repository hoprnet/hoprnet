[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / PermittableToken

# Class: PermittableToken

## Hierarchy

- `BaseContract`

  ↳ **`PermittableToken`**

## Table of contents

### Constructors

- [constructor](PermittableToken.md#constructor)

### Properties

- [\_deployedPromise](PermittableToken.md#_deployedpromise)
- [\_runningEvents](PermittableToken.md#_runningevents)
- [\_wrappedEmits](PermittableToken.md#_wrappedemits)
- [address](PermittableToken.md#address)
- [callStatic](PermittableToken.md#callstatic)
- [deployTransaction](PermittableToken.md#deploytransaction)
- [estimateGas](PermittableToken.md#estimategas)
- [filters](PermittableToken.md#filters)
- [functions](PermittableToken.md#functions)
- [interface](PermittableToken.md#interface)
- [populateTransaction](PermittableToken.md#populatetransaction)
- [provider](PermittableToken.md#provider)
- [resolvedAddress](PermittableToken.md#resolvedaddress)
- [signer](PermittableToken.md#signer)

### Methods

- [DOMAIN\_SEPARATOR](PermittableToken.md#domain_separator)
- [PERMIT\_TYPEHASH](PermittableToken.md#permit_typehash)
- [\_checkRunningEvents](PermittableToken.md#_checkrunningevents)
- [\_deployed](PermittableToken.md#_deployed)
- [\_wrapEvent](PermittableToken.md#_wrapevent)
- [allowance](PermittableToken.md#allowance)
- [approve](PermittableToken.md#approve)
- [attach](PermittableToken.md#attach)
- [balanceOf](PermittableToken.md#balanceof)
- [bridgeContract](PermittableToken.md#bridgecontract)
- [burn](PermittableToken.md#burn)
- [claimTokens](PermittableToken.md#claimtokens)
- [connect](PermittableToken.md#connect)
- [decimals](PermittableToken.md#decimals)
- [decreaseAllowance](PermittableToken.md#decreaseallowance)
- [decreaseApproval](PermittableToken.md#decreaseapproval)
- [deployed](PermittableToken.md#deployed)
- [emit](PermittableToken.md#emit)
- [expirations](PermittableToken.md#expirations)
- [fallback](PermittableToken.md#fallback)
- [finishMinting](PermittableToken.md#finishminting)
- [getTokenInterfacesVersion](PermittableToken.md#gettokeninterfacesversion)
- [increaseAllowance](PermittableToken.md#increaseallowance)
- [increaseApproval](PermittableToken.md#increaseapproval)
- [isBridge](PermittableToken.md#isbridge)
- [listenerCount](PermittableToken.md#listenercount)
- [listeners](PermittableToken.md#listeners)
- [mint](PermittableToken.md#mint)
- [mintingFinished](PermittableToken.md#mintingfinished)
- [move](PermittableToken.md#move)
- [name](PermittableToken.md#name)
- [nonces](PermittableToken.md#nonces)
- [off](PermittableToken.md#off)
- [on](PermittableToken.md#on)
- [once](PermittableToken.md#once)
- [owner](PermittableToken.md#owner)
- [permit](PermittableToken.md#permit)
- [pull](PermittableToken.md#pull)
- [push](PermittableToken.md#push)
- [queryFilter](PermittableToken.md#queryfilter)
- [removeAllListeners](PermittableToken.md#removealllisteners)
- [removeListener](PermittableToken.md#removelistener)
- [renounceOwnership](PermittableToken.md#renounceownership)
- [setBridgeContract](PermittableToken.md#setbridgecontract)
- [symbol](PermittableToken.md#symbol)
- [totalSupply](PermittableToken.md#totalsupply)
- [transfer](PermittableToken.md#transfer)
- [transferAndCall](PermittableToken.md#transferandcall)
- [transferFrom](PermittableToken.md#transferfrom)
- [transferOwnership](PermittableToken.md#transferownership)
- [version](PermittableToken.md#version)
- [getContractAddress](PermittableToken.md#getcontractaddress)
- [getInterface](PermittableToken.md#getinterface)
- [isIndexed](PermittableToken.md#isindexed)

## Constructors

### constructor

• **new PermittableToken**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `DOMAIN_SEPARATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `PERMIT_TYPEHASH` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `balanceOf` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `bridgeContract` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `burn` | (`_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `claimTokens` | (`_token`: `string`, `_to`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `decreaseAllowance` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `decreaseApproval` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `expirations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `finishMinting` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `increaseAllowance` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `move` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `nonces` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `permit` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `pull` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `push` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceOwnership` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `setBridgeContract` | (`_bridgeContract`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferAndCall` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `version` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:705

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
| `DOMAIN_SEPARATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `PERMIT_TYPEHASH` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `balanceOf` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `bridgeContract` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `burn` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `claimTokens` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `decreaseAllowance` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decreaseApproval` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `expirations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `increaseAllowance` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `move` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `nonces` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `permit` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `pull` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `push` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `setBridgeContract` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferAndCall` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `version` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:957

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner?`: `string`, `spender?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `Approval(address,address,uint256)` | (`owner?`: `string`, `spender?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `Burn` | (`burner?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `BigNumber`], `Object`\> |
| `Burn(address,uint256)` | (`burner?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `BigNumber`], `Object`\> |
| `Mint` | (`to?`: `string`, `amount?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `BigNumber`], `Object`\> |
| `Mint(address,uint256)` | (`to?`: `string`, `amount?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `BigNumber`], `Object`\> |
| `MintFinished` | () => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[], `Object`\> |
| `MintFinished()` | () => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[], `Object`\> |
| `OwnershipRenounced` | (`previousOwner?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`], `Object`\> |
| `OwnershipRenounced(address)` | (`previousOwner?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`], `Object`\> |
| `OwnershipTransferred` | (`previousOwner?`: `string`, `newOwner?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], `Object`\> |
| `OwnershipTransferred(address,address)` | (`previousOwner?`: `string`, `newOwner?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], `Object`\> |
| `Transfer(address,address,uint256)` | (`from?`: `string`, `to?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `Transfer(address,address,uint256,bytes)` | (`from?`: `string`, `to?`: `string`, `value?`: ``null``, `data?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`, `string`], `Object`\> |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:864

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DOMAIN_SEPARATOR` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `PERMIT_TYPEHASH` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `approve` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `balanceOf` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `bridgeContract` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `burn` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `claimTokens` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `decreaseAllowance` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decreaseApproval` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `expirations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `increaseAllowance` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `move` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `nonces` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `permit` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `pull` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `push` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `setBridgeContract` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferAndCall` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `version` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:375

___

### interface

• **interface**: `PermittableTokenInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:373

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DOMAIN_SEPARATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `PERMIT_TYPEHASH` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `approve` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `bridgeContract` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `burn` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `claimTokens` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `decreaseAllowance` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decreaseApproval` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `expirations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `increaseAllowance` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `move` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `nonces` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `permit` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `pull` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `push` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `setBridgeContract` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferAndCall` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `version` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:1115

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

### DOMAIN\_SEPARATOR

▸ **DOMAIN_SEPARATOR**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:569

___

### PERMIT\_TYPEHASH

▸ **PERMIT_TYPEHASH**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:565

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

### allowance

▸ **allowance**(`_owner`, `_spender`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `_spender` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:682

___

### approve

▸ **approve**(`_to`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:545

___

### attach

▸ **attach**(`addressOrName`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:334

___

### balanceOf

▸ **balanceOf**(`_owner`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:609

___

### bridgeContract

▸ **bridgeContract**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:674

___

### burn

▸ **burn**(`_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:590

___

### claimTokens

▸ **claimTokens**(`_token`, `_to`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_token` | `string` |
| `_to` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:603

___

### connect

▸ **connect**(`signerOrProvider`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:333

___

### decimals

▸ **decimals**(`overrides?`): `Promise`<`number`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`number`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:567

___

### decreaseAllowance

▸ **decreaseAllowance**(`spender`, `subtractedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | `string` |
| `subtractedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:649

___

### decreaseApproval

▸ **decreaseApproval**(`_spender`, `_subtractedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | `string` |
| `_subtractedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:597

___

### deployed

▸ **deployed**(): `Promise`<[`PermittableToken`](PermittableToken.md)\>

#### Returns

`Promise`<[`PermittableToken`](PermittableToken.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:335

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

### expirations

▸ **expirations**(`arg0`, `arg1`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `arg1` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:699

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

### finishMinting

▸ **finishMinting**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:617

___

### getTokenInterfacesVersion

▸ **getTokenInterfacesVersion**(`overrides?`): `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:623

___

### increaseAllowance

▸ **increaseAllowance**(`_to`, `_addedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_addedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:571

___

### increaseApproval

▸ **increaseApproval**(`_spender`, `_addedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | `string` |
| `_addedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:676

___

### isBridge

▸ **isBridge**(`_address`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_address` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:615

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

packages/ethereum/types/PermittableToken.d.ts:337

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

packages/ethereum/types/PermittableToken.d.ts:360

___

### mint

▸ **mint**(`_to`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:584

___

### mintingFinished

▸ **mintingFinished**(`overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:541

___

### move

▸ **move**(`_from`, `_to`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_from` | `string` |
| `_to` | `string` |
| `_amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:667

___

### name

▸ **name**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:543

___

### nonces

▸ **nonces**(`arg0`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:621

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`PermittableToken`](PermittableToken.md)

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

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:340

▸ **off**(`eventName`, `listener`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:361

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`PermittableToken`](PermittableToken.md)

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

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:344

▸ **on**(`eventName`, `listener`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:362

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`PermittableToken`](PermittableToken.md)

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

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:348

▸ **once**(`eventName`, `listener`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:363

___

### owner

▸ **owner**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:633

___

### permit

▸ **permit**(`_holder`, `_spender`, `_nonce`, `_expiry`, `_allowed`, `_v`, `_r`, `_s`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_holder` | `string` |
| `_spender` | `string` |
| `_nonce` | `BigNumberish` |
| `_expiry` | `BigNumberish` |
| `_allowed` | `boolean` |
| `_v` | `BigNumberish` |
| `_r` | `BytesLike` |
| `_s` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:635

___

### pull

▸ **pull**(`_from`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_from` | `string` |
| `_amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:688

___

### push

▸ **push**(`_to`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:661

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

packages/ethereum/types/PermittableToken.d.ts:367

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`PermittableToken`](PermittableToken.md)

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

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:356

▸ **removeAllListeners**(`eventName?`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:365

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`PermittableToken`](PermittableToken.md)

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

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:352

▸ **removeListener**(`eventName`, `listener`): [`PermittableToken`](PermittableToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`PermittableToken`](PermittableToken.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:364

___

### renounceOwnership

▸ **renounceOwnership**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:611

___

### setBridgeContract

▸ **setBridgeContract**(`_bridgeContract`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_bridgeContract` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:551

___

### symbol

▸ **symbol**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:647

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

packages/ethereum/types/PermittableToken.d.ts:556

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

packages/ethereum/types/PermittableToken.d.ts:655

___

### transferAndCall

▸ **transferAndCall**(`_to`, `_value`, `_data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_value` | `BigNumberish` |
| `_data` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:577

___

### transferFrom

▸ **transferFrom**(`_sender`, `_recipient`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_sender` | `string` |
| `_recipient` | `string` |
| `_amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:558

___

### transferOwnership

▸ **transferOwnership**(`_newOwner`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_newOwner` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:694

___

### version

▸ **version**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:595

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
