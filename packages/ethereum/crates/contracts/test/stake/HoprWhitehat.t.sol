// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../../src/stake/HoprWhitehat.sol";
import "forge-std/Test.sol";

contract HoprWhitehatTest is Test {
    HoprWhitehat public hoprWhitehat;

    function setUp() public virtual {
        // mock _newOwner with vm.addr(1)
        // mock _myHoprBoost with vm.addr(2)
        // mock _myHoprStake with vm.addr(3)
        // mock _xHopr with vm.addr(4)
        // mock _wxHopr with vm.addr(5)
        hoprWhitehat = new HoprWhitehat(vm.addr(1), vm.addr(2), vm.addr(3), vm.addr(4), vm.addr(5));
    }
}