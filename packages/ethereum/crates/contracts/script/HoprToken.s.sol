// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Script.sol";
import "../src/HoprToken.sol";
import "../test/utils/Deploy.sol";
import "./utils/EnvironmentConfig.s.sol";

contract DeployHoprTokenScript is Script, EnvironmentConfig, ERC1820RegistryFixture {
    function run() external {
        // get envirionment of the script
        getEnvrionment();

        // Only deploy Token contract in development envirionment
        if (currentEnvironmentType != EnvironmentType.DEVELOPMENT) {
            return;
        }
        
        // get deployer private key
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);
        // deploy token contract
        HoprToken hoprToken = new HoprToken();
        // write to file
        vm.stopBroadcast();

        // TODO: test
        readCurrentEnvironment();

        // FIXME: to write to a json file
        vm.writeLine("test.txt", string(abi.encodePacked("token_contract_address: ", vm.toString(address(hoprToken)))));
    }
}
