// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity ^0.8.0;

import "../../interfaces/IAvatar.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";


/**
 * @title Simplified Module Interface - A contract that can pass messages to a Module Manager contract if enabled by that contract.
 * @dev Adapted from `Module.sol` at commit 8a77e7b224af8004bd9f2ff4e2919642e93ffd85, which 
 * was audited https://github.com/gnosis/zodiac/tree/master/audits
 * This module removes target attribute, removes guard, and uses UUPS proxy.
 */
abstract contract SimplifiedModule is UUPSUpgradeable, OwnableUpgradeable {
    // Address that will ultimately execute function calls.
    address public avatar; 

    // Emitted each time the avatar is set.
    event AvatarSet(address indexed previousAvatar, address indexed newAvatar);

    /**
     * @dev Sets the avatar to a new avatar (`newAvatar`).
     * @notice Can only be called by the current owner.
     * @param _avatar address of the new avatar
     */
    function setAvatar(address _avatar) external onlyOwner {
        address previousAvatar = avatar;
        avatar = _avatar;
        emit AvatarSet(previousAvatar, _avatar);
    }

    /**
     * @dev Passes a transaction to be executed by the avatar.
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
    ) internal virtual returns (bool success) {
        success = IAvatar(avatar).execTransactionFromModule(
            to,
            value,
            data,
            operation
        );
        return success;
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
    ) internal virtual returns (bool success, bytes memory returnData) {
        (success, returnData) = IAvatar(avatar)
            .execTransactionFromModuleReturnData(to, value, data, operation);
        return (success, returnData);
    }

    /**
     * @dev Override {_authorizeUpgrade} to only allow owner to upgrade the contract
     */
    function _authorizeUpgrade(address) internal override onlyOwner {}
}
