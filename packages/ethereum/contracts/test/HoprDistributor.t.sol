// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../src/HoprDistributor.sol";
import "../src/HoprToken.sol";
import "forge-std/Test.sol";

contract HoprDistributorTest is Test {
    HoprDistributor public hoprDistributor;

    function setUp() public {
        // make vm.addr(1) HoprToken contract
        // use production parameters here
        hoprDistributor = new HoprDistributor(HoprToken(vm.addr(1)), 1614862800, 13333333000000000000000000);
    }
}