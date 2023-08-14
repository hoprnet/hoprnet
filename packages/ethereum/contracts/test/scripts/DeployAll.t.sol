// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "../../script/DeployAll.s.sol";
import "../utils/ERC1820Registry.sol";
import "forge-std/Test.sol";

contract DeployAllTest is Test, ERC1820RegistryFixtureTest {
    DeployAllContractsScript public deployScriptContract;

    function setUp() public override {
        super.setUp();
        deployScriptContract = new DeployAllContractsScript();
    }

    function test_Run() public {
        vm.setEnv("FOUNDRY_PROFILE", "local");
        vm.setEnv("NETWORK", "anvil-localhost");
        deployScriptContract.run();
    }
}
