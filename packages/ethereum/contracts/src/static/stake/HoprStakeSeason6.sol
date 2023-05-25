// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;
import './HoprStakeBase.sol';

/**
 * Staking program - Season 6
 * Inherit a HoprStakeBase.
 * Include new function for owner to stake for a group of accounts
 */
contract HoprStakeSeason6 is HoprStakeBase {
  using SafeERC20 for IERC20;

  /**
   * Staking season 6 starts at 1674738000 2pm CET 26th January 2023 and ends at 1682510400 2pm CET 26th April 2023
   * Basic APY is 1.25 % (= 1.25 / 100 * 1e12 / (365 * 24 * 60 * 60))
   *  the boost cap is 250k xHOPR (25e22)
   * It blocks HODLr, DAO_v2, Surveyor, Wildhorn_v2, PuzzleHunt_v1, PuzzleHunt_v2, PuzzleHunt_v3, ETH_Denver, 
   * Lucky NFTs, PuzzleHunt_v3, Matterhorn, DAO_v3, AMA_zing, Restaker, QV_game, Meme_master, Tokenlon_AMA, 
   * TokenlonAMA, Meme_master_v2 and Metadata_games at start
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
  ) HoprStakeBase(_newOwner, 1674738000, 1682510400, 396, 25e22, _nftAddress, _lockToken, _rewardToken) {
    // block a selection of HoprBoost NFTs
    _ownerBlockNftType(2); // HODLr
    _ownerBlockNftType(3); // Wildhorn_v1
    _ownerBlockNftType(4); // PuzzleHunt_v1
    _ownerBlockNftType(7); // PuzzleHunt_v2
    _ownerBlockNftType(8); // Wildhorn_v2
    _ownerBlockNftType(9); // DAO_v2
    _ownerBlockNftType(10); // Surveyor
    _ownerBlockNftType(11); // ETH_Denver
    _ownerBlockNftType(12); // Lucky
    _ownerBlockNftType(13); // PuzzleHunt_v3
    _ownerBlockNftType(14); // Matterhorn
    _ownerBlockNftType(15); // DAO_v3
    _ownerBlockNftType(16); // AMA_zing
    _ownerBlockNftType(17); // Restaker
    _ownerBlockNftType(18); // QV_game
    _ownerBlockNftType(19); // Meme_master
    _ownerBlockNftType(20); // Tokenlon_AMA
    _ownerBlockNftType(21); // TokenlonAMA
    _ownerBlockNftType(22); // Meme_master_v2
    _ownerBlockNftType(23); // Metadata_games
  }

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
}
