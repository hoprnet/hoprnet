[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprStakeSeason4

# Interface: HoprStakeSeason4

## Hierarchy

- `BaseContract`

  ↳ **`HoprStakeSeason4`**

## Table of contents

### Properties

- [\_deployedPromise](HoprStakeSeason4.md#_deployedpromise)
- [\_runningEvents](HoprStakeSeason4.md#_runningevents)
- [\_wrappedEmits](HoprStakeSeason4.md#_wrappedemits)
- [address](HoprStakeSeason4.md#address)
- [callStatic](HoprStakeSeason4.md#callstatic)
- [contractName](HoprStakeSeason4.md#contractname)
- [deployTransaction](HoprStakeSeason4.md#deploytransaction)
- [estimateGas](HoprStakeSeason4.md#estimategas)
- [filters](HoprStakeSeason4.md#filters)
- [functions](HoprStakeSeason4.md#functions)
- [interface](HoprStakeSeason4.md#interface)
- [off](HoprStakeSeason4.md#off)
- [on](HoprStakeSeason4.md#on)
- [once](HoprStakeSeason4.md#once)
- [populateTransaction](HoprStakeSeason4.md#populatetransaction)
- [provider](HoprStakeSeason4.md#provider)
- [removeListener](HoprStakeSeason4.md#removelistener)
- [resolvedAddress](HoprStakeSeason4.md#resolvedaddress)
- [signer](HoprStakeSeason4.md#signer)

### Methods

- [BASIC\_FACTOR\_NUMERATOR](HoprStakeSeason4.md#basic_factor_numerator)
- [BOOST\_CAP](HoprStakeSeason4.md#boost_cap)
- [FACTOR\_DENOMINATOR](HoprStakeSeason4.md#factor_denominator)
- [LOCK\_TOKEN](HoprStakeSeason4.md#lock_token)
- [NFT\_CONTRACT](HoprStakeSeason4.md#nft_contract)
- [PROGRAM\_END](HoprStakeSeason4.md#program_end)
- [PROGRAM\_START](HoprStakeSeason4.md#program_start)
- [REWARD\_TOKEN](HoprStakeSeason4.md#reward_token)
- [\_checkRunningEvents](HoprStakeSeason4.md#_checkrunningevents)
- [\_deployed](HoprStakeSeason4.md#_deployed)
- [\_wrapEvent](HoprStakeSeason4.md#_wrapevent)
- [accounts](HoprStakeSeason4.md#accounts)
- [attach](HoprStakeSeason4.md#attach)
- [availableReward](HoprStakeSeason4.md#availablereward)
- [claimRewards](HoprStakeSeason4.md#claimrewards)
- [connect](HoprStakeSeason4.md#connect)
- [deployed](HoprStakeSeason4.md#deployed)
- [emit](HoprStakeSeason4.md#emit)
- [fallback](HoprStakeSeason4.md#fallback)
- [getCumulatedRewardsIncrement](HoprStakeSeason4.md#getcumulatedrewardsincrement)
- [isBlockedNft](HoprStakeSeason4.md#isblockednft)
- [isNftTypeAndRankRedeemed1](HoprStakeSeason4.md#isnfttypeandrankredeemed1)
- [isNftTypeAndRankRedeemed2](HoprStakeSeason4.md#isnfttypeandrankredeemed2)
- [isNftTypeAndRankRedeemed3](HoprStakeSeason4.md#isnfttypeandrankredeemed3)
- [isNftTypeAndRankRedeemed4](HoprStakeSeason4.md#isnfttypeandrankredeemed4)
- [listenerCount](HoprStakeSeason4.md#listenercount)
- [listeners](HoprStakeSeason4.md#listeners)
- [onERC721Received](HoprStakeSeason4.md#onerc721received)
- [onTokenTransfer](HoprStakeSeason4.md#ontokentransfer)
- [owner](HoprStakeSeason4.md#owner)
- [ownerBlockNftType](HoprStakeSeason4.md#ownerblocknfttype)
- [ownerUnblockNftType](HoprStakeSeason4.md#ownerunblocknfttype)
- [queryFilter](HoprStakeSeason4.md#queryfilter)
- [reclaimErc20Tokens](HoprStakeSeason4.md#reclaimerc20tokens)
- [reclaimErc721Tokens](HoprStakeSeason4.md#reclaimerc721tokens)
- [redeemedFactor](HoprStakeSeason4.md#redeemedfactor)
- [redeemedFactorIndex](HoprStakeSeason4.md#redeemedfactorindex)
- [redeemedNft](HoprStakeSeason4.md#redeemednft)
- [redeemedNftIndex](HoprStakeSeason4.md#redeemednftindex)
- [removeAllListeners](HoprStakeSeason4.md#removealllisteners)
- [renounceOwnership](HoprStakeSeason4.md#renounceownership)
- [stakedHoprTokens](HoprStakeSeason4.md#stakedhoprtokens)
- [sync](HoprStakeSeason4.md#sync)
- [tokensReceived](HoprStakeSeason4.md#tokensreceived)
- [totalLocked](HoprStakeSeason4.md#totallocked)
- [transferOwnership](HoprStakeSeason4.md#transferownership)
- [unlock](HoprStakeSeason4.md#unlock)
- [unlockFor](HoprStakeSeason4.md#unlockfor)

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
| `BOOST_CAP` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `FACTOR_DENOMINATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `LOCK_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `NFT_CONTRACT` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `PROGRAM_END` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `PROGRAM_START` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `REWARD_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `accounts` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`] & { `actualLockedTokenAmount`: `BigNumber` ; `claimedRewards`: `BigNumber` ; `cumulatedRewards`: `BigNumber` ; `lastSyncTimestamp`: `BigNumber`  }\> |
| `availableReward` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `claimRewards` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `getCumulatedRewardsIncrement` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isBlockedNft` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isNftTypeAndRankRedeemed1` | (`nftType`: `string`, `nftRank`: `string`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isNftTypeAndRankRedeemed2` | (`nftTypeIndex`: `BigNumberish`, `nftRank`: `string`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isNftTypeAndRankRedeemed3` | (`nftTypeIndex`: `BigNumberish`, `boostNumerator`: `BigNumberish`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isNftTypeAndRankRedeemed4` | (`nftType`: `string`, `boostNumerator`: `BigNumberish`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `ownerBlockNftType` | (`typeIndex`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `ownerUnblockNftType` | (`typeIndex`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `redeemedFactor` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedFactorIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedNft` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedNftIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `stakedHoprTokens` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `sync` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `totalLocked` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `unlock` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `unlockFor` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:764

___

### contractName

• **contractName**: ``"HoprStakeSeason4"``

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:396

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
| `BOOST_CAP` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `FACTOR_DENOMINATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `LOCK_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `NFT_CONTRACT` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `PROGRAM_END` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `PROGRAM_START` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `REWARD_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `accounts` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `availableReward` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `claimRewards` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `getCumulatedRewardsIncrement` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isBlockedNft` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isNftTypeAndRankRedeemed1` | (`nftType`: `string`, `nftRank`: `string`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isNftTypeAndRankRedeemed2` | (`nftTypeIndex`: `BigNumberish`, `nftRank`: `string`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isNftTypeAndRankRedeemed3` | (`nftTypeIndex`: `BigNumberish`, `boostNumerator`: `BigNumberish`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isNftTypeAndRankRedeemed4` | (`nftType`: `string`, `boostNumerator`: `BigNumberish`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `ownerBlockNftType` | (`typeIndex`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `ownerUnblockNftType` | (`typeIndex`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `redeemedFactor` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedFactorIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedNft` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `redeemedNftIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `stakedHoprTokens` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `sync` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `totalLocked` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `unlock` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `unlockFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:999

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Claimed` | (`account?`: `string`, `rewardAmount?`: `BigNumberish`) => `ClaimedEventFilter` |
| `Claimed(address,uint256)` | (`account?`: `string`, `rewardAmount?`: `BigNumberish`) => `ClaimedEventFilter` |
| `NftAllowed` | (`typeIndex?`: `BigNumberish`) => `NftAllowedEventFilter` |
| `NftAllowed(uint256)` | (`typeIndex?`: `BigNumberish`) => `NftAllowedEventFilter` |
| `NftBlocked` | (`typeIndex?`: `BigNumberish`) => `NftBlockedEventFilter` |
| `NftBlocked(uint256)` | (`typeIndex?`: `BigNumberish`) => `NftBlockedEventFilter` |
| `OwnershipTransferred` | (`previousOwner?`: `string`, `newOwner?`: `string`) => `OwnershipTransferredEventFilter` |
| `OwnershipTransferred(address,address)` | (`previousOwner?`: `string`, `newOwner?`: `string`) => `OwnershipTransferredEventFilter` |
| `Redeemed` | (`account?`: `string`, `boostTokenId?`: `BigNumberish`, `factorRegistered?`: `boolean`) => `RedeemedEventFilter` |
| `Redeemed(address,uint256,bool)` | (`account?`: `string`, `boostTokenId?`: `BigNumberish`, `factorRegistered?`: `boolean`) => `RedeemedEventFilter` |
| `Released` | (`account?`: `string`, `actualAmount?`: `BigNumberish`) => `ReleasedEventFilter` |
| `Released(address,uint256)` | (`account?`: `string`, `actualAmount?`: `BigNumberish`) => `ReleasedEventFilter` |
| `RewardFueled` | (`amount?`: `BigNumberish`) => `RewardFueledEventFilter` |
| `RewardFueled(uint256)` | (`amount?`: `BigNumberish`) => `RewardFueledEventFilter` |
| `Staked` | (`account?`: `string`, `actualAmount?`: `BigNumberish`) => `StakedEventFilter` |
| `Staked(address,uint256)` | (`account?`: `string`, `actualAmount?`: `BigNumberish`) => `StakedEventFilter` |
| `Sync` | (`account?`: `string`, `increment?`: `BigNumberish`) => `SyncEventFilter` |
| `Sync(address,uint256)` | (`account?`: `string`, `increment?`: `BigNumberish`) => `SyncEventFilter` |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:926

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `BASIC_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `BOOST_CAP` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `FACTOR_DENOMINATOR` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `LOCK_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `NFT_CONTRACT` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `PROGRAM_END` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `PROGRAM_START` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `REWARD_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `accounts` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`] & { `actualLockedTokenAmount`: `BigNumber` ; `claimedRewards`: `BigNumber` ; `cumulatedRewards`: `BigNumber` ; `lastSyncTimestamp`: `BigNumber`  }\> |
| `availableReward` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `claimRewards` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `getCumulatedRewardsIncrement` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `isBlockedNft` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isNftTypeAndRankRedeemed1` | (`nftType`: `string`, `nftRank`: `string`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isNftTypeAndRankRedeemed2` | (`nftTypeIndex`: `BigNumberish`, `nftRank`: `string`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isNftTypeAndRankRedeemed3` | (`nftTypeIndex`: `BigNumberish`, `boostNumerator`: `BigNumberish`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isNftTypeAndRankRedeemed4` | (`nftType`: `string`, `boostNumerator`: `BigNumberish`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `ownerBlockNftType` | (`typeIndex`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `ownerUnblockNftType` | (`typeIndex`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `redeemedFactor` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `redeemedFactorIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `redeemedNft` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `redeemedNftIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `stakedHoprTokens` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `sync` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `totalLocked` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `unlock` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `unlockFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:422

___

### interface

• **interface**: `HoprStakeSeason4Interface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:401

___

### off

• **off**: `OnEvent`<[`HoprStakeSeason4`](HoprStakeSeason4.md)\>

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:417

___

### on

• **on**: `OnEvent`<[`HoprStakeSeason4`](HoprStakeSeason4.md)\>

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:418

___

### once

• **once**: `OnEvent`<[`HoprStakeSeason4`](HoprStakeSeason4.md)\>

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:419

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `BASIC_FACTOR_NUMERATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `BOOST_CAP` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `FACTOR_DENOMINATOR` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `LOCK_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `NFT_CONTRACT` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `PROGRAM_END` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `PROGRAM_START` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `REWARD_TOKEN` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `accounts` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `availableReward` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `claimRewards` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `getCumulatedRewardsIncrement` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isBlockedNft` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isNftTypeAndRankRedeemed1` | (`nftType`: `string`, `nftRank`: `string`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isNftTypeAndRankRedeemed2` | (`nftTypeIndex`: `BigNumberish`, `nftRank`: `string`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isNftTypeAndRankRedeemed3` | (`nftTypeIndex`: `BigNumberish`, `boostNumerator`: `BigNumberish`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isNftTypeAndRankRedeemed4` | (`nftType`: `string`, `boostNumerator`: `BigNumberish`, `hodler`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `onERC721Received` | (`operator`: `string`, `from`: `string`, `tokenId`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `onTokenTransfer` | (`_from`: `string`, `_value`: `BigNumberish`, `_data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `ownerBlockNftType` | (`typeIndex`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `ownerUnblockNftType` | (`typeIndex`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `reclaimErc20Tokens` | (`tokenAddress`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `reclaimErc721Tokens` | (`tokenAddress`: `string`, `tokenId`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `redeemedFactor` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `redeemedFactorIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `redeemedNft` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `redeemedNftIndex` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `stakedHoprTokens` | (`_account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `sync` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `totalLocked` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `unlock` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `unlockFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:1164

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

BaseContract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:82

___

### removeListener

• **removeListener**: `OnEvent`<[`HoprStakeSeason4`](HoprStakeSeason4.md)\>

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/HoprStakeSeason4.ts:420

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

___

### BOOST\_CAP

▸ **BOOST_CAP**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

___

### FACTOR\_DENOMINATOR

▸ **FACTOR_DENOMINATOR**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

___

### LOCK\_TOKEN

▸ **LOCK_TOKEN**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

___

### NFT\_CONTRACT

▸ **NFT_CONTRACT**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

___

### PROGRAM\_END

▸ **PROGRAM_END**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

___

### PROGRAM\_START

▸ **PROGRAM_START**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

___

### REWARD\_TOKEN

▸ **REWARD_TOKEN**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

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

___

### accounts

▸ **accounts**(`arg0`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`] & { `actualLockedTokenAmount`: `BigNumber` ; `claimedRewards`: `BigNumber` ; `cumulatedRewards`: `BigNumber` ; `lastSyncTimestamp`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`] & { `actualLockedTokenAmount`: `BigNumber` ; `claimedRewards`: `BigNumber` ; `cumulatedRewards`: `BigNumber` ; `lastSyncTimestamp`: `BigNumber`  }\>

___

### attach

▸ **attach**(`addressOrName`): [`HoprStakeSeason4`](HoprStakeSeason4.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprStakeSeason4`](HoprStakeSeason4.md)

#### Overrides

BaseContract.attach

___

### availableReward

▸ **availableReward**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

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

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprStakeSeason4`](HoprStakeSeason4.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprStakeSeason4`](HoprStakeSeason4.md)

#### Overrides

BaseContract.connect

___

### deployed

▸ **deployed**(): `Promise`<[`HoprStakeSeason4`](HoprStakeSeason4.md)\>

#### Returns

`Promise`<[`HoprStakeSeason4`](HoprStakeSeason4.md)\>

#### Overrides

BaseContract.deployed

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

___

### isBlockedNft

▸ **isBlockedNft**(`arg0`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

___

### isNftTypeAndRankRedeemed1

▸ **isNftTypeAndRankRedeemed1**(`nftType`, `nftRank`, `hodler`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `nftType` | `string` |
| `nftRank` | `string` |
| `hodler` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

___

### isNftTypeAndRankRedeemed2

▸ **isNftTypeAndRankRedeemed2**(`nftTypeIndex`, `nftRank`, `hodler`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `nftTypeIndex` | `BigNumberish` |
| `nftRank` | `string` |
| `hodler` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

___

### isNftTypeAndRankRedeemed3

▸ **isNftTypeAndRankRedeemed3**(`nftTypeIndex`, `boostNumerator`, `hodler`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `nftTypeIndex` | `BigNumberish` |
| `boostNumerator` | `BigNumberish` |
| `hodler` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

___

### isNftTypeAndRankRedeemed4

▸ **isNftTypeAndRankRedeemed4**(`nftType`, `boostNumerator`, `hodler`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `nftType` | `string` |
| `boostNumerator` | `BigNumberish` |
| `hodler` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

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

▸ **listeners**(`eventName?`): `Listener`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

`Listener`[]

#### Overrides

BaseContract.listeners

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

___

### owner

▸ **owner**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

___

### ownerBlockNftType

▸ **ownerBlockNftType**(`typeIndex`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `typeIndex` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

___

### ownerUnblockNftType

▸ **ownerUnblockNftType**(`typeIndex`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `typeIndex` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

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

___

### removeAllListeners

▸ **removeAllListeners**<`TEvent`\>(`eventFilter`): [`HoprStakeSeason4`](HoprStakeSeason4.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TEvent` | extends [`TypedEvent`](TypedEvent.md)<`any`, `any`, `TEvent`\> |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](TypedEventFilter.md)<`TEvent`\> |

#### Returns

[`HoprStakeSeason4`](HoprStakeSeason4.md)

#### Overrides

BaseContract.removeAllListeners

▸ **removeAllListeners**(`eventName?`): [`HoprStakeSeason4`](HoprStakeSeason4.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprStakeSeason4`](HoprStakeSeason4.md)

#### Overrides

BaseContract.removeAllListeners

___

### renounceOwnership

▸ **renounceOwnership**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

___

### stakedHoprTokens

▸ **stakedHoprTokens**(`_account`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

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

___

### totalLocked

▸ **totalLocked**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

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

___

### unlock

▸ **unlock**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

___

### unlockFor

▸ **unlockFor**(`account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>
