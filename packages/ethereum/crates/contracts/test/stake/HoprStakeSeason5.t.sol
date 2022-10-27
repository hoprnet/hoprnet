// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../../src/stake/HoprStakeSeason5.sol";
import "forge-std/Test.sol";

contract HoprStakeSeason5Test is Test {
    HoprStakeSeason5 public hoprStakeSeason5;

    function setUp() public virtual {
        // mock _newOwner with vm.addr(1)
        // mock _nftAddress with vm.addr(2)
        // mock _lockToken with vm.addr(3)
        // mock _rewardToken with vm.addr(4)
        hoprStakeSeason5 = new HoprStakeSeason5(vm.addr(1), vm.addr(2), vm.addr(3), vm.addr(4));
    }
}
