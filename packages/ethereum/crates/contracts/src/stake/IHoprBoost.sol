// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;

import '@openzeppelin/contracts/token/ERC721/extensions/IERC721Metadata.sol';

/**
 * @dev Required interface of an ERC721 compliant contract.
 */
interface IHoprBoost is IERC721Metadata {
  /**
   * @dev Returns the boost factor and the redeem deadline associated with ``tokenId``.
   * @param tokenId uint256 token Id of the boost.
   */
  function boostOf(uint256 tokenId) external view returns (uint256, uint256);

  /**
   * @dev Returns the boost type associated with ``tokenId``.
   * @param tokenId uint256 token Id of the boost.
   */
  function typeOf(uint256 tokenId) external view returns (string memory);

  /**
   * @dev Returns the boost type index associated with ``tokenId``.
   * @param tokenId uint256 token Id of the boost.
   */
  function typeIndexOf(uint256 tokenId) external view returns (uint256);
}
