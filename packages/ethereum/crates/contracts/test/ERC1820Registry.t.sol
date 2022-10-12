// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./utils/Deploy.sol";
import "forge-std/Test.sol";

contract ERC1820RegistryTest is Test, ERC1820RegistryFixture {
    function setUp() public virtual override {
        super.setUp();
    }

    function testERC1820IsDeployed() public {
        // ERC1820_REGISTRY_CONTRACT contains the exact deployed code
        assertEq0(address(ERC1820_REGISTRY_CONTRACT).code, ERC1820_REGISTRY_DEPLOYED_CODE);
    }
}
