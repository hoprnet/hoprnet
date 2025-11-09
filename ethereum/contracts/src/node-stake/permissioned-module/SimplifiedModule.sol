// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity ^0.8.0;

import { Enum } from "safe-contracts/common/Enum.sol";
import { IAvatar } from "../../interfaces/IAvatar.sol";
import { UUPSUpgradeable } from "openzeppelin-contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import { OwnableUpgradeable } from "openzeppelin-contracts-upgradeable/access/OwnableUpgradeable.sol";

abstract contract SimplifiedModuleEvents {
    // module emit event when execution is successful on avator
    event ExecutionSuccess();
    // module emit event when execution is failed on avator
    event ExecutionFailure();
}

/**
 * @title Simplified Module Interface - A contract that can pass messages to a Module Manager contract if enabled by
 * that contract.
 * @dev Adapted from Zodiac's `Module.sol` at
 * https://github.com/gnosis/zodiac/tree/8a77e7b224af8004bd9f2ff4e2919642e93ffd85/contracts/core/Module.sol
 *  , which * was audited https://github.com/gnosis/zodiac/tree/master/audits
 * This module removes target attribute, removes guard, and uses UUPS proxy.
 */
abstract contract SimplifiedModule is UUPSUpgradeable, OwnableUpgradeable, SimplifiedModuleEvents {
    /**
     * @dev Passes a transaction to be executed by the programmable account (i.e. "avatar" as per notion used in
     * Zodiac).
     * @notice Can only be called by this contract.
     * @param to Destination address of module transaction.
     * @param value Ether value of module transaction.
     * @param data Data payload of module transaction.
     * @param operation Operation type of module transaction: 0 == call, 1 == delegate call.
     */
    function exec(
        address to,
        uint256 value,
        bytes memory data,
        Enum.Operation operation
    )
        internal
        returns (bool success)
    {
        success = IAvatar(owner()).execTransactionFromModule(to, value, data, operation);
        if (success) {
            emit ExecutionSuccess();
        } else {
            emit ExecutionFailure();
        }
    }

    /**
     * @dev Passes a transaction to be executed by the avatar and returns data.
     * @notice Can only be called by this contract.
     * @param to Destination address of module transaction.
     * @param value Ether value of module transaction.
     * @param data Data payload of module transaction.
     * @param operation Operation type of module transaction: 0 == call, 1 == delegate call.
     */
    function execAndReturnData(
        address to,
        uint256 value,
        bytes memory data,
        Enum.Operation operation
    )
        internal
        returns (bool success, bytes memory returnedData)
    {
        (success, returnedData) = IAvatar(owner()).execTransactionFromModuleReturnData(to, value, data, operation);
        if (success) {
            emit ExecutionSuccess();
        } else {
            emit ExecutionFailure();
        }
    }

    /**
     * @dev Override {_authorizeUpgrade} to only allow owner to upgrade the contract
     */
    function _authorizeUpgrade(address) internal override(UUPSUpgradeable) onlyOwner { }
}
