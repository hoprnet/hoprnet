// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "./utils/NetworkConfig.s.sol";
import "./utils/SafeSuiteLib.sol";
import "../test/utils/SafeSingleton.sol";

contract SafeSuiteSetupScript is Script, NetworkConfig, SafeSingletonFixtureTest {
    function run() external {
        // 1. Network check
        // get environment of the script
        getNetwork();
        // read records of deployed files
        readCurrentNetwork();

        // Halt if Safe Singleton has not been deployed.
        mustHaveSingletonContract();
        emit log_string(string(abi.encodePacked("Deploying in ", currentNetworkId)));

        // 2. Get deployer private key
        // Set to default when it's in development environment (uint for 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
        uint256 deployerPrivateKey = currentEnvironmentType == EnvironmentType.LOCAL
            ? 77814517325470205911140941194401928579557062014761831930645393041380819009408
            : vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);
        vm.startBroadcast(deployerPrivateKey);

        // 3. deploy safe suites
        deployEntireSafeSuite();

        // broadcast transaction bundle
        vm.stopBroadcast();
    }
}
