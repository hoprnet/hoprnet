// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../../src/proxy/HoprStakingProxyForNetworkRegistry.sol";
import "forge-std/Test.sol";

contract HoprStakingProxyForNetworkRegistryTest is Test {
    HoprStakingProxyForNetworkRegistry public hoprStakingProxyForNetworkRegistry;

    function setUp() public virtual {
        // mock _stakeContract with vm.addr(1)
        // mock _newOwner with vm.addr(2)
        // set _minStake with the production value
        hoprStakingProxyForNetworkRegistry = new HoprStakingProxyForNetworkRegistry(vm.addr(1), vm.addr(2), 1000);
    }
}