// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { Test } from "forge-std/Test.sol";
import { HoprTicketPriceOracle, HoprTicketPriceOracleEvents } from "../src/TicketPriceOracle.sol";

contract TicketPriceOracleTest is Test, HoprTicketPriceOracleEvents {
    HoprTicketPriceOracle public oracle;
    address public owner;

    function setUp() public {
        owner = vm.addr(101); // make address(101) new owner
        oracle = new HoprTicketPriceOracle(owner, 1);
    }

    function test_setZeroFails() public {
        vm.prank(owner);
        vm.expectRevert(HoprTicketPriceOracle.TicketPriceMustNotBeZero.selector);
        oracle.setTicketPrice(0);
    }

    function test_setSameFails() public {
        vm.prank(owner);
        vm.expectRevert(HoprTicketPriceOracle.TicketPriceMustNotBeSame.selector);
        oracle.setTicketPrice(1);
    }

    function test_setSameAfterUpdateFails() public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(oracle));
        emit TicketPriceUpdated(1, 2);
        oracle.setTicketPrice(2);

        vm.prank(owner);
        vm.expectRevert(HoprTicketPriceOracle.TicketPriceMustNotBeSame.selector);
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
            vm.expectRevert(HoprTicketPriceOracle.TicketPriceMustNotBeZero.selector);
        } else {
            if (price == oracle.currentTicketPrice()) {
                vm.expectRevert(HoprTicketPriceOracle.TicketPriceMustNotBeSame.selector);
            } else {
                vm.expectEmit(true, false, false, false, address(oracle));
                emit TicketPriceUpdated(oracle.currentTicketPrice(), price);
            }
        }
        vm.prank(owner);
        oracle.setTicketPrice(price);
    }
}
