// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;
import './HoprStakeBase.sol';

/**
 * Staking program - Season 5
 * Inherit a HoprStakeBase.
 */
contract HoprStakeSeason5 is HoprStakeBase {
  /**
   * Staking season 5 starts at 1666785600 2pm CEST 26th October 2022 and ends at 1674738000 2pm CET 26th January 2023
   * Basic APY is 2.5 % and the boost cap is 200k xHOPR
   * It blocks HODLr, DAO_v2, Surveyor, Wildhorn_v2, PuzzleHunt_v1, PuzzleHunt_v2, PuzzleHunt_v3, ETH_Denver and Lucky NFTs at start
   * @param _newOwner address Address of the new owner. This new owner can reclaim any ERC20 and ERC721 token being accidentally sent to the lock contract.
   * @param _nftAddress address Address of the NFT contract.
   * @param _lockToken address Address of the stake token xHOPR.
   * @param _rewardToken address Address of the reward token wxHOPR.
   */
  constructor(
    address _newOwner,
    address _nftAddress,
    address _lockToken,
    address _rewardToken
  )
    HoprStakeBase(
      block.chainid == 100 ? 0xD9a00176Cf49dFB9cA3Ef61805a2850F45Cb1D05 : _newOwner,
      1666785600,
      1674738000,
      793,
      2e23,
      _nftAddress,
      _lockToken,
      _rewardToken
    )
  {
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
  }
}
