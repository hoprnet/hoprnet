// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "./utils/ERC1820Registry.sol";
import "forge-std/Test.sol";

contract ERC1820RegistryTest is Test, ERC1820RegistryFixtureTest {
    function setUp() public virtual override {
        super.setUp();
    }

    function testERC1820IsDeployed() public {
        // ERC1820_REGISTRY_CONTRACT contains the exact deployed code
        assertEq0(ERC1820_REGISTRY_ADDRESS.code, ERC1820_REGISTRY_DEPLOYED_CODE);
    }
}
