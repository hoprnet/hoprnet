// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import { Test } from "forge-std/Test.sol";

import { DeployNodeSafeScript } from "../../script/DeployNodeSafe.s.sol";
import { DeployAllContractsScript } from "../../script/DeployAll.s.sol";
import { ERC1820RegistryFixtureTest } from "../utils/ERC1820Registry.sol";
import { SafeSingletonFixtureTest } from "../utils/SafeSingleton.sol";

contract DeployNodeSafeScriptTest is Test, ERC1820RegistryFixtureTest, SafeSingletonFixtureTest {
    DeployAllContractsScript public deployScriptContract;
    DeployNodeSafeScript public deployNodeSafeScriptContract;

    function setUp() public override(ERC1820RegistryFixtureTest, SafeSingletonFixtureTest) {
        // super.setup()// invoke super.setup() for ERC1820RegistryFixtureTest, SafeSingletonFixtureTest separately
        ERC1820RegistryFixtureTest.setUp();
        SafeSingletonFixtureTest.setUp();

        vm.setEnv("FOUNDRY_PROFILE", "local");
        vm.setEnv("NETWORK", "anvil-localhost");
        vm.setEnv("USE_STAKING_PROXY", "true");
        deployScriptContract = new DeployAllContractsScript();
        deployScriptContract.run();
        deployNodeSafeScriptContract = new DeployNodeSafeScript();
    }

    function test_Run() public {
        // deployNodeSafeScriptContract.run();
    }
}
