[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC677BridgeToken

# Class: ERC677BridgeToken

## Hierarchy

- `Contract`

  ↳ **`ERC677BridgeToken`**

## Table of contents

### Constructors

- [constructor](erc677bridgetoken.md#constructor)

### Properties

- [\_deployedPromise](erc677bridgetoken.md#_deployedpromise)
- [\_runningEvents](erc677bridgetoken.md#_runningevents)
- [\_wrappedEmits](erc677bridgetoken.md#_wrappedemits)
- [address](erc677bridgetoken.md#address)
- [callStatic](erc677bridgetoken.md#callstatic)
- [deployTransaction](erc677bridgetoken.md#deploytransaction)
- [estimateGas](erc677bridgetoken.md#estimategas)
- [filters](erc677bridgetoken.md#filters)
- [functions](erc677bridgetoken.md#functions)
- [interface](erc677bridgetoken.md#interface)
- [populateTransaction](erc677bridgetoken.md#populatetransaction)
- [provider](erc677bridgetoken.md#provider)
- [resolvedAddress](erc677bridgetoken.md#resolvedaddress)
- [signer](erc677bridgetoken.md#signer)

### Methods

- [\_checkRunningEvents](erc677bridgetoken.md#_checkrunningevents)
- [\_deployed](erc677bridgetoken.md#_deployed)
- [\_wrapEvent](erc677bridgetoken.md#_wrapevent)
- [allowance](erc677bridgetoken.md#allowance)
- [allowance(address,address)](erc677bridgetoken.md#allowance(address,address))
- [approve](erc677bridgetoken.md#approve)
- [approve(address,uint256)](erc677bridgetoken.md#approve(address,uint256))
- [attach](erc677bridgetoken.md#attach)
- [balanceOf](erc677bridgetoken.md#balanceof)
- [balanceOf(address)](erc677bridgetoken.md#balanceof(address))
- [bridgeContract](erc677bridgetoken.md#bridgecontract)
- [bridgeContract()](erc677bridgetoken.md#bridgecontract())
- [burn](erc677bridgetoken.md#burn)
- [burn(uint256)](erc677bridgetoken.md#burn(uint256))
- [claimTokens](erc677bridgetoken.md#claimtokens)
- [claimTokens(address,address)](erc677bridgetoken.md#claimtokens(address,address))
- [connect](erc677bridgetoken.md#connect)
- [decimals](erc677bridgetoken.md#decimals)
- [decimals()](erc677bridgetoken.md#decimals())
- [decreaseAllowance](erc677bridgetoken.md#decreaseallowance)
- [decreaseAllowance(address,uint256)](erc677bridgetoken.md#decreaseallowance(address,uint256))
- [decreaseApproval](erc677bridgetoken.md#decreaseapproval)
- [decreaseApproval(address,uint256)](erc677bridgetoken.md#decreaseapproval(address,uint256))
- [deployed](erc677bridgetoken.md#deployed)
- [emit](erc677bridgetoken.md#emit)
- [fallback](erc677bridgetoken.md#fallback)
- [finishMinting](erc677bridgetoken.md#finishminting)
- [finishMinting()](erc677bridgetoken.md#finishminting())
- [getTokenInterfacesVersion](erc677bridgetoken.md#gettokeninterfacesversion)
- [getTokenInterfacesVersion()](erc677bridgetoken.md#gettokeninterfacesversion())
- [increaseAllowance](erc677bridgetoken.md#increaseallowance)
- [increaseAllowance(address,uint256)](erc677bridgetoken.md#increaseallowance(address,uint256))
- [increaseApproval](erc677bridgetoken.md#increaseapproval)
- [increaseApproval(address,uint256)](erc677bridgetoken.md#increaseapproval(address,uint256))
- [isBridge](erc677bridgetoken.md#isbridge)
- [isBridge(address)](erc677bridgetoken.md#isbridge(address))
- [listenerCount](erc677bridgetoken.md#listenercount)
- [listeners](erc677bridgetoken.md#listeners)
- [mint](erc677bridgetoken.md#mint)
- [mint(address,uint256)](erc677bridgetoken.md#mint(address,uint256))
- [mintingFinished](erc677bridgetoken.md#mintingfinished)
- [mintingFinished()](erc677bridgetoken.md#mintingfinished())
- [name](erc677bridgetoken.md#name)
- [name()](erc677bridgetoken.md#name())
- [off](erc677bridgetoken.md#off)
- [on](erc677bridgetoken.md#on)
- [once](erc677bridgetoken.md#once)
- [owner](erc677bridgetoken.md#owner)
- [owner()](erc677bridgetoken.md#owner())
- [queryFilter](erc677bridgetoken.md#queryfilter)
- [removeAllListeners](erc677bridgetoken.md#removealllisteners)
- [removeListener](erc677bridgetoken.md#removelistener)
- [renounceOwnership](erc677bridgetoken.md#renounceownership)
- [renounceOwnership()](erc677bridgetoken.md#renounceownership())
- [setBridgeContract](erc677bridgetoken.md#setbridgecontract)
- [setBridgeContract(address)](erc677bridgetoken.md#setbridgecontract(address))
- [symbol](erc677bridgetoken.md#symbol)
- [symbol()](erc677bridgetoken.md#symbol())
- [totalSupply](erc677bridgetoken.md#totalsupply)
- [totalSupply()](erc677bridgetoken.md#totalsupply())
- [transfer](erc677bridgetoken.md#transfer)
- [transfer(address,uint256)](erc677bridgetoken.md#transfer(address,uint256))
- [transferAndCall](erc677bridgetoken.md#transferandcall)
- [transferAndCall(address,uint256,bytes)](erc677bridgetoken.md#transferandcall(address,uint256,bytes))
- [transferFrom](erc677bridgetoken.md#transferfrom)
- [transferFrom(address,address,uint256)](erc677bridgetoken.md#transferfrom(address,address,uint256))
- [transferOwnership](erc677bridgetoken.md#transferownership)
- [transferOwnership(address)](erc677bridgetoken.md#transferownership(address))
- [getContractAddress](erc677bridgetoken.md#getcontractaddress)
- [getInterface](erc677bridgetoken.md#getinterface)
- [isIndexed](erc677bridgetoken.md#isindexed)

## Constructors

### constructor

• **new ERC677BridgeToken**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance(address,address)` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `approve(address,uint256)` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
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
| `finishMinting` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `finishMinting()` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `getTokenInterfacesVersion()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `increaseAllowance` | (`spender`: `string`, `addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `increaseAllowance(address,uint256)` | (`spender`: `string`, `addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `increaseApproval(address,uint256)` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isBridge(address)` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mint(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mintingFinished()` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
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
| `transferFrom` | (`_from`: `string`, `_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom(address,address,uint256)` | (`_from`: `string`, `_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `transferOwnership(address)` | (`_newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:760

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
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance(address,address)` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `approve(address,uint256)` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
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
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `finishMinting()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTokenInterfacesVersion()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `increaseAllowance` | (`spender`: `string`, `addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `increaseAllowance(address,uint256)` | (`spender`: `string`, `addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `increaseApproval(address,uint256)` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isBridge(address)` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `mint(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `mintingFinished()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
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
| `transferFrom` | (`_from`: `string`, `_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom(address,address,uint256)` | (`_from`: `string`, `_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferOwnership(address)` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:1043

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

packages/ethereum/types/ERC677BridgeToken.d.ts:995

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `allowance(address,address)` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `approve` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `approve(address,uint256)` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
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
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `finishMinting()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `getTokenInterfacesVersion()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`] & { `major`: `BigNumber` ; `minor`: `BigNumber` ; `patch`: `BigNumber`  }\> |
| `increaseAllowance` | (`spender`: `string`, `addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `increaseAllowance(address,uint256)` | (`spender`: `string`, `addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `increaseApproval(address,uint256)` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isBridge(address)` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `mint(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `mintingFinished()` | (`overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
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
| `transferFrom` | (`_from`: `string`, `_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom(address,address,uint256)` | (`_from`: `string`, `_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferOwnership(address)` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:270

___

### interface

• **interface**: `ERC677BridgeTokenInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:268

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowance` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allowance(address,address)` | (`_owner`: `string`, `_spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `approve` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `approve(address,uint256)` | (`_spender`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
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
| `finishMinting` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `finishMinting()` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `getTokenInterfacesVersion` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getTokenInterfacesVersion()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `increaseAllowance` | (`spender`: `string`, `addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `increaseAllowance(address,uint256)` | (`spender`: `string`, `addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `increaseApproval` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `increaseApproval(address,uint256)` | (`_spender`: `string`, `_addedValue`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `isBridge` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isBridge(address)` | (`_address`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `mint` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `mint(address,uint256)` | (`_to`: `string`, `_amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `mintingFinished` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `mintingFinished()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `owner()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
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
| `transferFrom` | (`_from`: `string`, `_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom(address,address,uint256)` | (`_from`: `string`, `_to`: `string`, `_value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership(address)` | (`_newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:1275

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

packages/ethereum/types/ERC677BridgeToken.d.ts:738

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

packages/ethereum/types/ERC677BridgeToken.d.ts:742

___

### approve

▸ **approve**(`_spender`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:524

___

### approve(address,uint256)

▸ **approve(address,uint256)**(`_spender`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_spender` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:528

___

### attach

▸ **attach**(`addressOrName`): [`ERC677BridgeToken`](erc677bridgetoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:229

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

packages/ethereum/types/ERC677BridgeToken.d.ts:640

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

packages/ethereum/types/ERC677BridgeToken.d.ts:640

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

packages/ethereum/types/ERC677BridgeToken.d.ts:722

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

packages/ethereum/types/ERC677BridgeToken.d.ts:722

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

packages/ethereum/types/ERC677BridgeToken.d.ts:606

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

packages/ethereum/types/ERC677BridgeToken.d.ts:609

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

packages/ethereum/types/ERC677BridgeToken.d.ts:628

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

packages/ethereum/types/ERC677BridgeToken.d.ts:632

___

### connect

▸ **connect**(`signerOrProvider`): [`ERC677BridgeToken`](erc677bridgetoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:228

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

packages/ethereum/types/ERC677BridgeToken.d.ts:564

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

packages/ethereum/types/ERC677BridgeToken.d.ts:564

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

packages/ethereum/types/ERC677BridgeToken.d.ts:698

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

packages/ethereum/types/ERC677BridgeToken.d.ts:702

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

packages/ethereum/types/ERC677BridgeToken.d.ts:616

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

packages/ethereum/types/ERC677BridgeToken.d.ts:620

___

### deployed

▸ **deployed**(): `Promise`<[`ERC677BridgeToken`](erc677bridgetoken.md)\>

#### Returns

`Promise`<[`ERC677BridgeToken`](erc677bridgetoken.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:230

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

### finishMinting

▸ **finishMinting**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:662

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

packages/ethereum/types/ERC677BridgeToken.d.ts:664

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

packages/ethereum/types/ERC677BridgeToken.d.ts:670

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

packages/ethereum/types/ERC677BridgeToken.d.ts:678

___

### increaseAllowance

▸ **increaseAllowance**(`spender`, `addedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | `string` |
| `addedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:568

___

### increaseAllowance(address,uint256)

▸ **increaseAllowance(address,uint256)**(`spender`, `addedValue`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | `string` |
| `addedValue` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:572

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

packages/ethereum/types/ERC677BridgeToken.d.ts:726

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

packages/ethereum/types/ERC677BridgeToken.d.ts:730

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

packages/ethereum/types/ERC677BridgeToken.d.ts:655

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

packages/ethereum/types/ERC677BridgeToken.d.ts:655

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

packages/ethereum/types/ERC677BridgeToken.d.ts:232

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

packages/ethereum/types/ERC677BridgeToken.d.ts:255

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

packages/ethereum/types/ERC677BridgeToken.d.ts:594

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

packages/ethereum/types/ERC677BridgeToken.d.ts:598

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

packages/ethereum/types/ERC677BridgeToken.d.ts:516

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

packages/ethereum/types/ERC677BridgeToken.d.ts:516

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

packages/ethereum/types/ERC677BridgeToken.d.ts:520

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

packages/ethereum/types/ERC677BridgeToken.d.ts:520

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC677BridgeToken`](erc677bridgetoken.md)

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

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:235

▸ **off**(`eventName`, `listener`): [`ERC677BridgeToken`](erc677bridgetoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:256

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC677BridgeToken`](erc677bridgetoken.md)

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

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:239

▸ **on**(`eventName`, `listener`): [`ERC677BridgeToken`](erc677bridgetoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:257

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC677BridgeToken`](erc677bridgetoken.md)

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

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:243

▸ **once**(`eventName`, `listener`): [`ERC677BridgeToken`](erc677bridgetoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:258

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

packages/ethereum/types/ERC677BridgeToken.d.ts:690

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

packages/ethereum/types/ERC677BridgeToken.d.ts:690

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

packages/ethereum/types/ERC677BridgeToken.d.ts:262

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`ERC677BridgeToken`](erc677bridgetoken.md)

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

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:251

▸ **removeAllListeners**(`eventName?`): [`ERC677BridgeToken`](erc677bridgetoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:260

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC677BridgeToken`](erc677bridgetoken.md)

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

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:247

▸ **removeListener**(`eventName`, `listener`): [`ERC677BridgeToken`](erc677bridgetoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC677BridgeToken`](erc677bridgetoken.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:259

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

packages/ethereum/types/ERC677BridgeToken.d.ts:647

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

packages/ethereum/types/ERC677BridgeToken.d.ts:649

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

packages/ethereum/types/ERC677BridgeToken.d.ts:536

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

packages/ethereum/types/ERC677BridgeToken.d.ts:539

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

packages/ethereum/types/ERC677BridgeToken.d.ts:694

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

packages/ethereum/types/ERC677BridgeToken.d.ts:694

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

packages/ethereum/types/ERC677BridgeToken.d.ts:546

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

packages/ethereum/types/ERC677BridgeToken.d.ts:546

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

packages/ethereum/types/ERC677BridgeToken.d.ts:710

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

packages/ethereum/types/ERC677BridgeToken.d.ts:714

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

packages/ethereum/types/ERC677BridgeToken.d.ts:580

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

packages/ethereum/types/ERC677BridgeToken.d.ts:585

___

### transferFrom

▸ **transferFrom**(`_from`, `_to`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_from` | `string` |
| `_to` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:550

___

### transferFrom(address,address,uint256)

▸ **transferFrom(address,address,uint256)**(`_from`, `_to`, `_value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_from` | `string` |
| `_to` | `string` |
| `_value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC677BridgeToken.d.ts:555

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

packages/ethereum/types/ERC677BridgeToken.d.ts:750

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

packages/ethereum/types/ERC677BridgeToken.d.ts:753

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
