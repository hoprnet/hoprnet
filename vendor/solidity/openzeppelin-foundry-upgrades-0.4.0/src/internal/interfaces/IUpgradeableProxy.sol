// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUpgradeableProxy {
    /**
     * Upgrades the proxy to a new implementation without calling a function on the new implementation.
     */
    function upgradeTo(address) external;

    /**
     * Upgrades the proxy to a new implementation and calls a function on the new implementation.
     * If UPGRADE_INTERFACE_VERSION is "5.0.0", bytes can be empty if no function should be called on the new implementation.
     */
    function upgradeToAndCall(address, bytes memory) external payable;
}
