// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import 'forge-std/Script.sol';
import './utils/NetworkConfig.s.sol';

contract DeployNodeManagementScript is
  Script,
  NetworkConfig
{

  function run() external {
    // 1. Network check
    // get envirionment of the script
    getNetwork();
    // read records of deployed files
    readCurrentNetwork();

    // 2. Get deployer private key
    uint256 deployerPrivateKey = vm.envUint('DEPLOYER_PRIVATE_KEY');
    address deployerAddress = vm.addr(deployerPrivateKey);
    vm.startBroadcast(deployerPrivateKey);

    // 3. Deploy
    // 3.1. Singleton of HoprNodeManagementModule
    // Only deploy Token contract when no deployed one is detected.
    // E.g. always in local envirionment, or should a new token contract be introduced in staging/production.
    if (
      currentEnvironmentType == EnvironmentType.LOCAL ||
      !isValidAddress(currentNetworkDetail.moduleImplementationAddress)
    ) {
      // deploy HoprNodeManagementModule contract
      currentNetworkDetail.moduleImplementationAddress = deployCode('HoprNodeManagementModule.sol');
    }

    // broadcast transaction bundle
    vm.stopBroadcast();

    // write to file
    writeCurrentNetwork();
  }
}
