// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUpgradeableBeacon {
    /**
     * Upgrades the beacon to a new implementation.
     */
    function upgradeTo(address) external;
}
