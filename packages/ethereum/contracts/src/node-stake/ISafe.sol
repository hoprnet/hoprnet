// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

/**
 * @title Safe interface
 */
interface ISafe {
  function getOwners() external view returns (address[] memory);
  
  function getModulesPaginated(address start, uint256 pageSize) external view returns (address[] memory array, address next);
}
