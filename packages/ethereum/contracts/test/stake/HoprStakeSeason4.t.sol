// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../../src/stake/HoprStakeSeason4.sol";
import "forge-std/Test.sol";

contract HoprStakeSeason4Test is Test {
    HoprStakeSeason4 public hoprStakeSeason4;

    function setUp() public virtual {
        // mock _nftAddress with vm.addr(1)
        // mock _newOwner with vm.addr(2)
        // mock _lockToken with vm.addr(3)
        // mock _rewardToke with vm.addr(4)
        hoprStakeSeason4 = new HoprStakeSeason4(vm.addr(1), vm.addr(2), vm.addr(3), vm.addr(4));
    }
}