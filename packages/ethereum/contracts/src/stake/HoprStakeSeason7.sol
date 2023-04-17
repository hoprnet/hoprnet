// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;
import './HoprStakeBase.sol';

/**
 * Staking program - Season 7
 * Inherit a HoprStakeBase.
 * Include new function for owner to stake for a group of accounts
 */
contract HoprStakeSeason7 is HoprStakeBase {
  using SafeERC20 for IERC20;

  /**
   * Staking season 7 starts at 1682510400 2pm CET 26th April 2023 and ends at 1690372800 2pm CET 26th July 2023
   * Basic APY is 1.25 % (= 1.25 / 100 * 1e12 / (365 * 24 * 60 * 60))
   * the boost cap is 250k xHOPR (25e22)
   * @param _newOwner address Address of the new owner. In production, it's 0xD9a00176Cf49dFB9cA3Ef61805a2850F45Cb1D05
   * @param _nftAddress address Address of the NFT contract. In production, it's 0x43d13D7B83607F14335cF2cB75E87dA369D056c7
   * @param _lockToken address Address of the stake token xHOPR. In production, it's 0xD057604A14982FE8D88c5fC25Aac3267eA142a08
   * @param _rewardToken address Address of the reward token wxHOPR. In production, it's 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1

   */
  constructor(
    address _newOwner,
    address _nftAddress,
    address _lockToken,
    address _rewardToken
  ) HoprStakeBase(_newOwner, 1674738000, 1682510400, 396, 25e22, _nftAddress, _lockToken, _rewardToken) {}

  /**
   * @dev allow the owner to stake tokens for some accounts
   * @notice Owner should have called `increaseApproval(s6contract, largeEnoughValue)` where ideally the
   * `largeEnoughValue` equals the sum of all the stakes
   * After PROGRAM_END, it refuses tokens;
   * @param _accounts address array of addresses that receives LOCK_TOKEN
   * @param _stakes uint256 array of token amount being transferred from owner's wallet
   */
  function batchStakeFor(address[] calldata _accounts, uint256[] calldata _stakes) external onlyOwner {
    require(block.timestamp <= PROGRAM_END, 'HoprStake: Program ended, cannot stake anymore.');
    require(_accounts.length == _stakes.length, 'HoprStake: accounts and stakes array lengths do not match');
    uint256 increaseLocked;
    address acc;
    uint256 val;
    for (uint256 index = 0; index < _accounts.length; index++) {
      acc = _accounts[index];
      val = _stakes[index];
      // for each account
      _sync(acc);
      accounts[acc].actualLockedTokenAmount += val;
      increaseLocked += val;
      emit Staked(acc, val);
    }
    totalLocked += increaseLocked;
    IERC20(LOCK_TOKEN).safeTransferFrom(owner(), address(this), increaseLocked);
  }

  /**
   * @dev Owner can block a list of NFTs from being redeemed
   * in the current staking contract by its type name (as in HoprBoost)
   * @param typeIndexes array of integer Type index to be blocked
   */
  function ownerBatchBlockNftType(uint256[] calldata typeIndexes) external onlyOwner {
    for (uint256 index = 0; index < typeIndexes.length; index++) {
      uint256 typeIndex = typeIndexes[index];
      require(!isBlockedNft[typeIndex], 'HoprStake: NFT type is already blocked');
      _ownerBlockNftType(typeIndex);
    }
  }

  /**
   * @dev Owner can allow blocked a list of NFTs to be redeemable.
   * @param typeIndexes array of integer Type index to be allowed
   */
  function ownerBatchUnblockNftType(uint256[] calldata typeIndexes) external onlyOwner {
    for (uint256 index = 0; index < typeIndexes.length; index++) {
      uint256 typeIndex = typeIndexes[index];
      require(isBlockedNft[typeIndex], 'HoprStake: NFT type is not blocked');
      isBlockedNft[typeIndex] = false;
      emit NftAllowed(typeIndex);
    }
  }
}
