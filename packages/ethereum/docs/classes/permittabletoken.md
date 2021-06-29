[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / PermittableToken

# Class: PermittableToken

## Hierarchy

- `Contract`

  ↳ **`PermittableToken`**

## Table of contents

### Constructors

- [constructor](permittabletoken.md#constructor)

### Properties

- [\_deployedPromise](permittabletoken.md#_deployedpromise)
- [\_runningEvents](permittabletoken.md#_runningevents)
- [\_wrappedEmits](permittabletoken.md#_wrappedemits)
- [address](permittabletoken.md#address)
- [callStatic](permittabletoken.md#callstatic)
- [deployTransaction](permittabletoken.md#deploytransaction)
- [estimateGas](permittabletoken.md#estimategas)
- [filters](permittabletoken.md#filters)
- [functions](permittabletoken.md#functions)
- [interface](permittabletoken.md#interface)
- [populateTransaction](permittabletoken.md#populatetransaction)
- [provider](permittabletoken.md#provider)
- [resolvedAddress](permittabletoken.md#resolvedaddress)
- [signer](permittabletoken.md#signer)

### Methods

- [DOMAIN\_SEPARATOR](permittabletoken.md#domain_separator)
- [DOMAIN\_SEPARATOR()](permittabletoken.md#domain_separator())
- [PERMIT\_TYPEHASH](permittabletoken.md#permit_typehash)
- [PERMIT\_TYPEHASH()](permittabletoken.md#permit_typehash())
- [\_checkRunningEvents](permittabletoken.md#_checkrunningevents)
- [\_deployed](permittabletoken.md#_deployed)
- [\_wrapEvent](permittabletoken.md#_wrapevent)
- [allowance](permittabletoken.md#allowance)
- [allowance(address,address)](permittabletoken.md#allowance(address,address))
- [approve](permittabletoken.md#approve)
- [approve(address,uint256)](permittabletoken.md#approve(address,uint256))
- [attach](permittabletoken.md#attach)
- [balanceOf](permittabletoken.md#balanceof)
- [balanceOf(address)](permittabletoken.md#balanceof(address))
- [bridgeContract](permittabletoken.md#bridgecontract)
- [bridgeContract()](permittabletoken.md#bridgecontract())
- [burn](permittabletoken.md#burn)
- [burn(uint256)](permittabletoken.md#burn(uint256))
- [claimTokens](permittabletoken.md#claimtokens)
- [claimTokens(address,address)](permittabletoken.md#claimtokens(address,address))
- [connect](permittabletoken.md#connect)
- [decimals](permittabletoken.md#decimals)
- [decimals()](permittabletoken.md#decimals())
- [decreaseAllowance](permittabletoken.md#decreaseallowance)
- [decreaseAllowance(address,uint256)](permittabletoken.md#decreaseallowance(address,uint256))
- [decreaseApproval](permittabletoken.md#decreaseapproval)
- [decreaseApproval(address,uint256)](permittabletoken.md#decreaseapproval(address,uint256))
- [deployed](permittabletoken.md#deployed)
- [emit](permittabletoken.md#emit)
- [expirations](permittabletoken.md#expirations)
- [expirations(address,address)](permittabletoken.md#expirations(address,address))
- [fallback](permittabletoken.md#fallback)
- [finishMinting](permittabletoken.md#finishminting)
- [finishMinting()](permittabletoken.md#finishminting())
- [getTokenInterfacesVersion](permittabletoken.md#gettokeninterfacesversion)
- [getTokenInterfacesVersion()](permittabletoken.md#gettokeninterfacesversion())
- [increaseAllowance](permittabletoken.md#increaseallowance)
- [increaseAllowance(address,uint256)](permittabletoken.md#increaseallowance(address,uint256))
- [increaseApproval](permittabletoken.md#increaseapproval)
- [increaseApproval(address,uint256)](permittabletoken.md#increaseapproval(address,uint256))
- [isBridge](permittabletoken.md#isbridge)
- [isBridge(address)](permittabletoken.md#isbridge(address))
- [listenerCount](permittabletoken.md#listenercount)
- [listeners](permittabletoken.md#listeners)
- [mint](permittabletoken.md#mint)
- [mint(address,uint256)](permittabletoken.md#mint(address,uint256))
- [mintingFinished](permittabletoken.md#mintingfinished)
- [mintingFinished()](permittabletoken.md#mintingfinished())
- [move](permittabletoken.md#move)
- [move(address,address,uint256)](permittabletoken.md#move(address,address,uint256))
- [name](permittabletoken.md#name)
- [name()](permittabletoken.md#name())
- [nonces](permittabletoken.md#nonces)
- [nonces(address)](permittabletoken.md#nonces(address))
- [off](permittabletoken.md#off)
- [on](permittabletoken.md#on)
- [once](permittabletoken.md#once)
- [owner](permittabletoken.md#owner)
- [owner()](permittabletoken.md#owner())
- [permit](permittabletoken.md#permit)
- [permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)](permittabletoken.md#permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32))
- [pull](permittabletoken.md#pull)
- [pull(address,uint256)](permittabletoken.md#pull(address,uint256))
- [push](permittabletoken.md#push)
- [push(address,uint256)](permittabletoken.md#push(address,uint256))
- [queryFilter](permittabletoken.md#queryfilter)
- [removeAllListeners](permittabletoken.md#removealllisteners)
- [removeListener](permittabletoken.md#removelistener)
- [renounceOwnership](permittabletoken.md#renounceownership)
- [renounceOwnership()](permittabletoken.md#renounceownership())
- [setBridgeContract](permittabletoken.md#setbridgecontract)
- [setBridgeContract(address)](permittabletoken.md#setbridgecontract(address))
- [symbol](permittabletoken.md#symbol)
- [symbol()](permittabletoken.md#symbol())
- [totalSupply](permittabletoken.md#totalsupply)
- [totalSupply()](permittabletoken.md#totalsupply())
- [transfer](permittabletoken.md#transfer)
- [transfer(address,uint256)](permittabletoken.md#transfer(address,uint256))
- [transferAndCall](permittabletoken.md#transferandcall)
- [transferAndCall(address,uint256,bytes)](permittabletoken.md#transferandcall(address,uint256,bytes))
- [transferFrom](permittabletoken.md#transferfrom)
- [transferFrom(address,address,uint256)](permittabletoken.md#transferfrom(address,address,uint256))
- [transferOwnership](permittabletoken.md#transferownership)
- [transferOwnership(address)](permittabletoken.md#transferownership(address))
- [version](permittabletoken.md#version)
- [version()](permittabletoken.md#version())
- [getContractAddress](permittabletoken.md#getcontractaddress)
- [getInterface](permittabletoken.md#getinterface)
- [isIndexed](permittabletoken.md#isindexed)

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
| `DOMAIN_SEPARATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `DOMAIN_SEPARATOR()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `PERMIT_TYPEHASH` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `PERMIT_TYPEHASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance(address,address)` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `approve(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `balanceOf` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOf(address)` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `bridgeContract` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `bridgeContract()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `burn` | (`_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `burn(uint256)` | (`_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `claimTokens` | (`_token`: `string`, `_to`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `claimTokens(address,address)` | (`_token`: `string`, `_to`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `decreaseAllowance` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `decreaseAllowance(address,uint256)` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `decreaseApproval` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `decreaseApproval(address,uint256)` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `expirations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `expirations(address,address)` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `finishMinting` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `finishMinting()` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `getTokenInterfacesVersion()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `increaseAllowance` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `increaseAllowance(address,uint256)` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `increaseApproval(address,uint256)` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isBridge(address)` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mint(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mintingFinished()` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `move` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `move(address,address,uint256)` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `nonces` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `nonces(address)` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `permit` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `pull` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `pull(address,uint256)` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `push` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `push(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceOwnership` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceOwnership()` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `setBridgeContract` | (`_bridgeContract`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `setBridgeContract(address)` | (`_bridgeContract`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `symbol()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transfer(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferAndCall` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferAndCall(address,uint256,bytes)` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom(address,address,uint256)` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `transferOwnership(address)` | (`_newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `version` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `version()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:1012

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
| `DOMAIN_SEPARATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `DOMAIN_SEPARATOR()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `PERMIT_TYPEHASH` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `PERMIT_TYPEHASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance(address,address)` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `approve(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `balanceOf` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOf(address)` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `bridgeContract` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `bridgeContract()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `burn` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `burn(uint256)` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `claimTokens` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `claimTokens(address,address)` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `decreaseAllowance` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decreaseAllowance(address,uint256)` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decreaseApproval` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decreaseApproval(address,uint256)` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `expirations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `expirations(address,address)` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `finishMinting()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTokenInterfacesVersion()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `increaseAllowance` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `increaseAllowance(address,uint256)` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `increaseApproval(address,uint256)` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isBridge(address)` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `mint(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `mintingFinished()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `move` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `move(address,address,uint256)` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `nonces` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `nonces(address)` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `permit` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `pull` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `pull(address,uint256)` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `push` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `push(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceOwnership()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `setBridgeContract` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `setBridgeContract(address)` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `symbol()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transfer(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferAndCall` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferAndCall(address,uint256,bytes)` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom(address,address,uint256)` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferOwnership(address)` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `version` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `version()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:1388

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner`: `string`, `spender`: `string`, `value`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `Burn` | (`burner`: `string`, `value`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `BigNumber`], `Object`\> |
| `Mint` | (`to`: `string`, `amount`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `BigNumber`], `Object`\> |
| `MintFinished` | () => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[], `Object`\> |
| `OwnershipRenounced` | (`previousOwner`: `string`) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`], `Object`\> |
| `OwnershipTransferred` | (`previousOwner`: `string`, `newOwner`: `string`) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`], `Object`\> |
| `Transfer` | (`from`: `string`, `to`: `string`, `value`: ``null``, `data`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `BigNumber`, `string`], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:1340

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DOMAIN_SEPARATOR` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `DOMAIN_SEPARATOR()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `PERMIT_TYPEHASH` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `PERMIT_TYPEHASH()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `allowance(address,address)` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `approve` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `approve(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `balanceOf` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `balanceOf(address)` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `bridgeContract` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `bridgeContract()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `burn` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `burn(uint256)` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `claimTokens` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `claimTokens(address,address)` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `decreaseAllowance` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decreaseAllowance(address,uint256)` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decreaseApproval` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decreaseApproval(address,uint256)` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `expirations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `expirations(address,address)` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `finishMinting()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `getTokenInterfacesVersion()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `increaseAllowance` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `increaseAllowance(address,uint256)` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `increaseApproval(address,uint256)` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isBridge(address)` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `mint(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `mintingFinished()` | (`overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `move` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `move(address,address,uint256)` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `nonces` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `nonces(address)` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `permit` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `pull` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `pull(address,uint256)` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `push` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `push(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceOwnership()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `setBridgeContract` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `setBridgeContract(address)` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `symbol()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transfer(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferAndCall` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferAndCall(address,uint256,bytes)` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom(address,address,uint256)` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferOwnership(address)` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `version` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `version()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:336

___

### interface

• **interface**: `PermittableTokenInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:334

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DOMAIN_SEPARATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `DOMAIN_SEPARATOR()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `PERMIT_TYPEHASH` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `PERMIT_TYPEHASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allowance(address,address)` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `approve` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `approve(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf(address)` | (`_owner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `bridgeContract` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `bridgeContract()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `burn` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `burn(uint256)` | (`_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `claimTokens` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `claimTokens(address,address)` | (`_token`: `string`, `_to`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `decreaseAllowance` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decreaseAllowance(address,uint256)` | (`spender`: `string`, `subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decreaseApproval` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decreaseApproval(address,uint256)` | (`_spender`: `string`, `_subtractedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `expirations` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `expirations(address,address)` | (`arg0`: `string`, `arg1`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `finishMinting()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getTokenInterfacesVersion()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `increaseAllowance` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `increaseAllowance(address,uint256)` | (`_to`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `increaseApproval(address,uint256)` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isBridge(address)` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `mint(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `mintingFinished()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `move` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `move(address,address,uint256)` | (`_from`: `string`, `_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `nonces` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `nonces(address)` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `permit` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)` | (`_holder`: `string`, `_spender`: `string`, `_nonce`: `BigNumberish`, `_expiry`: `BigNumberish`, `_allowed`: `boolean`, `_v`: `BigNumberish`, `_r`: `BytesLike`, `_s`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `pull` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `pull(address,uint256)` | (`_from`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `push` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `push(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `setBridgeContract` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `setBridgeContract(address)` | (`_bridgeContract`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `symbol()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transfer` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transfer(address,uint256)` | (`_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferAndCall` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferAndCall(address,uint256,bytes)` | (`_to`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom(address,address,uint256)` | (`_sender`: `string`, `_recipient`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership(address)` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `version` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `version()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:1713

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

### DOMAIN\_SEPARATOR

▸ **DOMAIN_SEPARATOR**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:731

___

### DOMAIN\_SEPARATOR()

▸ **DOMAIN_SEPARATOR()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:731

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

packages/ethereum/types/PermittableToken.d.ts:723

___

### PERMIT\_TYPEHASH()

▸ **PERMIT_TYPEHASH()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:723

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

packages/ethereum/types/PermittableToken.d.ts:966

___

### allowance(address,address)

▸ **allowance(address,address)**(`_owner`, `_spender`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `_spender` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:970

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

packages/ethereum/types/PermittableToken.d.ts:683

___

### approve(address,uint256)

▸ **approve(address,uint256)**(`_to`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:687

___

### attach

▸ **attach**(`addressOrName`): [`PermittableToken`](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:295

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

packages/ethereum/types/PermittableToken.d.ts:811

___

### balanceOf(address)

▸ **balanceOf(address)**(`_owner`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:811

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

packages/ethereum/types/PermittableToken.d.ts:950

___

### bridgeContract()

▸ **bridgeContract()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:950

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

packages/ethereum/types/PermittableToken.d.ts:773

___

### burn(uint256)

▸ **burn(uint256)**(`_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:776

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

packages/ethereum/types/PermittableToken.d.ts:799

___

### claimTokens(address,address)

▸ **claimTokens(address,address)**(`_token`, `_to`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_token` | `string` |
| `_to` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:803

___

### connect

▸ **connect**(`signerOrProvider`): [`PermittableToken`](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:294

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

packages/ethereum/types/PermittableToken.d.ts:727

___

### decimals()

▸ **decimals()**(`overrides?`): `Promise`<`number`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`number`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:727

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

packages/ethereum/types/PermittableToken.d.ts:900

___

### decreaseAllowance(address,uint256)

▸ **decreaseAllowance(address,uint256)**(`spender`, `subtractedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | `string` |
| `subtractedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:904

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

packages/ethereum/types/PermittableToken.d.ts:787

___

### decreaseApproval(address,uint256)

▸ **decreaseApproval(address,uint256)**(`_spender`, `_subtractedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | `string` |
| `_subtractedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:791

___

### deployed

▸ **deployed**(): `Promise`<[`PermittableToken`](permittabletoken.md)\>

#### Returns

`Promise`<[`PermittableToken`](permittabletoken.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:296

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

packages/ethereum/types/PermittableToken.d.ts:1000

___

### expirations(address,address)

▸ **expirations(address,address)**(`arg0`, `arg1`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `arg1` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:1004

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

### finishMinting

▸ **finishMinting**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:833

___

### finishMinting()

▸ **finishMinting()**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:835

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

packages/ethereum/types/PermittableToken.d.ts:848

___

### getTokenInterfacesVersion()

▸ **getTokenInterfacesVersion()**(`overrides?`): `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:856

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

packages/ethereum/types/PermittableToken.d.ts:735

___

### increaseAllowance(address,uint256)

▸ **increaseAllowance(address,uint256)**(`_to`, `_addedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_addedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:739

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

packages/ethereum/types/PermittableToken.d.ts:954

___

### increaseApproval(address,uint256)

▸ **increaseApproval(address,uint256)**(`_spender`, `_addedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | `string` |
| `_addedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:958

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

packages/ethereum/types/PermittableToken.d.ts:826

___

### isBridge(address)

▸ **isBridge(address)**(`_address`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_address` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:826

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

packages/ethereum/types/PermittableToken.d.ts:298

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

packages/ethereum/types/PermittableToken.d.ts:321

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

packages/ethereum/types/PermittableToken.d.ts:761

___

### mint(address,uint256)

▸ **mint(address,uint256)**(`_to`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:765

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

packages/ethereum/types/PermittableToken.d.ts:675

___

### mintingFinished()

▸ **mintingFinished()**(`overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:675

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

packages/ethereum/types/PermittableToken.d.ts:936

___

### move(address,address,uint256)

▸ **move(address,address,uint256)**(`_from`, `_to`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/PermittableToken.d.ts:941

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

packages/ethereum/types/PermittableToken.d.ts:679

___

### name()

▸ **name()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:679

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

packages/ethereum/types/PermittableToken.d.ts:841

___

### nonces(address)

▸ **nonces(address)**(`arg0`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:841

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`PermittableToken`](permittabletoken.md)

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

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:301

▸ **off**(`eventName`, `listener`): [`PermittableToken`](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:322

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`PermittableToken`](permittabletoken.md)

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

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:305

▸ **on**(`eventName`, `listener`): [`PermittableToken`](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:323

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`PermittableToken`](permittabletoken.md)

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

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:309

▸ **once**(`eventName`, `listener`): [`PermittableToken`](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:324

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

packages/ethereum/types/PermittableToken.d.ts:868

___

### owner()

▸ **owner()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:868

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

packages/ethereum/types/PermittableToken.d.ts:872

___

### permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)

▸ **permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)**(`_holder`, `_spender`, `_nonce`, `_expiry`, `_allowed`, `_v`, `_r`, `_s`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/PermittableToken.d.ts:882

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

packages/ethereum/types/PermittableToken.d.ts:978

___

### pull(address,uint256)

▸ **pull(address,uint256)**(`_from`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_from` | `string` |
| `_amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:982

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

packages/ethereum/types/PermittableToken.d.ts:924

___

### push(address,uint256)

▸ **push(address,uint256)**(`_to`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_to` | `string` |
| `_amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:928

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

packages/ethereum/types/PermittableToken.d.ts:328

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`PermittableToken`](permittabletoken.md)

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

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:317

▸ **removeAllListeners**(`eventName?`): [`PermittableToken`](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:326

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`PermittableToken`](permittabletoken.md)

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

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:313

▸ **removeListener**(`eventName`, `listener`): [`PermittableToken`](permittabletoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`PermittableToken`](permittabletoken.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:325

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

packages/ethereum/types/PermittableToken.d.ts:818

___

### renounceOwnership()

▸ **renounceOwnership()**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:820

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

packages/ethereum/types/PermittableToken.d.ts:695

___

### setBridgeContract(address)

▸ **setBridgeContract(address)**(`_bridgeContract`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_bridgeContract` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:698

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

packages/ethereum/types/PermittableToken.d.ts:896

___

### symbol()

▸ **symbol()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:896

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

packages/ethereum/types/PermittableToken.d.ts:705

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

packages/ethereum/types/PermittableToken.d.ts:705

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

packages/ethereum/types/PermittableToken.d.ts:912

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

packages/ethereum/types/PermittableToken.d.ts:916

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

packages/ethereum/types/PermittableToken.d.ts:747

___

### transferAndCall(address,uint256,bytes)

▸ **transferAndCall(address,uint256,bytes)**(`_to`, `_value`, `_data`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/PermittableToken.d.ts:752

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

packages/ethereum/types/PermittableToken.d.ts:709

___

### transferFrom(address,address,uint256)

▸ **transferFrom(address,address,uint256)**(`_sender`, `_recipient`, `_amount`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/PermittableToken.d.ts:714

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

packages/ethereum/types/PermittableToken.d.ts:990

___

### transferOwnership(address)

▸ **transferOwnership(address)**(`_newOwner`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_newOwner` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:993

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

packages/ethereum/types/PermittableToken.d.ts:783

___

### version()

▸ **version()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/PermittableToken.d.ts:783

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
