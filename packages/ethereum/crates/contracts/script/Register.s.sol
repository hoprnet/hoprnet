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

    function selfRegisterNodes(string[] memory peerIds) external {
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

    function selfDeregisterNodes(string[] memory peerIds) external {
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

    function registerNodes(address[] memory stakingAddresses, string[] memory peerIds) external {
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
}
