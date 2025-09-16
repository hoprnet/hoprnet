// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Vm} from "forge-std/Vm.sol";
import {console} from "forge-std/console.sol";

import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

import {Options} from "../Options.sol";
import {Versions} from "./Versions.sol";
import {Utils} from "./Utils.sol";
import {DefenderDeploy} from "./DefenderDeploy.sol";

import {IUpgradeableProxy} from "./interfaces/IUpgradeableProxy.sol";
import {IProxyAdmin} from "./interfaces/IProxyAdmin.sol";
import {IUpgradeableBeacon} from "./interfaces/IUpgradeableBeacon.sol";

/**
 * @dev Internal helper methods to validate/deploy implementations and perform upgrades.
 *
 * WARNING: DO NOT USE DIRECTLY. Use Upgrades.sol, LegacyUpgrades.sol or Defender.sol instead.
 */
library Core {
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
        address newImpl = prepareUpgrade(contractName, opts);
        upgradeProxyTo(proxy, newImpl, data);
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
    ) internal tryPrank(tryCaller) {
        upgradeProxy(proxy, contractName, data, opts);
    }

    using Strings for *;

    /**
     * @dev Upgrades a proxy to a new implementation contract. Only supported for UUPS or transparent proxies.
     *
     * @param proxy Address of the proxy to upgrade
     * @param newImpl Address of the new implementation contract to upgrade to
     * @param data Encoded call data of an arbitrary function to call during the upgrade process, or empty if no function needs to be called during the upgrade
     */
    function upgradeProxyTo(address proxy, address newImpl, bytes memory data) internal {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        bytes32 adminSlot = vm.load(proxy, ADMIN_SLOT);
        if (adminSlot == bytes32(0)) {
            string memory upgradeInterfaceVersion = getUpgradeInterfaceVersion(proxy);
            if (upgradeInterfaceVersion.equal("5.0.0") || data.length > 0) {
                IUpgradeableProxy(proxy).upgradeToAndCall(newImpl, data);
            } else {
                IUpgradeableProxy(proxy).upgradeTo(newImpl);
            }
        } else {
            address admin = address(uint160(uint256(adminSlot)));
            string memory upgradeInterfaceVersion = getUpgradeInterfaceVersion(admin);
            if (upgradeInterfaceVersion.equal("5.0.0") || data.length > 0) {
                IProxyAdmin(admin).upgradeAndCall(proxy, newImpl, data);
            } else {
                IProxyAdmin(admin).upgrade(proxy, newImpl);
            }
        }
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Upgrades a proxy to a new implementation contract. Only supported for UUPS or transparent proxies.
     *
     * This function provides an additional `tryCaller` parameter to test an upgrade using a specific caller address.
     * Use this if you encounter `OwnableUnauthorizedAccount` errors in your tests.
     *
     * @param proxy Address of the proxy to upgrade
     * @param newImpl Address of the new implementation contract to upgrade to
     * @param data Encoded call data of an arbitrary function to call during the upgrade process, or empty if no function needs to be called during the upgrade
     * @param tryCaller Address to use as the caller of the upgrade function. This should be the address that owns the proxy or its ProxyAdmin.
     */
    function upgradeProxyTo(
        address proxy,
        address newImpl,
        bytes memory data,
        address tryCaller
    ) internal tryPrank(tryCaller) {
        upgradeProxyTo(proxy, newImpl, data);
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
        address newImpl = prepareUpgrade(contractName, opts);
        upgradeBeaconTo(beacon, newImpl);
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
    ) internal tryPrank(tryCaller) {
        upgradeBeacon(beacon, contractName, opts);
    }

    /**
     * @dev Upgrades a beacon to a new implementation contract address.
     *
     * @param beacon Address of the beacon to upgrade
     * @param newImpl Address of the new implementation contract to upgrade to
     */
    function upgradeBeaconTo(address beacon, address newImpl) internal {
        IUpgradeableBeacon(beacon).upgradeTo(newImpl);
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Upgrades a beacon to a new implementation contract.
     *
     * This function provides an additional `tryCaller` parameter to test an upgrade using a specific caller address.
     * Use this if you encounter `OwnableUnauthorizedAccount` errors in your tests.
     *
     * @param beacon Address of the beacon to upgrade
     * @param newImpl Address of the new implementation contract to upgrade to
     * @param tryCaller Address to use as the caller of the upgrade function. This should be the address that owns the beacon.
     */
    function upgradeBeaconTo(address beacon, address newImpl, address tryCaller) internal tryPrank(tryCaller) {
        upgradeBeaconTo(beacon, newImpl);
    }

    /**
     * @dev Validates an implementation contract, but does not deploy it.
     *
     * @param contractName Name of the contract to validate, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param opts Common options
     */
    function validateImplementation(string memory contractName, Options memory opts) internal {
        _validate(contractName, opts, false);
    }

    /**
     * @dev Validates and deploys an implementation contract, and returns its address.
     *
     * @param contractName Name of the contract to deploy, e.g. "MyContract.sol" or "MyContract.sol:MyContract" or artifact path relative to the project root directory
     * @param opts Common options
     * @return Address of the implementation contract
     */
    function deployImplementation(string memory contractName, Options memory opts) internal returns (address) {
        validateImplementation(contractName, opts);
        return deploy(contractName, opts.constructorData, opts);
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
        _validate(contractName, opts, true);
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
        validateUpgrade(contractName, opts);
        return deploy(contractName, opts.constructorData, opts);
    }

    /**
     * @dev Gets the admin address of a transparent proxy from its ERC1967 admin storage slot.
     *
     * @param proxy Address of a transparent proxy
     * @return Admin address
     */
    function getAdminAddress(address proxy) internal view returns (address) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        bytes32 adminSlot = vm.load(proxy, ADMIN_SLOT);
        return address(uint160(uint256(adminSlot)));
    }

    /**
     * @dev Gets the implementation address of a transparent or UUPS proxy from its ERC1967 implementation storage slot.
     *
     * @param proxy Address of a transparent or UUPS proxy
     * @return Implementation address
     */
    function getImplementationAddress(address proxy) internal view returns (address) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        bytes32 implSlot = vm.load(proxy, IMPLEMENTATION_SLOT);
        return address(uint160(uint256(implSlot)));
    }

    /**
     * @dev Gets the beacon address of a beacon proxy from its ERC1967 beacon storage slot.
     *
     * @param proxy Address of a beacon proxy
     * @return Beacon address
     */
    function getBeaconAddress(address proxy) internal view returns (address) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        bytes32 beaconSlot = vm.load(proxy, BEACON_SLOT);
        return address(uint160(uint256(beaconSlot)));
    }

    /**
     * @notice For tests only. If broadcasting in scripts, use the `--sender <ADDRESS>` option with `forge script` instead.
     *
     * @dev Runs a function as a prank, or just runs the function normally if the prank could not be started.
     */
    modifier tryPrank(address deployer) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);

        try vm.startPrank(deployer) {
            _;
            vm.stopPrank();
        } catch {
            _;
        }
    }

    /**
     * @dev Storage slot with the address of the implementation.
     * This is the keccak-256 hash of "eip1967.proxy.implementation" subtracted by 1.
     */
    bytes32 private constant IMPLEMENTATION_SLOT = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;

    /**
     * @dev Storage slot with the admin of the proxy.
     * This is the keccak-256 hash of "eip1967.proxy.admin" subtracted by 1.
     */
    bytes32 private constant ADMIN_SLOT = 0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103;

    /**
     * @dev Storage slot with the UpgradeableBeacon contract which defines the implementation for the proxy.
     * This is the keccak-256 hash of "eip1967.proxy.beacon" subtracted by 1.
     */
    bytes32 private constant BEACON_SLOT = 0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50;

    /**
     * @dev Gets the upgrade interface version string from a proxy or admin contract using the `UPGRADE_INTERFACE_VERSION()` getter.
     * If the contract does not have the getter or the return data does not look like a string, this function returns an empty string.
     */
    function getUpgradeInterfaceVersion(address addr) internal view returns (string memory) {
        // Use staticcall to prevent forge from broadcasting it, and to ensure no state changes
        (bool success, bytes memory returndata) = addr.staticcall(
            abi.encodeWithSignature("UPGRADE_INTERFACE_VERSION()")
        );
        if (success && returndata.length > 32) {
            return abi.decode(returndata, (string));
        } else {
            return "";
        }
    }

    /**
     * @dev Infers whether the address might be a ProxyAdmin contract.
     */
    function inferProxyAdmin(address addr) internal view returns (bool) {
        return _hasOwner(addr);
    }

    /**
     * @dev Returns true if the address is a contract with an `owner()` function that is not state-changing and returns something that might be an address,
     * otherwise returns false.
     */
    function _hasOwner(address addr) private view returns (bool) {
        // Use staticcall to prevent forge from broadcasting it, and to ensure no state changes
        (bool success, bytes memory returndata) = addr.staticcall(abi.encodeWithSignature("owner()"));
        return (success && returndata.length == 32);
    }

    function _validate(string memory contractName, Options memory opts, bool requireReference) private {
        if (opts.unsafeSkipAllChecks) {
            return;
        }

        string[] memory inputs = buildValidateCommand(contractName, opts, requireReference);
        Vm.FfiResult memory result = Utils.runAsBashCommand(inputs);
        string memory stdout = string(result.stdout);

        // CLI validate command uses exit code to indicate if the validation passed or failed.
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);
        if (result.exitCode == 0) {
            // As an extra precaution, we also check stdout for "SUCCESS" to ensure it actually ran.
            if (vm.contains(stdout, "SUCCESS")) {
                if (result.stderr.length > 0) {
                    // Prints warnings from stderr
                    console.log(string(result.stderr));
                }
                return;
            } else {
                revert(string(abi.encodePacked("Failed to run upgrade safety validation: ", stdout)));
            }
        } else {
            if (vm.contains(stdout, "FAILED")) {
                if (result.stderr.length > 0) {
                    // Prints warnings from stderr
                    console.log(string(result.stderr));
                }
                // Validations ran but some contracts were not upgrade safe
                revert(string(abi.encodePacked("Upgrade safety validation failed:\n", stdout)));
            } else {
                // Validations failed to run
                revert(string(abi.encodePacked("Failed to run upgrade safety validation: ", string(result.stderr))));
            }
        }
    }

    function buildValidateCommand(
        string memory contractName,
        Options memory opts,
        bool requireReference
    ) internal view returns (string[] memory) {
        string memory outDir = Utils.getOutDir();

        string[] memory inputBuilder = new string[](2 ** 16);

        uint16 i = 0;

        inputBuilder[i++] = "npx";
        inputBuilder[i++] = string(abi.encodePacked("@openzeppelin/upgrades-core@", Versions.UPGRADES_CORE));
        inputBuilder[i++] = "validate";
        inputBuilder[i++] = string(abi.encodePacked(outDir, "/build-info"));
        inputBuilder[i++] = "--contract";
        inputBuilder[i++] = Utils.getFullyQualifiedName(contractName, outDir);

        bool hasReferenceContract = bytes(opts.referenceContract).length != 0;
        bool hasReferenceBuildInfoDir = bytes(opts.referenceBuildInfoDir).length != 0;

        if (hasReferenceContract) {
            string memory referenceArg = hasReferenceBuildInfoDir
                ? opts.referenceContract
                : Utils.getFullyQualifiedName(opts.referenceContract, outDir);
            inputBuilder[i++] = "--reference";
            inputBuilder[i++] = string(abi.encodePacked('"', referenceArg, '"'));
        }

        if (hasReferenceBuildInfoDir) {
            inputBuilder[i++] = "--referenceBuildInfoDirs";
            inputBuilder[i++] = string(abi.encodePacked('"', opts.referenceBuildInfoDir, '"'));
        }

        for (uint8 j = 0; j < opts.exclude.length; j++) {
            string memory exclude = opts.exclude[j];
            if (bytes(exclude).length != 0) {
                inputBuilder[i++] = "--exclude";
                inputBuilder[i++] = string(abi.encodePacked('"', exclude, '"'));
            }
        }

        if (opts.unsafeSkipStorageCheck) {
            inputBuilder[i++] = "--unsafeSkipStorageCheck";
        } else if (requireReference) {
            inputBuilder[i++] = "--requireReference";
        }

        if (bytes(opts.unsafeAllow).length != 0) {
            inputBuilder[i++] = "--unsafeAllow";
            inputBuilder[i++] = opts.unsafeAllow;
        }

        if (opts.unsafeAllowRenames) {
            inputBuilder[i++] = "--unsafeAllowRenames";
        }

        // Create a copy of inputs but with the correct length
        string[] memory inputs = new string[](i);
        for (uint16 j = 0; j < i; j++) {
            inputs[j] = inputBuilder[j];
        }

        return inputs;
    }

    function deploy(
        string memory contractName,
        bytes memory constructorData,
        Options memory opts
    ) internal returns (address) {
        if (opts.defender.useDefenderDeploy) {
            return DefenderDeploy.deploy(contractName, constructorData, opts.defender);
        } else {
            return _deploy(contractName, constructorData);
        }
    }

    function _deploy(string memory contractName, bytes memory constructorData) private returns (address) {
        bytes memory creationCode = Vm(Utils.CHEATCODE_ADDRESS).getCode(contractName);
        address deployedAddress = _deployFromBytecode(abi.encodePacked(creationCode, constructorData));
        if (deployedAddress == address(0)) {
            revert(
                string(
                    abi.encodePacked(
                        "Failed to deploy contract ",
                        contractName,
                        ' using constructor data "',
                        string(constructorData),
                        '"'
                    )
                )
            );
        }
        return deployedAddress;
    }

    function _deployFromBytecode(bytes memory bytecode) private returns (address) {
        address addr;
        /// @solidity memory-safe-assembly
        assembly {
            addr := create(0, add(bytecode, 32), mload(bytecode))
        }
        return addr;
    }
}
