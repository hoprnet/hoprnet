// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "forge-std/Test.sol";
import "./utils/EnvironmentConfig.s.sol";

/**
 * @dev script to interact with Network Registry for both selfRegister and onlyOwner register
 * Private key of the caller must be saved under the envrionment variable `PRIVATE_KEY`
 * Wrapper of NetworkRegistery contract with detection of contract address per environment_name/environment_type
 */
contract RegisterScript is Test, EnvironmentConfig {
    using stdJson for string;

    function getEnvironmentAndMsgSender() private {
        // 1. Environment check
        // get envirionment of the script
        getEnvrionment();
        // read records of deployed files
        readCurrentEnvironment();

        // 2. Get private key of caller
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);
        vm.startBroadcast(deployerPrivateKey);
    }

    function selfRegisterNodes(string[] calldata peerIds) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. call hoprNetworkRegistry.selfRegister(peerIds);
        (bool successSelfRegister, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("selfRegister(string[])", peerIds));
        if (!successSelfRegister) {
            emit log_string("Cannot register peers");
            revert("Cannot register peers");
        }
        vm.stopBroadcast();
    }

    function selfDeregisterNodes(string[] calldata peerIds) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. call hoprNetworkRegistry.selfDeregister(peerIds);
        (bool successSelfDeregister, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("selfDeregister(string[])", peerIds));
        if (!successSelfDeregister) {
            emit log_string("Cannot deregister peers");
            revert("Cannot deregister peers");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev function called by the owner
     */
    function registerNodes(address[] calldata stakingAddresses, string[] calldata peerIds) external {
        require(stakingAddresses.length == peerIds.length, "Input lengths are different");

        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. owner registers nodes, depending on the envirionment 
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT) {
            // call register accounts on HoprDummyProxyForNetworkRegistry
            (bool successRegisterNodesOnDummyProxy, ) = currentEnvironmentDetail.networkRegistryProxyContractAddress.call(abi.encodeWithSignature("ownerBatchAddAccounts(address[])", stakingAddresses));
            if (!successRegisterNodesOnDummyProxy) {
                emit log_string("Cannot add stakingAddresses on to the dummy proxy.");
                revert("Cannot add stakingAddresses on to the dummy proxy.");
            }
        }
        // actual register nodes
        (bool successRegisterNodes, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("ownerRegister(address[],string[])", stakingAddresses, peerIds));
        if (!successRegisterNodes) {
            emit log_string("Cannot register nodes as an owner");
            revert("Cannot register nodes as an owner");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev function called by the owner
     */
    function deregisterNodes(address[] calldata stakingAddresses, string[] calldata peerIds) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. owner registers nodes, depending on the envirionment 
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT) {
            // call deregister accounts on HoprDummyProxyForNetworkRegistry
            (bool successDeregisterNodesOnDummyProxy, ) = currentEnvironmentDetail.networkRegistryProxyContractAddress.call(abi.encodeWithSignature("ownerBatchRemoveAccounts(address[])", stakingAddresses));
            if (!successDeregisterNodesOnDummyProxy) {
                emit log_string("Cannot remove stakingAddresses from the dummy proxy.");
                revert("Cannot remove stakingAddresses from the dummy proxy.");
            }
        }
        // actual deregister nodes
        (bool successDeregisterNodes, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("ownerDeregister(string[])", peerIds));
        if (!successDeregisterNodes) {
            emit log_string("Cannot rdeegister nodes as an owner");
            revert("Cannot deregister nodes as an owner");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev function called by the owner
     */
    function disableNetworkRegistry() external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. check if current NR is enabled.
        (bool successReadEnabled, bytes memory returndataReadEnabled) = currentEnvironmentDetail.networkRegistryContractAddress.staticcall(abi.encodeWithSignature("enabled()"));
        if (!successReadEnabled) {
            revert("Cannot read enabled from network registry contract.");
        }
        bool isEnabled = abi.decode(returndataReadEnabled, (bool));

        // 3. disable if needed
        if (isEnabled) {
            (bool successDisableNetworkRegistry, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("disableRegistry()"));
            if (!successDisableNetworkRegistry) {
                emit log_string("Cannot disable network registery as an owner");
                revert("Cannotdisable network registery as an owner");
            }
            vm.stopBroadcast();
        }
    }

    /**
     * @dev function called by the owner
     */
    function enableNetworkRegistry() external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. check if current NR is enabled.
        (bool successReadEnabled, bytes memory returndataReadEnabled) = currentEnvironmentDetail.networkRegistryContractAddress.staticcall(abi.encodeWithSignature("enabled()"));
        if (!successReadEnabled) {
            revert("Cannot read enabled from network registry contract.");
        }
        bool isEnabled = abi.decode(returndataReadEnabled, (bool));

        // 3. enable if needed
        if (!isEnabled) {
            (bool successEnableNetworkRegistry, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("enableRegistry()"));
            if (!successEnableNetworkRegistry) {
                emit log_string("Cannot enable network registery as an owner");
                revert("Cannot enable network registery as an owner");
            }
            vm.stopBroadcast();
        }
    }

    /**
     * @dev function called by the owner
     */
    function forceEligibilityUpdate(address[] calldata stakingAddresses, bool[] calldata eligibility) external {
        require(stakingAddresses.length == eligibility.length, "Input lengths are different");

        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. update emit EligibilityUpdate events by the owner
        (bool successForceEligibilityUpdate, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("ownerForceEligibility(address[],bool[])", stakingAddresses, eligibility));
        if (!successForceEligibilityUpdate) {
            emit log_string("Cannot force update eligibility as an owner");
            revert("Cannot force update eligibility as an owner");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev function called by the owner
     */
    function syncEligibility(string[] calldata peerIds) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. sync peers eligibility according to the latest requirement of its current state
        (bool successSyncEligibility, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("sync(string[])", peerIds));
        if (!successSyncEligibility) {
            emit log_string("Cannot sync eligibility as an owner");
            revert("Cannot sync eligibility as an owner");
        }
        vm.stopBroadcast();
    }
}
