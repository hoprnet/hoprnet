// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { ERC1820RegistryFixtureTest } from "./utils/ERC1820Registry.sol";
import { Test } from "forge-std/Test.sol";

contract ERC1820RegistryTest is Test, ERC1820RegistryFixtureTest {
    function setUp() public virtual override {
        super.setUp();
    }

    function testERC1820IsDeployed() public {
        // ERC1820_REGISTRY_CONTRACT contains the exact deployed code
        assertEq0(ERC1820_REGISTRY_ADDRESS.code, ERC1820_REGISTRY_DEPLOYED_CODE);
    }
}
