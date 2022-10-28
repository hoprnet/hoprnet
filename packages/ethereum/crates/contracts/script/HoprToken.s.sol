// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Script.sol";
import "../src/HoprToken.sol";

contract DeployHoprTokenScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        HoprToken hoprToken = new HoprToken();

        vm.stopBroadcast();
    }
}