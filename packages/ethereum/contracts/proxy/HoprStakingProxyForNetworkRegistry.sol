// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import '@openzeppelin/contracts/access/Ownable.sol';
import '../IHoprNetworkRegistryRequirement.sol';

/**
 * @dev Interface for staking contract
 * source code at https://github.com/hoprnet/hopr-stake/tree/main/contracts
 * staking v2 is deployed at https://blockscout.com/xdai/mainnet/address/0x2cDD13ddB0346E0F620C8E5826Da5d7230341c6E
 * staking v3 is deployed at https://blockscout.com/xdai/mainnet/address/0xae933331ef0bE122f9499512d3ed4Fa3896DCf20
 */
contract IHoprStake {
  function stakedHoprTokens(address _account) public view returns (uint256) {}

  function isNftTypeAndRankRedeemed3(
    uint256 nftTypeIndex,
    uint256 boostNumerator,
    address hodler
  ) external view returns (bool) {}
}

/**
 * @dev Proxy for staking (v2/v3) contract, which an "HoprNetworkRegistry requirement" is implemented
 * Only accounts with HoprBoost NFTs that are of the type and rank in the `eligibleNftTypeAndRank` array
 * are considered as eligible, when their stake is also above the `stakeThreshold`
 */
contract HoprStakingProxyForNetworkRegistry is IHoprNetworkRegistryRequirement, Ownable {
  struct NftTypeAndRank {
    uint256 nftType;
    uint256 nftRank;
  }

  IHoprStake public immutable STAKE_CONTRACT; // contract of HoprStake contract
  uint256 public stakeThreshold; // minimum amount HOPR tokens being staked in the staking contract to be considered eligible
  NftTypeAndRank[] public eligibleNftTypeAndRank; // list of NFTs whose owner are considered as eligible to the network if the `stakeThreshold` is also met
  NftTypeAndRank[] public specialeNftTypeAndRank; // list of NFTs whose owner are considered as eligible to the network without meeting the `stakeThreshold`, e.g. "Dev NFT"

  event NftTypeAndRankAdded(uint256 indexed nftType, uint256 indexed nftRank); // emit when a new NFT type and rank gets included in the eligibility list
  event NftTypeAndRankRemoved(uint256 indexed nftType, uint256 indexed nftRank); // emit when a NFT type and rank gets removed from the eligibility list
  event SpecialNftTypeAndRankAdded(uint256 indexed nftType, uint256 indexed nftRank); // emit when a new special type and rank of NFT gets included in the eligibility list
  event SpecialNftTypeAndRankRemoved(uint256 indexed nftType, uint256 indexed nftRank); // emit when a special type and rank of NFT gets removed from the eligibility list
  event ThresholdUpdated(uint256 indexed threshold); // emit when the staking threshold gets updated.

  constructor(
    address stakeContract,
    address newOwner,
    uint256 minStake
  ) {
    STAKE_CONTRACT = IHoprStake(stakeContract);
    stakeThreshold = minStake;
    emit ThresholdUpdated(stakeThreshold);
    _transferOwnership(newOwner);
  }

  /**
   * @dev Checks if the provided account has
   * a) special NFTs, e.g. "Dev NFT"
   * b) redeemed any NFT of eligibleNftTypeAndRank and staked HOPR tokens above the `threshold`
   * @param account staker address that has a hopr nodes running
   */
  function isRequirementFulfilled(address account) external view returns (bool) {
    // if the account owns a special NFT, requirement is fulfilled
    for (uint256 i = 0; i < specialeNftTypeAndRank.length; i++) {
      NftTypeAndRank memory eligible = specialeNftTypeAndRank[i];
      if (STAKE_CONTRACT.isNftTypeAndRankRedeemed3(eligible.nftType, eligible.nftRank, account)) {
        return true;
      }
    }

    // when no special NFT is present, the account needs to 1) reach the minimum stake, 2) own an eligible NFT
    // for self-claiming accounts, check against the current criteria
    uint256 amount = STAKE_CONTRACT.stakedHoprTokens(account);
    if (amount < stakeThreshold) {
      // threshold does not meet
      return false;
    }
    // check on regular eligible NFTs.
    for (uint256 i = 0; i < eligibleNftTypeAndRank.length; i++) {
      NftTypeAndRank memory eligible = eligibleNftTypeAndRank[i];
      if (STAKE_CONTRACT.isNftTypeAndRankRedeemed3(eligible.nftType, eligible.nftRank, account)) {
        return true;
      }
    }

    return false;
  }

  /**
   * @dev Owner adds/updates NFT type and rank to the list of special NFTs in batch.
   * @param nftTypes Array of type indexes of the special HoprBoost NFT
   * @param nftRanks Array of HOPR boost numerator, which is associated to the special NFT
   */
  function ownerBatchAddSpecialNftTypeAndRank(uint256[] calldata nftTypes, uint256[] calldata nftRanks)
    external
    onlyOwner
  {
    require(
      nftTypes.length == nftRanks.length,
      'HoprStakingProxyForNetworkRegistry: ownerBatchAddSpecialNftTypeAndRank lengths mismatch'
    );
    for (uint256 index = 0; index < nftTypes.length; index++) {
      _addSpecialNftTypeAndRank(nftTypes[index], nftRanks[index]);
    }
  }

  /**
   * @dev Owner removes from list of special NFTs in batch.
   * @param nftTypes Array of type index of the special HoprBoost NFT
   * @param nftRanks Array of  HOPR boost numerator, which is associated to the special NFT
   */
  function ownerBatchRemoveSpecialNftTypeAndRank(uint256[] calldata nftTypes, uint256[] calldata nftRanks)
    external
    onlyOwner
  {
    require(
      nftTypes.length == nftRanks.length,
      'HoprStakingProxyForNetworkRegistry: ownerBatchRemoveSpecialNftTypeAndRank lengths mismatch'
    );
    for (uint256 index = 0; index < nftTypes.length; index++) {
      _removeSpecialNftTypeAndRank(nftTypes[index], nftRanks[index]);
    }
  }

  /**
   * @dev Owner adds/updates NFT type and rank to the list of eligibles NFTs in batch.
   * @param nftTypes Array of type indexes of the eligible HoprBoost NFT
   * @param nftRanks Array of HOPR boost numerator, which is associated to the eligible NFT
   */
  function ownerBatchAddNftTypeAndRank(uint256[] calldata nftTypes, uint256[] calldata nftRanks) external onlyOwner {
    require(
      nftTypes.length == nftRanks.length,
      'HoprStakingProxyForNetworkRegistry: ownerBatchAddNftTypeAndRank lengths mismatch'
    );
    for (uint256 index = 0; index < nftTypes.length; index++) {
      _addNftTypeAndRank(nftTypes[index], nftRanks[index]);
    }
  }

  /**
   * @dev Owner removes from list of eligible NFTs in batch.
   * @param nftTypes Array of type index of the eligible HoprBoost NFT
   * @param nftRanks Array of  HOPR boost numerator, which is associated to the eligible NFT
   */
  function ownerBatchRemoveNftTypeAndRank(uint256[] calldata nftTypes, uint256[] calldata nftRanks) external onlyOwner {
    require(
      nftTypes.length == nftRanks.length,
      'HoprStakingProxyForNetworkRegistry: ownerBatchRemoveNftTypeAndRank lengths mismatch'
    );
    for (uint256 index = 0; index < nftTypes.length; index++) {
      _removeNftTypeAndRank(nftTypes[index], nftRanks[index]);
    }
  }

  /**
   * @dev Owner adds/updates NFT type and rank to the list of eligibles NFTs.
   * @param nftType Type index of the eligible HoprBoost NFT
   * @param nftRank HOPR boost numerator, which is associated to the eligible NFT
   */
  function ownerAddNftTypeAndRank(uint256 nftType, uint256 nftRank) external onlyOwner {
    _addNftTypeAndRank(nftType, nftRank);
  }

  /**
   * @dev Owner removes from list of eligible NFTs
   * @param nftType Type index of the eligible HoprBoost NFT
   * @param nftRank HOPR boost numerator, which is associated to the eligible NFT
   */
  function ownerRemoveNftTypeAndRank(uint256 nftType, uint256 nftRank) external onlyOwner {
    _removeNftTypeAndRank(nftType, nftRank);
  }

  /**
   * @dev Owner updates the minimal staking amount required for users to add themselves onto the HoprNetworkRegistry
   * @param newThreshold Minimum stake of HOPR token
   */
  function ownerUpdateThreshold(uint256 newThreshold) external onlyOwner {
    stakeThreshold = newThreshold;
    emit ThresholdUpdated(stakeThreshold);
  }

  /**
   * @dev adds NFT type and rank to the list of special NFTs.
   * @param nftType Type index of the special HoprBoost NFT
   * @param nftRank HOPR boost numerator, which is associated to the special NFT
   */
  function _addSpecialNftTypeAndRank(uint256 nftType, uint256 nftRank) private {
    uint256 i = 0;
    for (i; i < specialeNftTypeAndRank.length; i++) {
      // walk through all the types
      if (specialeNftTypeAndRank[i].nftType == nftType && specialeNftTypeAndRank[i].nftRank == nftRank) {
        // already exist;
        return;
      }
    }
    specialeNftTypeAndRank.push(NftTypeAndRank({nftType: nftType, nftRank: nftRank}));
    emit SpecialNftTypeAndRankAdded(nftType, nftRank);
    (nftType, nftRank);
  }

  /**
   * @dev Remove from list of special NFTs
   * @param nftType Type index of the special HoprBoost NFT
   * @param nftRank HOPR boost numerator, which is associated to the special NFT
   */
  function _removeSpecialNftTypeAndRank(uint256 nftType, uint256 nftRank) private {
    // walk through
    for (uint256 i = 0; i < specialeNftTypeAndRank.length; i++) {
      if (specialeNftTypeAndRank[i].nftType == nftType && specialeNftTypeAndRank[i].nftRank == nftRank) {
        // overwrite with the last element in the array
        specialeNftTypeAndRank[i] = specialeNftTypeAndRank[specialeNftTypeAndRank.length - 1];
        specialeNftTypeAndRank.pop();
        emit SpecialNftTypeAndRankRemoved(nftType, nftRank);
      }
    }
  }

  /**
   * @dev adds NFT type and rank to the list of eligibles NFTs.
   * @param nftType Type index of the eligible HoprBoost NFT
   * @param nftRank HOPR boost numerator, which is associated to the eligible NFT
   */
  function _addNftTypeAndRank(uint256 nftType, uint256 nftRank) private {
    uint256 i = 0;
    for (i; i < eligibleNftTypeAndRank.length; i++) {
      // walk through all the types
      if (eligibleNftTypeAndRank[i].nftType == nftType && eligibleNftTypeAndRank[i].nftRank == nftRank) {
        // already exist;
        return;
      }
    }
    eligibleNftTypeAndRank.push(NftTypeAndRank({nftType: nftType, nftRank: nftRank}));
    emit NftTypeAndRankAdded(nftType, nftRank);
  }

  /**
   * @dev Remove from list of eligible NFTs
   * @param nftType Type index of the eligible HoprBoost NFT
   * @param nftRank HOPR boost numerator, which is associated to the eligible NFT
   */
  function _removeNftTypeAndRank(uint256 nftType, uint256 nftRank) private {
    // walk through
    for (uint256 i = 0; i < eligibleNftTypeAndRank.length; i++) {
      if (eligibleNftTypeAndRank[i].nftType == nftType && eligibleNftTypeAndRank[i].nftRank == nftRank) {
        // overwrite with the last element in the array
        eligibleNftTypeAndRank[i] = eligibleNftTypeAndRank[eligibleNftTypeAndRank.length - 1];
        eligibleNftTypeAndRank.pop();
        emit NftTypeAndRankRemoved(nftType, nftRank);
      }
    }
  }
}
