// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Options} from "../src/Options.sol";
import {Core} from "../src/internal/Core.sol";

/**
 * @dev Library for managing upgradeable contracts from Forge scripts or tests.
 *
 * NOTE: Only for upgrading existing deployments using OpenZeppelin Contracts v4.
 * For new deployments, use OpenZeppelin Contracts v5 and Upgrades.sol.
 */
library Upgrades {
    /**
     * @dev Upgrades a proxy to a new implementation contract. Only supported for UUPS or transparent proxies.
     *
     * Requires that either the `referenceContract` option is set, or the new implementation contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * @param proxy Address of the proxy to upgrade
     * @param contractName Name of the new implementation contract to upgrade to, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param data Encoded call data of an arbitrary function to call during the upgrade process, or empty if no function needs to be called during the upgrade
     * @param opts Common options
     */
    function upgradeProxy(address proxy, string memory contractName, bytes memory data, Options memory opts) internal {
        Core.upgradeProxy(proxy, contractName, data, opts);
    }

    /**
     * @dev Upgrades a proxy to a new implementation contract. Only supported for UUPS or transparent proxies.
     *
     * Requires that either the `referenceContract` option is set, or the new implementation contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * @param proxy Address of the proxy to upgrade
     * @param contractName Name of the new implementation contract to upgrade to, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param data Encoded call data of an arbitrary function to call during the upgrade process, or empty if no function needs to be called during the upgrade
     */
    function upgradeProxy(address proxy, string memory contractName, bytes memory data) internal {
        Options memory opts;
        Core.upgradeProxy(proxy, contractName, data, opts);
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Upgrades a proxy to a new implementation contract. Only supported for UUPS or transparent proxies.
     *
     * Requires that either the `referenceContract` option is set, or the new implementation contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * This function provides an additional `tryCaller` parameter to test an upgrade using a specific caller address.
     * Use this if you encounter `OwnableUnauthorizedAccount` errors in your tests.
     *
     * @param proxy Address of the proxy to upgrade
     * @param contractName Name of the new implementation contract to upgrade to, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param data Encoded call data of an arbitrary function to call during the upgrade process, or empty if no function needs to be called during the upgrade
     * @param opts Common options
     * @param tryCaller Address to use as the caller of the upgrade function. This should be the address that owns the proxy or its ProxyAdmin.
     */
    function upgradeProxy(
        address proxy,
        string memory contractName,
        bytes memory data,
        Options memory opts,
        address tryCaller
    ) internal {
        Core.upgradeProxy(proxy, contractName, data, opts, tryCaller);
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Upgrades a proxy to a new implementation contract. Only supported for UUPS or transparent proxies.
     *
     * Requires that either the `referenceContract` option is set, or the new implementation contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * This function provides an additional `tryCaller` parameter to test an upgrade using a specific caller address.
     * Use this if you encounter `OwnableUnauthorizedAccount` errors in your tests.
     *
     * @param proxy Address of the proxy to upgrade
     * @param contractName Name of the new implementation contract to upgrade to, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param data Encoded call data of an arbitrary function to call during the upgrade process, or empty if no function needs to be called during the upgrade
     * @param tryCaller Address to use as the caller of the upgrade function. This should be the address that owns the proxy or its ProxyAdmin.
     */
    function upgradeProxy(address proxy, string memory contractName, bytes memory data, address tryCaller) internal {
        Options memory opts;
        Core.upgradeProxy(proxy, contractName, data, opts, tryCaller);
    }

    /**
     * @dev Upgrades a beacon to a new implementation contract.
     *
     * Requires that either the `referenceContract` option is set, or the new implementation contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * @param beacon Address of the beacon to upgrade
     * @param contractName Name of the new implementation contract to upgrade to, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param opts Common options
     */
    function upgradeBeacon(address beacon, string memory contractName, Options memory opts) internal {
        Core.upgradeBeacon(beacon, contractName, opts);
    }

    /**
     * @dev Upgrades a beacon to a new implementation contract.
     *
     * Requires that either the `referenceContract` option is set, or the new implementation contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * @param beacon Address of the beacon to upgrade
     * @param contractName Name of the new implementation contract to upgrade to, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     */
    function upgradeBeacon(address beacon, string memory contractName) internal {
        Options memory opts;
        Core.upgradeBeacon(beacon, contractName, opts);
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Upgrades a beacon to a new implementation contract.
     *
     * Requires that either the `referenceContract` option is set, or the new implementation contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * This function provides an additional `tryCaller` parameter to test an upgrade using a specific caller address.
     * Use this if you encounter `OwnableUnauthorizedAccount` errors in your tests.
     *
     * @param beacon Address of the beacon to upgrade
     * @param contractName Name of the new implementation contract to upgrade to, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param opts Common options
     * @param tryCaller Address to use as the caller of the upgrade function. This should be the address that owns the beacon.
     */
    function upgradeBeacon(
        address beacon,
        string memory contractName,
        Options memory opts,
        address tryCaller
    ) internal {
        Core.upgradeBeacon(beacon, contractName, opts, tryCaller);
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Upgrades a beacon to a new implementation contract.
     *
     * Requires that either the `referenceContract` option is set, or the new implementation contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * This function provides an additional `tryCaller` parameter to test an upgrade using a specific caller address.
     * Use this if you encounter `OwnableUnauthorizedAccount` errors in your tests.
     *
     * @param beacon Address of the beacon to upgrade
     * @param contractName Name of the new implementation contract to upgrade to, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param tryCaller Address to use as the caller of the upgrade function. This should be the address that owns the beacon.
     */
    function upgradeBeacon(address beacon, string memory contractName, address tryCaller) internal {
        Options memory opts;
        Core.upgradeBeacon(beacon, contractName, opts, tryCaller);
    }

    /**
     * @dev Validates a new implementation contract in comparison with a reference contract, but does not deploy it.
     *
     * Requires that either the `referenceContract` option is set, or the contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * @param contractName Name of the contract to validate, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param opts Common options
     */
    function validateUpgrade(string memory contractName, Options memory opts) internal {
        Core.validateUpgrade(contractName, opts);
    }

    /**
     * @dev Validates a new implementation contract in comparison with a reference contract, deploys the new implementation contract,
     * and returns its address.
     *
     * Requires that either the `referenceContract` option is set, or the contract has a `@custom:oz-upgrades-from <reference>` annotation.
     *
     * Use this method to prepare an upgrade to be run from an admin address you do not control directly or cannot use from your deployment environment.
     *
     * @param contractName Name of the contract to deploy, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param opts Common options
     * @return Address of the new implementation contract
     */
    function prepareUpgrade(string memory contractName, Options memory opts) internal returns (address) {
        return Core.prepareUpgrade(contractName, opts);
    }

    /**
     * @dev Gets the admin address of a transparent proxy from its ERC1967 admin storage slot.
     *
     * @param proxy Address of a transparent proxy
     * @return Admin address
     */
    function getAdminAddress(address proxy) internal view returns (address) {
        return Core.getAdminAddress(proxy);
    }

    /**
     * @dev Gets the implementation address of a transparent or UUPS proxy from its ERC1967 implementation storage slot.
     *
     * @param proxy Address of a transparent or UUPS proxy
     * @return Implementation address
     */
    function getImplementationAddress(address proxy) internal view returns (address) {
        return Core.getImplementationAddress(proxy);
    }

    /**
     * @dev Gets the beacon address of a beacon proxy from its ERC1967 beacon storage slot.
     *
     * @param proxy Address of a beacon proxy
     * @return Beacon address
     */
    function getBeaconAddress(address proxy) internal view returns (address) {
        return Core.getBeaconAddress(proxy);
    }
}

/**
 * @dev Library for managing upgradeable contracts from Forge tests, without validations.
 *
 * Can be used with `forge coverage`. Requires implementation contracts to be instantiated first.
 * Does not require `--ffi` and does not require a clean compilation before each run.
 *
 * Not supported for OpenZeppelin Defender deployments.
 *
 * WARNING: Not recommended for use in Forge scripts.
 * `UnsafeUpgrades` does not validate whether your contracts are upgrade safe or whether new implementations are compatible with previous ones.
 * Use `Upgrades` if you want validations to be run.
 *
 * NOTE: Only for upgrading existing deployments using OpenZeppelin Contracts v4.
 * For new deployments, use OpenZeppelin Contracts v5 and Upgrades.sol.
 */
library UnsafeUpgrades {
    /**
     * @dev Upgrades a proxy to a new implementation contract address. Only supported for UUPS or transparent proxies.
     *
     * @param proxy Address of the proxy to upgrade
     * @param newImpl Address of the new implementation contract to upgrade to
     * @param data Encoded call data of an arbitrary function to call during the upgrade process, or empty if no function needs to be called during the upgrade
     */
    function upgradeProxy(address proxy, address newImpl, bytes memory data) internal {
        Core.upgradeProxyTo(proxy, newImpl, data);
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Upgrades a proxy to a new implementation contract address. Only supported for UUPS or transparent proxies.
     *
     * This function provides an additional `tryCaller` parameter to test an upgrade using a specific caller address.
     * Use this if you encounter `OwnableUnauthorizedAccount` errors in your tests.
     *
     * @param proxy Address of the proxy to upgrade
     * @param newImpl Address of the new implementation contract to upgrade to
     * @param data Encoded call data of an arbitrary function to call during the upgrade process, or empty if no function needs to be called during the upgrade
     * @param tryCaller Address to use as the caller of the upgrade function. This should be the address that owns the proxy or its ProxyAdmin.
     */
    function upgradeProxy(address proxy, address newImpl, bytes memory data, address tryCaller) internal {
        Core.upgradeProxyTo(proxy, newImpl, data, tryCaller);
    }

    /**
     * @dev Upgrades a beacon to a new implementation contract address.
     *
     * @param beacon Address of the beacon to upgrade
     * @param newImpl Address of the new implementation contract to upgrade to
     */
    function upgradeBeacon(address beacon, address newImpl) internal {
        Core.upgradeBeaconTo(beacon, newImpl);
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Upgrades a beacon to a new implementation contract address.
     *
     * This function provides an additional `tryCaller` parameter to test an upgrade using a specific caller address.
     * Use this if you encounter `OwnableUnauthorizedAccount` errors in your tests.
     *
     * @param beacon Address of the beacon to upgrade
     * @param newImpl Address of the new implementation contract to upgrade to
     * @param tryCaller Address to use as the caller of the upgrade function. This should be the address that owns the beacon.
     */
    function upgradeBeacon(address beacon, address newImpl, address tryCaller) internal {
        Core.upgradeBeaconTo(beacon, newImpl, tryCaller);
    }

    /**
     * @dev Gets the admin address of a transparent proxy from its ERC1967 admin storage slot.
     *
     * @param proxy Address of a transparent proxy
     * @return Admin address
     */
    function getAdminAddress(address proxy) internal view returns (address) {
        return Core.getAdminAddress(proxy);
    }

    /**
     * @dev Gets the implementation address of a transparent or UUPS proxy from its ERC1967 implementation storage slot.
     *
     * @param proxy Address of a transparent or UUPS proxy
     * @return Implementation address
     */
    function getImplementationAddress(address proxy) internal view returns (address) {
        return Core.getImplementationAddress(proxy);
    }

    /**
     * @dev Gets the beacon address of a beacon proxy from its ERC1967 beacon storage slot.
     *
     * @param proxy Address of a beacon proxy
     * @return Beacon address
     */
    function getBeaconAddress(address proxy) internal view returns (address) {
        return Core.getBeaconAddress(proxy);
    }
}
