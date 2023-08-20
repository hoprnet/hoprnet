// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.19;

import { Ownable } from "openzeppelin-contracts/access/Ownable.sol";
import { IHoprNetworkRegistryRequirement } from "../interfaces/INetworkRegistryRequirement.sol";

/**
 * @dev Dummy roxy which return true if an address is registered by the owner, when isAllAllowed is false.
 * It allows all the accounts when isAllAllowed is set to true. By default isAllAllowed is false.
 * Eligible account can register as many nodes as possible, capped at `type(uint256).max`
 */
contract HoprDummyProxyForNetworkRegistry is IHoprNetworkRegistryRequirement, Ownable {
    mapping(address account => bool isRegistered) private registeredAccounts;
    uint256 public constant MAX_REGISTRATION_PER_ACCOUNT = type(uint256).max;
    bool public isAllAllowed;

    event AccountRegistered(address indexed account);
    event AccountDeregistered(address indexed account);
    event AllowAllAccountsEligible(bool isAllowed);

    constructor(address newOwner) {
        _transferOwnership(newOwner);
        isAllAllowed = false;
        emit AllowAllAccountsEligible(false);
    }

    /**
     * @dev Checks if the provided account is registered by the owner
     * @param account address of the account that runs a hopr node
     */
    function maxAllowedRegistrations(address account) external view returns (uint256) {
        if (isAllAllowed || registeredAccounts[account]) {
            return MAX_REGISTRATION_PER_ACCOUNT;
        } else {
            return 0;
        }
    }

    /**
     * @dev Get if the staking account is eligible to act on node address
     */
    function canOperateFor(address, address) external pure returns (bool eligiblity) {
        return true;
    }

    /**
     * @dev Update the global toggle that allows all the accounts to be eligible
     */
    function updateAllowAll(bool _updatedAllow) external onlyOwner {
        if (isAllAllowed == _updatedAllow) {
            return;
        }
        isAllAllowed = _updatedAllow;
        emit AllowAllAccountsEligible(_updatedAllow);
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
        if (!registeredAccounts[account]) {
            registeredAccounts[account] = true;
            emit AccountRegistered(account);
        }
    }

    /**
     * @dev remove account from the registry list
     * @param account address to be removed from the registry
     */
    function _removeAccount(address account) private {
        if (registeredAccounts[account]) {
            delete registeredAccounts[account];
            emit AccountDeregistered(account);
        }
    }
}
