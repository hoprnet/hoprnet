// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

/**
 * @title DualRoleAccess interface
 */
interface IDualRoleAccess {
  function ADMIN_ROLE() external view returns (bytes32);

  function NODE_ROLE() external view returns (bytes32);

  function hasRole(bytes32 role, address account) external view returns (bool);
}
