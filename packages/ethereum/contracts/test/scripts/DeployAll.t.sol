// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import '../../script/DeployAll.s.sol';
import 'forge-std/Test.sol';

contract DeployAllTest is Test {
    DeployAllContractsScript public deployScriptContract;

    function setUp() public {
        deployScriptContract = new DeployAllContractsScript();
    }

    function test_Run() public {
        vm.setEnv("FOUNDRY_PROFILE", "development");
        vm.setEnv("NETWORK", "anvil-localhost");
        deployScriptContract.run();
    }
}