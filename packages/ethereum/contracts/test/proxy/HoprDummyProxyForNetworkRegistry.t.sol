// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../../src/proxy/HoprDummyProxyForNetworkRegistry.sol";
import "forge-std/Test.sol";

contract HoprDummyProxyForNetworkRegistryTest is Test {
    HoprDummyProxyForNetworkRegistry public hoprDummyProxyForNetworkRegistry;

    function setUp() public virtual {
        // mock _newOwner with vm.addr(1)
        hoprDummyProxyForNetworkRegistry = new HoprDummyProxyForNetworkRegistry(vm.addr(1));
    }
}