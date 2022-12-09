[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprStake

# Interface: HoprStake

## Hierarchy

- `BaseContract`

  ↳ **`HoprStake`**

## Table of contents

### Properties

- [\_deployedPromise](HoprStake.md#_deployedpromise)
- [\_runningEvents](HoprStake.md#_runningevents)
- [\_wrappedEmits](HoprStake.md#_wrappedemits)
- [address](HoprStake.md#address)
- [callStatic](HoprStake.md#callstatic)
- [deployTransaction](HoprStake.md#deploytransaction)
- [estimateGas](HoprStake.md#estimategas)
- [filters](HoprStake.md#filters)
- [functions](HoprStake.md#functions)
- [interface](HoprStake.md#interface)
- [off](HoprStake.md#off)
- [on](HoprStake.md#on)
- [once](HoprStake.md#once)
- [populateTransaction](HoprStake.md#populatetransaction)
- [provider](HoprStake.md#provider)
- [removeListener](HoprStake.md#removelistener)
- [resolvedAddress](HoprStake.md#resolvedaddress)
- [signer](HoprStake.md#signer)

### Methods

- [BASIC\_FACTOR\_NUMERATOR](HoprStake.md#basic_factor_numerator)
- [BASIC\_START](HoprStake.md#basic_start)
- [BOOST\_CAP](HoprStake.md#boost_cap)
- [FACTOR\_DENOMINATOR](HoprStake.md#factor_denominator)
- [LOCK\_TOKEN](HoprStake.md#lock_token)
- [PROGRAM\_END](HoprStake.md#program_end)
- [REWARD\_TOKEN](HoprStake.md#reward_token)
- [SEED\_FACTOR\_NUMERATOR](HoprStake.md#seed_factor_numerator)
- [SEED\_START](HoprStake.md#seed_start)
- [\_checkRunningEvents](HoprStake.md#_checkrunningevents)
- [\_deployed](HoprStake.md#_deployed)
- [\_wrapEvent](HoprStake.md#_wrapevent)
- [accounts](HoprStake.md#accounts)
- [attach](HoprStake.md#attach)
- [availableReward](HoprStake.md#availablereward)
- [claimRewards](HoprStake.md#claimrewards)
- [connect](HoprStake.md#connect)
- [deployed](HoprStake.md#deployed)
- [emit](HoprStake.md#emit)
- [fallback](HoprStake.md#fallback)
- [getCumulatedRewardsIncrement](HoprStake.md#getcumulatedrewardsincrement)
- [listenerCount](HoprStake.md#listenercount)
- [listeners](HoprStake.md#listeners)
- [lock](HoprStake.md#lock)
- [nftContract](HoprStake.md#nftcontract)
- [onERC721Received](HoprStake.md#onerc721received)
- [onTokenTransfer](HoprStake.md#ontokentransfer)
- [owner](HoprStake.md#owner)
- [queryFilter](HoprStake.md#queryfilter)
- [reclaimErc20Tokens](HoprStake.md#reclaimerc20tokens)
- [reclaimErc721Tokens](HoprStake.md#reclaimerc721tokens)
- [redeemedFactor](HoprStake.md#redeemedfactor)
- [redeemedFactorIndex](HoprStake.md#redeemedfactorindex)
- [redeemedNft](HoprStake.md#redeemednft)
- [redeemedNftIndex](HoprStake.md#redeemednftindex)
- [removeAllListeners](HoprStake.md#removealllisteners)
- [renounceOwnership](HoprStake.md#renounceownership)
- [sync](HoprStake.md#sync)
- [tokensReceived](HoprStake.md#tokensreceived)
- [totalLocked](HoprStake.md#totallocked)
- [transferOwnership](HoprStake.md#transferownership)
- [unlock](HoprStake.md#unlock)

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
| `BASIC_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `BASIC_START` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `BOOST_CAP` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `FACTOR_DENOMINATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `LOCK_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `PROGRAM_END` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `REWARD_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `SEED_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `SEED_START` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `accounts` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`] & { `actualLockedTokenAmount`: `BigNumber` ; `claimedRewards`: `BigNumber` ; `cumulatedRewards`: `BigNumber` ; `lastSyncTimestamp`: `BigNumber` ; `virtualLockedTokenAmount`: `BigNumber`  }\> |
| `availableReward` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `claimRewards` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `getCumulatedRewardsIncrement` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `lock` | (`investors`: `string`[], `caps`: `BigNumberish`[], `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `nftContract` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `redeemedFactor` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedFactorIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedNft` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedNftIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `sync` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `totalLocked` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `unlock` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:682

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
| `BASIC_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `BASIC_START` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `BOOST_CAP` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `FACTOR_DENOMINATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `LOCK_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `PROGRAM_END` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `REWARD_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `SEED_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `SEED_START` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `accounts` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `availableReward` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `claimRewards` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `getCumulatedRewardsIncrement` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `lock` | (`investors`: `string`[], `caps`: `BigNumberish`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `nftContract` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `redeemedFactor` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedFactorIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedNft` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedNftIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `sync` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `totalLocked` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `unlock` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:872

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Claimed` | (`account?`: `string`, `rewardAmount?`: `BigNumberish`) => `ClaimedEventFilter` |
| `Claimed(address,uint256)` | (`account?`: `string`, `rewardAmount?`: `BigNumberish`) => `ClaimedEventFilter` |
| `OwnershipTransferred` | (`previousOwner?`: `string`, `newOwner?`: `string`) => `OwnershipTransferredEventFilter` |
| `OwnershipTransferred(address,address)` | (`previousOwner?`: `string`, `newOwner?`: `string`) => `OwnershipTransferredEventFilter` |
| `Redeemed` | (`account?`: `string`, `boostTokenId?`: `BigNumberish`, `factorRegistered?`: `boolean`) => `RedeemedEventFilter` |
| `Redeemed(address,uint256,bool)` | (`account?`: `string`, `boostTokenId?`: `BigNumberish`, `factorRegistered?`: `boolean`) => `RedeemedEventFilter` |
| `Released` | (`account?`: `string`, `actualAmount?`: `BigNumberish`, `virtualAmount?`: `BigNumberish`) => `ReleasedEventFilter` |
| `Released(address,uint256,uint256)` | (`account?`: `string`, `actualAmount?`: `BigNumberish`, `virtualAmount?`: `BigNumberish`) => `ReleasedEventFilter` |
| `RewardFueled` | (`amount?`: `BigNumberish`) => `RewardFueledEventFilter` |
| `RewardFueled(uint256)` | (`amount?`: `BigNumberish`) => `RewardFueledEventFilter` |
| `Staked` | (`account?`: `string`, `actualAmount?`: `BigNumberish`, `virtualAmount?`: `BigNumberish`) => `StakedEventFilter` |
| `Staked(address,uint256,uint256)` | (`account?`: `string`, `actualAmount?`: `BigNumberish`, `virtualAmount?`: `BigNumberish`) => `StakedEventFilter` |
| `Sync` | (`account?`: `string`, `increment?`: `BigNumberish`) => `SyncEventFilter` |
| `Sync(address,uint256)` | (`account?`: `string`, `increment?`: `BigNumberish`) => `SyncEventFilter` |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:805

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `BASIC_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `BASIC_START` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `BOOST_CAP` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `FACTOR_DENOMINATOR` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `LOCK_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `PROGRAM_END` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `REWARD_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `SEED_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `SEED_START` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `accounts` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`] & { `actualLockedTokenAmount`: `BigNumber` ; `claimedRewards`: `BigNumber` ; `cumulatedRewards`: `BigNumber` ; `lastSyncTimestamp`: `BigNumber` ; `virtualLockedTokenAmount`: `BigNumber`  }\> |
| `availableReward` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `claimRewards` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `getCumulatedRewardsIncrement` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `lock` | (`investors`: `string`[], `caps`: `BigNumberish`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `nftContract` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `redeemedFactor` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `redeemedFactorIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `redeemedNft` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `redeemedNftIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `sync` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `totalLocked` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `unlock` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:419

___

### interface

• **interface**: `HoprStakeInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:398

___

### off

• **off**: `OnEvent`<[`HoprStake`](HoprStake.md)\>

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:414

___

### on

• **on**: `OnEvent`<[`HoprStake`](HoprStake.md)\>

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:415

___

### once

• **once**: `OnEvent`<[`HoprStake`](HoprStake.md)\>

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:416

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `BASIC_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `BASIC_START` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `BOOST_CAP` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `FACTOR_DENOMINATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `LOCK_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `PROGRAM_END` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `REWARD_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `SEED_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `SEED_START` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `accounts` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `availableReward` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `claimRewards` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `getCumulatedRewardsIncrement` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `lock` | (`investors`: `string`[], `caps`: `BigNumberish`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `nftContract` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `redeemedFactor` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `redeemedFactorIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `redeemedNft` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `redeemedNftIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `sync` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `totalLocked` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `unlock` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:995

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

BaseContract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:82

___

### removeListener

• **removeListener**: `OnEvent`<[`HoprStake`](HoprStake.md)\>

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:417

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

### BASIC\_FACTOR\_NUMERATOR

▸ **BASIC_FACTOR_NUMERATOR**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:553

___

### BASIC\_START

▸ **BASIC_START**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:555

___

### BOOST\_CAP

▸ **BOOST_CAP**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:557

___

### FACTOR\_DENOMINATOR

▸ **FACTOR_DENOMINATOR**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:559

___

### LOCK\_TOKEN

▸ **LOCK_TOKEN**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:561

___

### PROGRAM\_END

▸ **PROGRAM_END**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:563

___

### REWARD\_TOKEN

▸ **REWARD_TOKEN**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:565

___

### SEED\_FACTOR\_NUMERATOR

▸ **SEED_FACTOR_NUMERATOR**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:567

___

### SEED\_START

▸ **SEED_START**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:569

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

### accounts

▸ **accounts**(`arg0`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`] & { `actualLockedTokenAmount`: `BigNumber` ; `claimedRewards`: `BigNumber` ; `cumulatedRewards`: `BigNumber` ; `lastSyncTimestamp`: `BigNumber` ; `virtualLockedTokenAmount`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`] & { `actualLockedTokenAmount`: `BigNumber` ; `claimedRewards`: `BigNumber` ; `cumulatedRewards`: `BigNumber` ; `lastSyncTimestamp`: `BigNumber` ; `virtualLockedTokenAmount`: `BigNumber`  }\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:571

___

### attach

▸ **attach**(`addressOrName`): [`HoprStake`](HoprStake.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprStake`](HoprStake.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:395

___

### availableReward

▸ **availableReward**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:584

___

### claimRewards

▸ **claimRewards**(`account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:586

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprStake`](HoprStake.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprStake`](HoprStake.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:394

___

### deployed

▸ **deployed**(): `Promise`<[`HoprStake`](HoprStake.md)\>

#### Returns

`Promise`<[`HoprStake`](HoprStake.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:396

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

### getCumulatedRewardsIncrement

▸ **getCumulatedRewardsIncrement**(`_account`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:591

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:406

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:409

___

### lock

▸ **lock**(`investors`, `caps`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `investors` | `string`[] |
| `caps` | `BigNumberish`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:596

___

### nftContract

▸ **nftContract**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:602

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:604

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:612

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:619

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:400

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:621

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:626

___

### redeemedFactor

▸ **redeemedFactor**(`arg0`, `arg1`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `arg1` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:632

___

### redeemedFactorIndex

▸ **redeemedFactorIndex**(`arg0`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:638

___

### redeemedNft

▸ **redeemedNft**(`arg0`, `arg1`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `arg1` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:643

___

### redeemedNftIndex

▸ **redeemedNftIndex**(`arg0`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:649

___

### removeAllListeners

▸ **removeAllListeners**<`TEvent`\>(`eventFilter`): [`HoprStake`](HoprStake.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TEvent` | extends [`TypedEvent`](TypedEvent.md)<`any`, `any`, `TEvent`\> |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](TypedEventFilter.md)<`TEvent`\> |

#### Returns

[`HoprStake`](HoprStake.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:410

▸ **removeAllListeners**(`eventName?`): [`HoprStake`](HoprStake.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprStake`](HoprStake.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:413

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:651

___

### sync

▸ **sync**(`account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:655

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:660

___

### totalLocked

▸ **totalLocked**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:670

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

packages/ethereum/src/types/contracts/stake/HoprStake.ts:672

___

### unlock

▸ **unlock**(`account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/contracts/stake/HoprStake.ts:677
