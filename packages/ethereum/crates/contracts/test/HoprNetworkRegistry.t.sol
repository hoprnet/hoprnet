// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../src/HoprNetworkRegistry.sol";
import "forge-std/Test.sol";

contract HoprNetworkRegistryTest is Test {
    HoprNetworkRegistry public hoprNetworkRegistry;

    function setUp() public {
        // make vm.addr(1) requirementImplementation
        // make address(1) new owner
        hoprNetworkRegistry = new HoprNetworkRegistry(vm.addr(1), address(1));
    }
}