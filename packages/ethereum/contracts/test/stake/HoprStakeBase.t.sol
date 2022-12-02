// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../../src/stake/HoprStakeBase.sol";
import "forge-std/Test.sol";

contract HoprStakeBaseTest is Test {
    HoprStakeBase public hoprStakeBase;

    function setUp() public virtual {
        // mock _newOwner with vm.addr(1)
        // mock _programStart with block.timestamp
        // mock _programEnd with block.timestamp + 3000
        // mock _basicFactorNumerator with 100
        // mock _boostCap with 1 ether
        // mock _nftAddress with vm.addr(2)
        // mock _lockToken with vm.addr(3)
        // mock _rewardToken with vm.addr(4)
        hoprStakeBase = new HoprStakeBase(vm.addr(1), uint256(block.timestamp), uint256(block.timestamp + 3000), 100, 1 ether, vm.addr(2), vm.addr(3), vm.addr(4));
    }
}
