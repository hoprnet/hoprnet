// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Script.sol";
import "../src/HoprChannels.sol";
import "./utils/EnvironmentConfig.s.sol";

contract DeployHoprChannelsScript is Script, EnvironmentConfig {

    uint256 public shortDuration = 15 * 1e3; // 15 seconds in ms
    uint256 public longDuration = 5 * 60 * 1e3; // 5 minutes in ms

    function run() external {
        // get envirionment of the script
        getEnvrionment();

        // Detect if a HoprChannels contract has been deployed for the "environmentName". If not, deploy
        

        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);
        // HoprChannels hoprChannels = new HoprChannels();
        vm.stopBroadcast();
    }
}