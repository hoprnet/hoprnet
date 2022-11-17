[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprWhitehat

# Interface: HoprWhitehat

## Hierarchy

- `BaseContract`

  ↳ **`HoprWhitehat`**

## Table of contents

### Properties

- [\_deployedPromise](HoprWhitehat.md#_deployedpromise)
- [\_runningEvents](HoprWhitehat.md#_runningevents)
- [\_wrappedEmits](HoprWhitehat.md#_wrappedemits)
- [address](HoprWhitehat.md#address)
- [callStatic](HoprWhitehat.md#callstatic)
- [deployTransaction](HoprWhitehat.md#deploytransaction)
- [estimateGas](HoprWhitehat.md#estimategas)
- [filters](HoprWhitehat.md#filters)
- [functions](HoprWhitehat.md#functions)
- [interface](HoprWhitehat.md#interface)
- [off](HoprWhitehat.md#off)
- [on](HoprWhitehat.md#on)
- [once](HoprWhitehat.md#once)
- [populateTransaction](HoprWhitehat.md#populatetransaction)
- [provider](HoprWhitehat.md#provider)
- [removeListener](HoprWhitehat.md#removelistener)
- [resolvedAddress](HoprWhitehat.md#resolvedaddress)
- [signer](HoprWhitehat.md#signer)

### Methods

- [\_checkRunningEvents](HoprWhitehat.md#_checkrunningevents)
- [\_deployed](HoprWhitehat.md#_deployed)
- [\_wrapEvent](HoprWhitehat.md#_wrapevent)
- [activate](HoprWhitehat.md#activate)
- [attach](HoprWhitehat.md#attach)
- [canImplementInterfaceForAddress](HoprWhitehat.md#canimplementinterfaceforaddress)
- [connect](HoprWhitehat.md#connect)
- [currentCaller](HoprWhitehat.md#currentcaller)
- [deactivate](HoprWhitehat.md#deactivate)
- [deployed](HoprWhitehat.md#deployed)
- [emit](HoprWhitehat.md#emit)
- [fallback](HoprWhitehat.md#fallback)
- [gimmeToken](HoprWhitehat.md#gimmetoken)
- [gimmeTokenFor](HoprWhitehat.md#gimmetokenfor)
- [isActive](HoprWhitehat.md#isactive)
- [listenerCount](HoprWhitehat.md#listenercount)
- [listeners](HoprWhitehat.md#listeners)
- [myHoprBoost](HoprWhitehat.md#myhoprboost)
- [myHoprStake](HoprWhitehat.md#myhoprstake)
- [onERC721Received](HoprWhitehat.md#onerc721received)
- [onTokenTransfer](HoprWhitehat.md#ontokentransfer)
- [owner](HoprWhitehat.md#owner)
- [ownerRescueBoosterNft](HoprWhitehat.md#ownerrescueboosternft)
- [ownerRescueBoosterNftInBatch](HoprWhitehat.md#ownerrescueboosternftinbatch)
- [queryFilter](HoprWhitehat.md#queryfilter)
- [reclaimErc20Tokens](HoprWhitehat.md#reclaimerc20tokens)
- [reclaimErc721Tokens](HoprWhitehat.md#reclaimerc721tokens)
- [removeAllListeners](HoprWhitehat.md#removealllisteners)
- [renounceOwnership](HoprWhitehat.md#renounceownership)
- [rescuedXHoprAmount](HoprWhitehat.md#rescuedxhopramount)
- [tokensReceived](HoprWhitehat.md#tokensreceived)
- [transferBackOwnership](HoprWhitehat.md#transferbackownership)
- [transferOwnership](HoprWhitehat.md#transferownership)
- [wxHopr](HoprWhitehat.md#wxhopr)
- [xHopr](HoprWhitehat.md#xhopr)

## Properties

### \_deployedPromise

• **\_deployedPromise**: `Promise`<`Contract`\>

#### Inherited from

BaseContract.\_deployedPromise

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:100

___

### \_runningEvents

• **\_runningEvents**: `Object`

#### Index signature

▪ [eventTag: `string`]: `RunningEvent`

#### Inherited from

BaseContract.\_runningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:101

___

### \_wrappedEmits

• **\_wrappedEmits**: `Object`

#### Index signature

▪ [eventTag: `string`]: (...`args`: `any`[]) => `void`

#### Inherited from

BaseContract.\_wrappedEmits

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:104

___

### address

• `Readonly` **address**: `string`

#### Inherited from

BaseContract.address

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:79

___

### callStatic

• **callStatic**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `activate` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `currentCaller` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `deactivate` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `gimmeToken` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `gimmeTokenFor` | (`staker`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `isActive` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `myHoprBoost` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `myHoprStake` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `ownerRescueBoosterNft` | (`stakerAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `ownerRescueBoosterNftInBatch` | (`stakerAddress`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceOwnership` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `rescuedXHoprAmount` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `transferBackOwnership` | (`multisig`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `wxHopr` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `xHopr` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:549

___

### deployTransaction

• `Readonly` **deployTransaction**: `TransactionResponse`

#### Inherited from

BaseContract.deployTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:99

___

### estimateGas

• **estimateGas**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `activate` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `currentCaller` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `deactivate` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `gimmeToken` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `gimmeTokenFor` | (`staker`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `isActive` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `myHoprBoost` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `myHoprStake` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `ownerRescueBoosterNft` | (`stakerAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `ownerRescueBoosterNftInBatch` | (`stakerAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `rescuedXHoprAmount` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferBackOwnership` | (`multisig`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `wxHopr` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `xHopr` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:702

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Called777Hook` | (`contractAddress?`: `string`, `from?`: `string`, `amount?`: `BigNumberish`) => `Called777HookEventFilter` |
| `Called777Hook(address,address,uint256)` | (`contractAddress?`: `string`, `from?`: `string`, `amount?`: `BigNumberish`) => `Called777HookEventFilter` |
| `Called777HookForFunding` | (`contractAddress?`: `string`, `from?`: `string`, `amount?`: `BigNumberish`) => `Called777HookForFundingEventFilter` |
| `Called777HookForFunding(address,address,uint256)` | (`contractAddress?`: `string`, `from?`: `string`, `amount?`: `BigNumberish`) => `Called777HookForFundingEventFilter` |
| `OwnershipTransferred` | (`previousOwner?`: `string`, `newOwner?`: `string`) => `OwnershipTransferredEventFilter` |
| `OwnershipTransferred(address,address)` | (`previousOwner?`: `string`, `newOwner?`: `string`) => `OwnershipTransferredEventFilter` |
| `Received677` | (`contractAddress?`: `string`, `from?`: `string`, `amount?`: `BigNumberish`) => `Received677EventFilter` |
| `Received677(address,address,uint256)` | (`contractAddress?`: `string`, `from?`: `string`, `amount?`: `BigNumberish`) => `Received677EventFilter` |
| `ReclaimedBoost` | (`account?`: `string`, `tokenId?`: `BigNumberish`) => `ReclaimedBoostEventFilter` |
| `ReclaimedBoost(address,uint256)` | (`account?`: `string`, `tokenId?`: `BigNumberish`) => `ReclaimedBoostEventFilter` |
| `RequestedGimme` | (`account?`: `string`, `entitledReward?`: `BigNumberish`) => `RequestedGimmeEventFilter` |
| `RequestedGimme(address,uint256)` | (`account?`: `string`, `entitledReward?`: `BigNumberish`) => `RequestedGimmeEventFilter` |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:640

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `activate` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `currentCaller` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `deactivate` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `gimmeToken` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `gimmeTokenFor` | (`staker`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `isActive` | (`overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `myHoprBoost` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `myHoprStake` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `ownerRescueBoosterNft` | (`stakerAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `ownerRescueBoosterNftInBatch` | (`stakerAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `rescuedXHoprAmount` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferBackOwnership` | (`multisig`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `wxHopr` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `xHopr` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:347

___

### interface

• **interface**: `HoprWhitehatInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:326

___

### off

• **off**: `OnEvent`<[`HoprWhitehat`](HoprWhitehat.md)\>

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:342

___

### on

• **on**: `OnEvent`<[`HoprWhitehat`](HoprWhitehat.md)\>

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:343

___

### once

• **once**: `OnEvent`<[`HoprWhitehat`](HoprWhitehat.md)\>

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:344

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `activate` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `currentCaller` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `deactivate` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `gimmeToken` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `gimmeTokenFor` | (`staker`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `isActive` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `myHoprBoost` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `myHoprStake` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `ownerRescueBoosterNft` | (`stakerAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `ownerRescueBoosterNftInBatch` | (`stakerAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `rescuedXHoprAmount` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferBackOwnership` | (`multisig`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `wxHopr` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `xHopr` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:804

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

BaseContract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:82

___

### removeListener

• **removeListener**: `OnEvent`<[`HoprWhitehat`](HoprWhitehat.md)\>

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:345

___

### resolvedAddress

• `Readonly` **resolvedAddress**: `Promise`<`string`\>

#### Inherited from

BaseContract.resolvedAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:98

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

BaseContract.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:81

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

node_modules/@ethersproject/contracts/lib/index.d.ts:121

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

node_modules/@ethersproject/contracts/lib/index.d.ts:114

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

node_modules/@ethersproject/contracts/lib/index.d.ts:122

___

### activate

▸ **activate**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:449

___

### attach

▸ **attach**(`addressOrName`): [`HoprWhitehat`](HoprWhitehat.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprWhitehat`](HoprWhitehat.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:323

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

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:453

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprWhitehat`](HoprWhitehat.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprWhitehat`](HoprWhitehat.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:322

___

### currentCaller

▸ **currentCaller**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:459

___

### deactivate

▸ **deactivate**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:461

___

### deployed

▸ **deployed**(): `Promise`<[`HoprWhitehat`](HoprWhitehat.md)\>

#### Returns

`Promise`<[`HoprWhitehat`](HoprWhitehat.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:324

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

node_modules/@ethersproject/contracts/lib/index.d.ts:127

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

node_modules/@ethersproject/contracts/lib/index.d.ts:115

___

### gimmeToken

▸ **gimmeToken**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:465

___

### gimmeTokenFor

▸ **gimmeTokenFor**(`staker`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `staker` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:469

___

### isActive

▸ **isActive**(`overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:474

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

node_modules/@ethersproject/contracts/lib/index.d.ts:128

___

### listeners

▸ **listeners**<`TEvent`\>(`eventFilter?`): `TypedListener`<`TEvent`\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TEvent` | extends [`TypedEvent`](TypedEvent.md)<`any`, `any`, `TEvent`\> |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [`TypedEventFilter`](TypedEventFilter.md)<`TEvent`\> |

#### Returns

`TypedListener`<`TEvent`\>[]

#### Overrides

BaseContract.listeners

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:334

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

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:337

___

### myHoprBoost

▸ **myHoprBoost**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:476

___

### myHoprStake

▸ **myHoprStake**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:478

___

### onERC721Received

▸ **onERC721Received**(`operator`, `from`, `tokenId`, `data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `from` | `string` |
| `tokenId` | `BigNumberish` |
| `data` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:480

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

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:488

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

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:495

___

### ownerRescueBoosterNft

▸ **ownerRescueBoosterNft**(`stakerAddress`, `tokenId`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `stakerAddress` | `string` |
| `tokenId` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:497

___

### ownerRescueBoosterNftInBatch

▸ **ownerRescueBoosterNftInBatch**(`stakerAddress`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `stakerAddress` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:503

___

### queryFilter

▸ **queryFilter**<`TEvent`\>(`event`, `fromBlockOrBlockhash?`, `toBlock?`): `Promise`<`TEvent`[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TEvent` | extends [`TypedEvent`](TypedEvent.md)<`any`, `any`, `TEvent`\> |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [`TypedEventFilter`](TypedEventFilter.md)<`TEvent`\> |
| `fromBlockOrBlockhash?` | `string` \| `number` |
| `toBlock?` | `string` \| `number` |

#### Returns

`Promise`<`TEvent`[]\>

#### Overrides

BaseContract.queryFilter

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:328

___

### reclaimErc20Tokens

▸ **reclaimErc20Tokens**(`tokenAddress`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `tokenAddress` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:508

___

### reclaimErc721Tokens

▸ **reclaimErc721Tokens**(`tokenAddress`, `tokenId`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `tokenAddress` | `string` |
| `tokenId` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:513

___

### removeAllListeners

▸ **removeAllListeners**<`TEvent`\>(`eventFilter`): [`HoprWhitehat`](HoprWhitehat.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TEvent` | extends [`TypedEvent`](TypedEvent.md)<`any`, `any`, `TEvent`\> |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](TypedEventFilter.md)<`TEvent`\> |

#### Returns

[`HoprWhitehat`](HoprWhitehat.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:338

▸ **removeAllListeners**(`eventName?`): [`HoprWhitehat`](HoprWhitehat.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprWhitehat`](HoprWhitehat.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:341

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

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:519

___

### rescuedXHoprAmount

▸ **rescuedXHoprAmount**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:523

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

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:525

___

### transferBackOwnership

▸ **transferBackOwnership**(`multisig`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multisig` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:535

___

### transferOwnership

▸ **transferOwnership**(`newOwner`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newOwner` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:540

___

### wxHopr

▸ **wxHopr**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:545

___

### xHopr

▸ **xHopr**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprWhitehat.ts:547
