// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "./utils/NetworkConfig.s.sol";
import "../src/utils/SafeSuiteLib.sol";
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
        // Set to default when it's in development environment (uint for
        // 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
        uint256 deployerPrivateKey = currentEnvironmentType == EnvironmentType.LOCAL
            ? 77_814_517_325_470_205_911_140_941_194_401_928_579_557_062_014_761_831_930_645_393_041_380_819_009_408
            : vm.envUint("DEPLOYER_PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        // 3. deploy safe suites
        deployEntireSafeSuite();

        // broadcast transaction bundle
        vm.stopBroadcast();
    }
}
