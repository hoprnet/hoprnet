// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import 'forge-std/Script.sol';
import './utils/NetworkConfig.s.sol';
import './utils/SafeSuiteLib.sol';
import '../test/utils/SafeSingleton.sol';

contract SafeSuiteSetupScript is Script, NetworkConfig, SafeSingletonFixtureTest {
  function run() external {
    // 1. Network check
    // get envirionment of the script
    getNetwork();
    // read records of deployed files
    readCurrentNetwork();

    // Halt if Safe Singleton has not been deployed.
    mustHaveSingletonContract();
    emit log_string(string(abi.encodePacked('Deploying in ', currentNetworkId)));

    // 2. Get deployer private key
    uint256 deployerPrivateKey = vm.envUint('DEPLOYER_PRIVATE_KEY');
    address deployerAddress = vm.addr(deployerPrivateKey);
    vm.startBroadcast(deployerPrivateKey);

    // 3. deploy safe suites
    deployEntireSafeSuite();

    // broadcast transaction bundle
    vm.stopBroadcast();
  }
}
