// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import '@openzeppelin/contracts/access/Ownable.sol';

contract IStake {
  function stakedHoprTokens(address _account) public view returns (uint256) {}

  function isNftTypeAndRankRedeemed3(
    uint256 nftTypeIndex,
    uint256 boostNumerator,
    address hodler
  ) external view returns (bool) {}
}

/**
 * @dev Whitelist addresses and hopr nodes
 */
contract HoprNetworkRegistry is Ownable {
  struct NftTypeAndRank {
    uint256 nftType;
    uint256 nftRank;
  }

  IStake public immutable STAKE_CONTRACT;
  mapping(address => string) public stakerToMultiaddr;
  mapping(string => address) public multiaddrToStaker;
  mapping(address => uint256) public stakedAmount; // for accounts that are not part of the staking program
  NftTypeAndRank[] public eligibleNftTypeAndRank;
  uint256 public stakeThreshold;

  event AddedToWhitelist(address indexed staker, string HoprMultiaddr);
  event OwnerAddedToWhitelist(address indexed staker, uint256 indexed amount, string HoprMultiaddr);
  event OwnerRemovedFromWhitelist(address indexed staker);
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
    transferOwnership(newOwner);
  }

  /**
   * @dev Checks if the msg.sender has staked any NFT of eligibleNftTypeAndRank and
   * if staked token amount is above `threshold` of staked HOPR tokens
   * @notice It allows msg.sender to update whitelist node address.
   * @param hoprAddress Hopr nodes id. e.g. 16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1
   */
  function addToWhitelist(string calldata hoprAddress) external returns (bool) {
    uint256 amount = STAKE_CONTRACT.stakedHoprTokens(msg.sender);
    require(amount >= stakeThreshold, 'HoprNetworkRegistry: staked amount does not meet the threshold');

    for (uint256 i = 0; i < eligibleNftTypeAndRank.length; i++) {
      NftTypeAndRank memory eligible = eligibleNftTypeAndRank[i];
      if (STAKE_CONTRACT.isNftTypeAndRankRedeemed3(eligible.nftType, eligible.nftRank, msg.sender)) {
        stakerToMultiaddr[msg.sender] = hoprAddress;
        multiaddrToStaker[hoprAddress] = msg.sender;
        // stakedAmount[msg.sender] = amount; // DO NOT register staked token amount for participants of staking program
        emit AddedToWhitelist(msg.sender, hoprAddress);
        return true;
      }
    }
    return false;
  }

  /**
   * @dev Owner adds Ethereum addresses and HOPR node ids to the whitelist.
   * Allows owner to set arbitrary HOPR Addresses on the whitelist even if the stakers do not fulfill staking requirements.
   * HOPR node address validation should be done off-chain.
   * @notice It allows owner to overwrite exisitng entries. Owner can also add accounts with amounts below threshold,
   * Those accounts will unfortunately not considered as "whitelisted", as threshold is
   * @param hoprAddresses Array of hopr nodes id. e.g. 16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1
   * @param stakers Array of Ethereum staker accounts, e.g. 0xf6A8b267f43998B890857f8d1C9AabC68F8556ee
   * @param amounts Array of staked amounts, e.g. 1000000000000000000
   */
  function ownerAddToWhitelist(
    string[] calldata hoprAddresses,
    address[] calldata stakers,
    uint256[] calldata amounts
  ) external onlyOwner {
    require(hoprAddresses.length == stakers.length, 'HoprNetworkRegistry: hoprAddresses and stakers lengths mismatch');
    require(amounts.length == stakers.length, 'HoprNetworkRegistry: amounts and stakers lengths mismatch');
    for (uint256 i = 0; i < stakers.length; i++) {
      require(amounts[i] > 0, 'HoprNetworkRegistry: staked amount should be above zero');

      string memory hoprAddress = hoprAddresses[i];
      address staker = stakers[i];
      uint256 amount = amounts[i];
      stakerToMultiaddr[staker] = hoprAddress;
      multiaddrToStaker[hoprAddress] = staker;
      stakedAmount[staker] = amount;
      emit OwnerAddedToWhitelist(staker, amount, hoprAddress);
    }
  }

  /**
   * @dev Owner removes previously owner-added Ethereum addresses and HOPR node ids from the whitelist.
   * @notice Owner can even remove self-declared entries.
   * @param stakers Array of Ethereum staker accounts, e.g. 0xf6A8b267f43998B890857f8d1C9AabC68F8556ee
   */
  function ownerRemoveFromWhitelist(address[] calldata stakers) external onlyOwner {
    for (uint256 i = 0; i < stakers.length; i++) {
      address staker = stakers[i];
      string memory hoprAddress = stakerToMultiaddr[staker];
      delete stakerToMultiaddr[staker];
      delete multiaddrToStaker[hoprAddress];
      delete stakedAmount[staker];
      emit OwnerRemovedFromWhitelist(staker);
    }
  }

  /**
   * @dev Owner adds/updates NFT type and rank to the list of eligibles NFTs in batch.
   * @param nftTypes Array of type indexes of the eligible HoprBoost NFT
   * @param nftRanks Array of HOPR boost numerator, which is associated to the eligible NFT
   */
  function ownerBatchAddNftTypeAndRank(uint256[] calldata nftTypes, uint256[] calldata nftRanks) external onlyOwner {
    require(nftTypes.length == nftRanks.length, 'HoprNetworkRegistry: ownerBatchAddNftTypeAndRank lengths mismatch');
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
    require(nftTypes.length == nftRanks.length, 'HoprNetworkRegistry: ownerRemoveNftTypeAndRank lengths mismatch');
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
   * @dev Owner updates the minimal staking amount required for users to add themselves onto the whitelist
   * @param newThreshold Minimum stake of HOPR token
   */
  function ownerUpdateThreshold(uint256 newThreshold) external onlyOwner {
    stakeThreshold = newThreshold;
    emit UpdatedThreshold(stakeThreshold);
  }

  /**
   * @dev Returns if a hopr address is whitelisted according to the latest criteria (threshold and
   * eligible NFT type and rank).
   * @notice This checks a node address against the the current whitelisting criteria.
   * E.g. A participant of staking program was eligible when it called `addToWhitelist`, but later the
   * owner lifts the threshold beyond the staking amount of the participant. Now the participant
   * would NO LONGER be "whitelisted" according to this function
   * @param hoprAddress hopr node address
   */
  function isWhitelisted(string calldata hoprAddress) public view returns (bool) {
    address staker = multiaddrToStaker[hoprAddress];
    if (staker == address(0)) {
      // this address has never been registered
      return false;
    }

    if (stakedAmount[staker] >= stakeThreshold) {
      // this address is added by the owner with a positive stake,
      // when the owner-added stake is above the threshold, it's whitelisted
      return true;
    }

    // for self-claiming accounts, check against the current criteria
    uint256 amount = STAKE_CONTRACT.stakedHoprTokens(staker);
    if (amount < stakeThreshold) {
      // threshold does not meet
      return false;
    }

    for (uint256 i = 0; i < eligibleNftTypeAndRank.length; i++) {
      NftTypeAndRank memory eligible = eligibleNftTypeAndRank[i];
      if (STAKE_CONTRACT.isNftTypeAndRankRedeemed3(eligible.nftType, eligible.nftRank, staker)) {
        return true;
      }
    }

    return false;
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
