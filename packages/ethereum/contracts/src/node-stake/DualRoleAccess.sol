// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

import '@openzeppelin/contracts/access/AccessControlEnumerable.sol';

error UnsupportedRole();
error HoldingADifferentRole();
error ForbiddenRoleRevocation();
error ForbiddenRoleRenouncement();
error ForbiddenRoleAdminChange();

contract DualRoleAccess is AccessControlEnumerable {
  bytes32 public constant ADMIN_ROLE = keccak256('HOPR_STAKE_ADMIN_ROLE');
  bytes32 public constant NODE_ROLE = keccak256('HOPR_STAKE_NODE_CHAIN_KEY_ROLE');

  enum ApprovalLevel {
    NODE_ONLY,
    ADMIN_ONLY,
    NODE_AND_ADMIN,
    NODE_OR_ADMIN
  }

  /*
   * @dev when deploying the singleton contract, the default admin role is kept as 0x00,
   * so no account can alter the storage of the singleton contract.
   */
  constructor() {
    _setupRole(DEFAULT_ADMIN_ROLE, address(0));
  }

  /**
   * @dev this function is supposed to be called during the initial setup
   * @param _safeAddr address of the safe, which is set to be the DEFAULT_ADMIN.
   */
  function setupRoles(address _safeAddr, address[] memory _admin_key_addrs, address[] memory _node_key_addrs) internal {
    require(_safeAddr != address(0));
    _setupRole(DEFAULT_ADMIN_ROLE, _safeAddr);
    for (uint256 i = 0; i < _admin_key_addrs.length; i++) {
      _setupRole(ADMIN_ROLE, _admin_key_addrs[i]);
    }
    for (uint256 j = 0; j < _node_key_addrs.length; j++) {
      _setupRole(NODE_ROLE, _node_key_addrs[j]);
    }
  }

  /**
   * @dev Overload {renounceRole} to forbid it from being called directly by the key holder
   * This leave the only way of revoking a role to be by its admin role, which can only be
   * the DEFAULT_ADMIN (which should be the Safe)
   */
  function renounceRole(bytes32 role, address account) public override(AccessControl, IAccessControl) {
    revert ForbiddenRoleRenouncement();
  }

  /**
   * @dev Overload {_grantRole} to only allow two roles being granted
   */
  function _grantRole(bytes32 role, address account) internal override(AccessControlEnumerable) {
    // role provided is not allowed
    if (!isRoleValid(role)) {
      revert UnsupportedRole();
    }

    // role can only be granted if the account doesn't have the other role
    if ((role == ADMIN_ROLE && hasRole(NODE_ROLE, account)) || (role == NODE_ROLE && hasRole(ADMIN_ROLE, account))) {
      // if the account has a differnt role NODE_ROLE
      revert HoldingADifferentRole();
    }

    // if the role is one of the two and the account doesn't hold the other role
    super._grantRole(role, account);
  }

  /**
   * @dev Overload {_setRoleAdmin} to forbid the role admin from changing away from the DEFAULT_ADMIN
   * which should be the Safe
   */
  function _setRoleAdmin(bytes32 role, bytes32 adminRole) internal override(AccessControl) {
    revert ForbiddenRoleAdminChange();
  }

  /**
   * @dev Overload {_revokeRole} to prevent the DEFAULT_ADMIN_ROLE from being revoked
   */
  function _revokeRole(bytes32 role, address account) internal override(AccessControlEnumerable) {
    if (role == DEFAULT_ADMIN_ROLE) {
      revert ForbiddenRoleRevocation();
    }
    super._revokeRole(role, account);
  }

  /**
   * @dev Check if the account has one of two specified roles
   */
  function hasValidRole(address account) public view returns (bool) {
    return hasRole(NODE_ROLE, account) || hasRole(ADMIN_ROLE, account);
  }

  /**
   * @dev Check if the provided role is one of the two special roles
   */
  function isRoleValid(bytes32 role) public pure returns (bool) {
    return role == ADMIN_ROLE || role == NODE_ROLE;
  }

  // case 1: node only
  function _checkApprovalLevelNodeOnly(address[] memory accounts) internal view returns (bool) {
    for (uint256 i = 0; i < accounts.length; i++) {
      if (hasRole(NODE_ROLE, accounts[i])) {
        return true;
      }
    }
    return false;
  }

  // case 2: admin only
  function _checkApprovalLevelAdminOnly(address[] memory accounts) internal view returns (bool) {
    for (uint256 i = 0; i < accounts.length; i++) {
      if (hasRole(ADMIN_ROLE, accounts[i])) {
        return true;
      }
    }
    return false;
  }

  // case 3: admin AND node
  function _checkApprovalLevelAdminAndNode(address[] memory accounts) internal returns (bool) {
    bool atLeastOneAdmin;
    bool atLeastOneNode;
    for (uint256 i = 0; i < accounts.length; i++) {
      if (hasRole(ADMIN_ROLE, accounts[i])) {
        atLeastOneAdmin = true;
      } else if (hasRole(NODE_ROLE, accounts[i])) {
        atLeastOneNode = true;
      }
      if (atLeastOneAdmin && atLeastOneNode) {
        return true;
      }
    }
    return false;
  }

  // case 4: admin OR node
  function _checkApprovalLevelAdminOrNode(address[] memory accounts) internal returns (bool) {
    for (uint256 i = 0; i < accounts.length; i++) {
      if (hasRole(ADMIN_ROLE, accounts[i]) || hasRole(NODE_ROLE, accounts[i])) {
        return true;
      }
    }
    return false;
  }
}
