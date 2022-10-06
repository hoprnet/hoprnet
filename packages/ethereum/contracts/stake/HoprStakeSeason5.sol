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
   */
  constructor() HoprStakeBase(0xD9a00176Cf49dFB9cA3Ef61805a2850F45Cb1D05, 1666785600, 1674738000, 793, 2e23) {
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
