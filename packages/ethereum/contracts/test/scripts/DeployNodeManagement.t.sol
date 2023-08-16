// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "../../script/DeployNodeSafe.s.sol";
import "../../script/DeployAll.s.sol";
import "../utils/ERC1820Registry.sol";
import "forge-std/Test.sol";

contract DeployNodeSafeScriptTest is Test, ERC1820RegistryFixtureTest {
    DeployAllContractsScript public deployScriptContract;
    DeployNodeSafeScript public deployNodeSafeScriptContract;

    function setUp() public override {
        super.setUp();
        vm.setEnv("FOUNDRY_PROFILE", "local");
        vm.setEnv("NETWORK", "anvil-localhost");
        deployScriptContract = new DeployAllContractsScript();
        deployScriptContract.run();
        deployNodeSafeScriptContract = new DeployNodeSafeScript();
    }

    function test_Run() public {
        // deployNodeSafeScriptContract.run();
    }
}
