// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "./utils/NetworkConfig.s.sol";
import "./DeployAll.s.sol";
import "../src/utils/TargetUtils.sol";
import "forge-std/Test.sol";

contract DeployNodeSafeScript is Script, Test, NetworkConfig {
    using TargetUtils for address;

    function run() external {
        // 1. Network check
        // get environment of the script
        getNetwork();
        // read records of deployed files
        readCurrentNetwork();

        emit log_string(string(abi.encodePacked("Deploying in ", currentNetworkId)));

        // 2. Get deployer internal key.
        // Set to default when it's in development environment (uint for
        // 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
        uint256 deployerPrivateKey = currentEnvironmentType == EnvironmentType.LOCAL
            ? 77_814_517_325_470_205_911_140_941_194_401_928_579_557_062_014_761_831_930_645_393_041_380_819_009_408
            : vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);
        emit log_named_address("deployerAddress", deployerAddress);
        vm.startBroadcast(deployerPrivateKey);

        // 3. TODO: Check if following addresses are deployed correctly
        // 3.1 HoprNodeStakeFactory
        // 3.2 HoprNodeManagementModule singleton
        // 3.3 HoprNodeSafeRegistry
        // 3.4 HoprToken
        // 3.4 HoprChannels

        address[] memory admins = new address[](2);
        admins[0] = deployerAddress;
        admins[1] = COMM_MULTISIG_ADDRESS;

        uint256 nonce = vm.getNonce(deployerAddress);

        // build default permissions
        CapabilityPermission[] memory capabilityPermissions = new CapabilityPermission[](9);
        for (uint256 i = 0; i < capabilityPermissions.length; i++) {
            capabilityPermissions[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        capabilityPermissions[8] = CapabilityPermission.SPECIFIC_FALLBACK_BLOCK;

        Target defaultChannelsTarget = TargetUtils.encodeDefaultPermissions(
            currentNetworkDetail.addresses.channelsContractAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.SPECIFIC_FALLBACK_BLOCK,
            capabilityPermissions
        );

        bytes memory cloneCallData = abi.encodeWithSignature(
            "clone(address,address[],uint256,bytes32)",
            address(currentNetworkDetail.addresses.moduleImplementationAddress),
            admins,
            nonce,
            bytes32(Target.unwrap(defaultChannelsTarget))
        );
        emit log_named_bytes32("defaultChannelsTarget", bytes32(Target.unwrap(defaultChannelsTarget)));

        (bool success, bytes memory returnedData) =
            currentNetworkDetail.addresses.nodeStakeV2FactoryAddress.call(cloneCallData);
        require(success, "call node stake factory must succeed");

        (address module, address safe) = abi.decode(returnedData, (address, address));
        emit log_named_address("safe address", safe);
        emit log_named_address("module address", module);

        // deploy
        // broadcast transaction bundle
        vm.stopBroadcast();

        // write to file
        writeCurrentNetwork();
    }
}
