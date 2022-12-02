// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../../src/stake/HoprStakeSeason3.sol";
import "forge-std/Test.sol";

contract HoprStakeSeason3Test is Test {
    HoprStakeSeason3 public hoprStakeSeason3;

    function setUp() public virtual {
        // mock _nftAddress with vm.addr(1)
        // mock _newOwner with vm.addr(2)
        // mock _lockToken with vm.addr(3)
        // mock _rewardToke with vm.addr(4)
        hoprStakeSeason3 = new HoprStakeSeason3(vm.addr(1), vm.addr(2), vm.addr(3), vm.addr(4));
    }
}