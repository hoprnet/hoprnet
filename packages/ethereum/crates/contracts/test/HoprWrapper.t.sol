// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../src/HoprWrapper.sol";
import "../src/HoprToken.sol";
import "./utils/ERC1820Registry.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract HoprWrapperTest is Test, ERC1820RegistryFixture {
    HoprWrapper public hoprWrapper;

    function setUp() public virtual override {
        super.setUp();
        // erc20Mock = new ERC20Mock(address(1), type(uint256).max);
        // make vm.addr(1) HoprToken contract
        // make vm.addr(2) xHOPR contract
        hoprWrapper = new HoprWrapper(IERC20(vm.addr(2)), HoprToken(vm.addr(1)));
    }
}