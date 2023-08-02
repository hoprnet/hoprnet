// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Test.sol";
import "../src/TicketPriceOracle.sol";

contract TicketPriceOracleTest is Test {
    TicketPriceOracle public oracle;
    address public owner;

    /**
     * Manually import the errors and events
     */
    event TicketPriceUpdated(uint256, uint256);

    error TicketPriceMustNotBeZero();
    error TicketPriceMustNotBeSame();

    function setUp() public {
        owner = vm.addr(101); // make address(101) new owner
        oracle = new TicketPriceOracle(owner, 1);
    }

    function test_setZeroFails() public {
        vm.prank(owner);
        vm.expectRevert(TicketPriceMustNotBeZero.selector);
        oracle.setTicketPrice(0);
    }

    function test_setSameFails() public {
        vm.prank(owner);
        vm.expectRevert(TicketPriceMustNotBeSame.selector);
        oracle.setTicketPrice(1);
    }

    function test_setSameAfterUpdateFails() public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(oracle));
        emit TicketPriceUpdated(1, 2);
        oracle.setTicketPrice(2);

        vm.prank(owner);
        vm.expectRevert(TicketPriceMustNotBeSame.selector);
        oracle.setTicketPrice(2);
    }

    function test_setAsNonOwnerFails() public {
        vm.prank(vm.addr(102));
        vm.expectRevert(bytes("Ownable: caller is not the owner"));
        oracle.setTicketPrice(2);
    }

    function test_setUpAndDown() public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(oracle));
        emit TicketPriceUpdated(1, 2);
        oracle.setTicketPrice(2);

        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(oracle));
        emit TicketPriceUpdated(2, 1);
        oracle.setTicketPrice(1);
    }

    function testFuzz_setUpAndDown(uint256 price) public {
        if (price == 0) {
            vm.expectRevert(TicketPriceMustNotBeZero.selector);
        } else {
            if (price == oracle.currentTicketPrice()) {
                vm.expectRevert(TicketPriceMustNotBeSame.selector);
            } else {
                vm.expectEmit(true, false, false, false, address(oracle));
                emit TicketPriceUpdated(oracle.currentTicketPrice(), price);
            }
        }
        vm.prank(owner);
        oracle.setTicketPrice(price);
    }
}
