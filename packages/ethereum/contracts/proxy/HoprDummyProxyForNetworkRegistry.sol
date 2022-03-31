// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import '@openzeppelin/contracts/access/Ownable.sol';
import '../IHoprNetworkRegistryRequirement.sol';

/**
 * @dev Dummy roxy which return true if an address is registered by the owner
 */
contract HoprDummyProxyForNetworkRegistry is IHoprNetworkRegistryRequirement, Ownable {
  mapping(address => bool) registeredAccounts;
  event AccountRegistered(address indexed account);
  event AccountDeregistered(address indexed account);

  constructor(address newOwner) {
    _transferOwnership(newOwner);
  }

  /**
   * @dev Checks if the provided account is registered by the owner
   * @param account address of the account that runs a hopr node
   */
  function isRequirementFulfilled(address account) external view returns (bool) {
    return registeredAccounts[account];
  }

  /**
   * @dev Owner add accounts onto the registry list in batch.
   * @param accounts addresses to be removed from the registry
   */
  function ownerBatchAddAccounts(address[] calldata accounts) external onlyOwner {
    for (uint256 index = 0; index < accounts.length; index++) {
      _addAccount(accounts[index]);
    }
  }

  /**
   * @dev Owner removes from list of eligible NFTs in batch.
   * @param accounts addresses to be removed from the registry
   */
  function ownerBatchRemoveAccounts(address[] calldata accounts) external onlyOwner {
    for (uint256 index = 0; index < accounts.length; index++) {
      _removeAccount(accounts[index]);
    }
  }

  /**
   * @dev Owner add account onto the registry list
   * @param account address to be added onto the registry
   */
  function ownerAddAccount(address account) external onlyOwner {
    _addAccount(account);
  }

  /**
   * @dev Owner move account from the registry list
   * @param account address to be removed from the registry
   */
  function ownerRemoveAccount(address account) external onlyOwner {
    _removeAccount(account);
  }

  /**
   * @dev add account onto the registry list
   * @param account address to be added into the registry
   */
  function _addAccount(address account) private {
    registeredAccounts[account] = true;
    emit AccountRegistered(account);
  }

  /**
   * @dev remove account from the registry list
   * @param account address to be removed from the registry
   */
  function _removeAccount(address account) private {
    delete registeredAccounts[account];
    emit AccountDeregistered(account);
  }
}
