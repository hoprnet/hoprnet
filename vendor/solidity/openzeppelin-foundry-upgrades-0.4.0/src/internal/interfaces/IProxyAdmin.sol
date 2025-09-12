// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IProxyAdmin {
    /**
     * Upgrades a proxy to a new implementation without calling a function on the new implementation.
     */
    function upgrade(address, address) external;

    /**
     * Upgrades a proxy to a new implementation and calls a function on the new implementation.
     * If UPGRADE_INTERFACE_VERSION is "5.0.0", bytes can be empty if no function should be called on the new implementation.
     */
    function upgradeAndCall(address, address, bytes memory) external payable;
}
