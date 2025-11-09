// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { Test } from "forge-std/Test.sol";
import { Ownable2Step } from "openzeppelin-contracts/access/Ownable2Step.sol";
import {
    WinProb,
    HoprWinningProbabilityOracle,
    HoprWinningProbabilityOracleEvents
} from "../src/WinningProbabilityOracle.sol";

contract Ownable2StepEvents {
    event OwnershipTransferStarted(address indexed previousOwner, address indexed newOwner);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
}

contract TicketWinningProbabilityOracleTest is Test, HoprWinningProbabilityOracleEvents, Ownable2StepEvents {
    HoprWinningProbabilityOracle public oracle;
    address public owner;

    function setUp() public {
        owner = vm.addr(101); // make address(101) new owner
        oracle = new HoprWinningProbabilityOracle(owner, WinProb.wrap(0xffffffffffffff));
    }

    function test_setUpWithZero() public {
        HoprWinningProbabilityOracle newOracle = new HoprWinningProbabilityOracle(owner, WinProb.wrap(0));
        assertEq(address(oracle).code, address(newOracle).code);
        assertEq(WinProb.unwrap(newOracle.currentWinProb()), 0);
    }

    function test_setZero() public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(oracle));
        emit WinProbUpdated(WinProb.wrap(0xffffffffffffff), WinProb.wrap(0));
        oracle.setWinProb(WinProb.wrap(0));
    }

    function testRevert_setSameFails() public {
        vm.prank(owner);
        vm.expectRevert(HoprWinningProbabilityOracle.WinProbMustNotBeSame.selector);
        oracle.setWinProb(WinProb.wrap(0xffffffffffffff));
    }

    function testRevert_setByNonOwnerFails() public {
        vm.prank(vm.addr(102));
        vm.expectRevert(bytes("Ownable: caller is not the owner"));
        oracle.setWinProb(WinProb.wrap(0xffffffffffff00));
    }

    function testFuzz_setWinProb(WinProb newWinProb) public {
        if (newWinProb == oracle.currentWinProb()) {
            // the new winning probability must not be the same as the current one
            vm.expectRevert(HoprWinningProbabilityOracle.WinProbMustNotBeSame.selector);
        } else {
            vm.expectEmit(true, false, false, false, address(oracle));
            emit WinProbUpdated(oracle.currentWinProb(), newWinProb);
        }

        vm.prank(owner);
        oracle.setWinProb(newWinProb);
    }

    function test_setALargeValueFails() public {
        // use a value that is larger than the max of WinProb type
        uint256 largeValue = type(uint256).max;
        vm.prank(owner);

        // fail to use a low-level call to set a large value
        bytes memory payload = abi.encodePacked(oracle.setWinProb.selector, largeValue);
        (bool success,) = address(oracle).call(payload);
        assertFalse(success);
    }

    function test_transferOwnership() public {
        address newOwner = vm.addr(103);
        // initiate the ownership transfer
        vm.prank(owner);
        vm.expectEmit(true, true, true, false, address(oracle));
        emit OwnershipTransferStarted(owner, newOwner);
        oracle.transferOwnership(newOwner);

        // complete the ownership transfer
        vm.prank(newOwner);
        vm.expectEmit(true, true, true, false, address(oracle));
        emit OwnershipTransferred(owner, newOwner);
        oracle.acceptOwnership();
        assertEq(oracle.owner(), newOwner);
    }

    function testRevert_acceptOwnershipByNonPendingOwner() public {
        address newOwner = vm.addr(103);

        // Initiate the ownership transfer
        vm.prank(owner);
        oracle.transferOwnership(newOwner);

        // Attempt to accept ownership by an unauthorized address
        vm.prank(vm.addr(104));
        vm.expectRevert("Ownable2Step: caller is not the new owner");
        oracle.acceptOwnership();
    }
}
