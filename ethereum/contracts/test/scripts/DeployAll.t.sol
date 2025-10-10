// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import { Test } from "forge-std/Test.sol";

import { DeployAllContractsScript } from "../../script/DeployAll.s.sol";
import { ERC1820RegistryFixtureTest } from "../utils/ERC1820Registry.sol";
import { SafeSingletonFixtureTest } from "../utils/SafeSingleton.sol";

contract DeployAllTest is Test, ERC1820RegistryFixtureTest, SafeSingletonFixtureTest {
    DeployAllContractsScript public deployScriptContract;

    function setUp() public override(ERC1820RegistryFixtureTest, SafeSingletonFixtureTest) {
        // invoke super.setup() for ERC1820RegistryFixtureTest, SafeSingletonFixtureTest separately
        ERC1820RegistryFixtureTest.setUp();
        SafeSingletonFixtureTest.setUp();
    }

    function test_Run() public {
        deployScriptContract = new DeployAllContractsScript();
        vm.setEnv("FOUNDRY_PROFILE", "local");
        vm.setEnv("NETWORK", "anvil-localhost");
        vm.setEnv("USE_STAKING_PROXY", "true");
        deployScriptContract.run();
    }
}
