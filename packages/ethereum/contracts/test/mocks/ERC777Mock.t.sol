// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.0;

import "../../src/static/stake/mocks/ERC777Mock.sol";
import "../utils/ERC1820Registry.sol";
import "forge-std/Test.sol";

/**
 * @title Simplified tests for ERC777 mock
 */
contract ERC777MockTest is Test, ERC1820RegistryFixtureTest {
    ERC777Mock public erc777Mock;

    function setUp() public virtual override {
        super.setUp();

        address[] memory defaultOperators = new address[](1);
        defaultOperators[0] = msg.sender;

        erc777Mock = new ERC777Mock(
            address(1),
            0,
            "ERC777 Mock Token",
            "ERC777Mock",
            defaultOperators
        );
    }

    function testFuzz_MintInternal(uint256 amount) public {
        address recipient = vm.addr(200);
        erc777Mock.mintInternal(recipient, amount, hex"00", hex"00");
        assertEq(erc777Mock.balanceOf(recipient), amount);
    }

    function testFuzz_ApproveInternal(address holder, address spender, uint256 amount) public {
        vm.assume(holder != address(0));
        vm.assume(spender != address(0));
        erc777Mock.approveInternal(holder, spender, amount);
        assertEq(erc777Mock.allowance(holder, spender), amount);
    }
}
