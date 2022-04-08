// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import '@openzeppelin/contracts/access/Ownable.sol';
import '../IHoprNetworkRegistryRequirement.sol';

/**
 * @dev Interface for staking contract
 * source code at https://github.com/hoprnet/hopr-stake/tree/main/contracts
 * staking v2 is deployed at https://blockscout.com/xdai/mainnet/address/0x2cDD13ddB0346E0F620C8E5826Da5d7230341c6E
 */
contract IStake {
  function stakedHoprTokens(address _account) public view returns (uint256) {}

  function isNftTypeAndRankRedeemed3(
    uint256 nftTypeIndex,
    uint256 boostNumerator,
    address hodler
  ) external view returns (bool) {}
}

/**
 * @dev Proxy for staking (v2) contract, which an "HoprNetworkRegistry requirement" is implemented
 */
contract HoprStakingProxyForNetworkRegistry is IHoprNetworkRegistryRequirement, Ownable {
  struct NftTypeAndRank {
    uint256 nftType;
    uint256 nftRank;
  }

  IStake public immutable STAKE_CONTRACT;
  NftTypeAndRank[] public eligibleNftTypeAndRank;
  uint256 public stakeThreshold;

  event AddedNftTypeAndRank(uint256 indexed nftType, uint256 indexed nftRank);
  event RemovedNftTypeAndRank(uint256 indexed nftType, uint256 indexed nftRank);
  event UpdatedThreshold(uint256 indexed threshold);

  constructor(
    address stakeContract,
    address newOwner,
    uint256 minStake
  ) {
    STAKE_CONTRACT = IStake(stakeContract);
    stakeThreshold = minStake;
    emit UpdatedThreshold(stakeThreshold);
    _transferOwnership(newOwner);
  }

  /**
   * @dev Checks if the provided account has staked any NFT of eligibleNftTypeAndRank and
   * if staked token amount is above `threshold` of staked HOPR tokens
   * @param account staker address that has a hopr nodes running
   */
  function isRequirementFulfilled(address account) external view returns (bool) {
    // for self-claiming accounts, check against the current criteria
    uint256 amount = STAKE_CONTRACT.stakedHoprTokens(account);
    if (amount < stakeThreshold) {
      // threshold does not meet
      return false;
    }

    for (uint256 i = 0; i < eligibleNftTypeAndRank.length; i++) {
      NftTypeAndRank memory eligible = eligibleNftTypeAndRank[i];
      if (STAKE_CONTRACT.isNftTypeAndRankRedeemed3(eligible.nftType, eligible.nftRank, account)) {
        return true;
      }
    }

    return false;
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
    emit UpdatedThreshold(stakeThreshold);
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
    emit AddedNftTypeAndRank(nftType, nftRank);
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
        emit RemovedNftTypeAndRank(nftType, nftRank);
      }
    }
  }
}
