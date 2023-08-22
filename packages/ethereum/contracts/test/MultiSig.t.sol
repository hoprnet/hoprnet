// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { Test } from "forge-std/Test.sol";
import { AccountsFixtureTest } from "./utils/Accounts.sol";

import { HoprMultiSig } from "../src/MultiSig.sol";
import { HoprNodeSafeRegistry } from "../src/node-stake/NodeSafeRegistry.sol";

// We need this dummy contract to correctly set msg.sender
// when testing modifiers of abstract contracts
contract MultiSigContract is HoprMultiSig {
    function modifierOnlySafe(address self) public onlySafe(self) { }

    function modifierNoSafeSet() public noSafeSet { }

    function mySetNodeSafeRegistry(HoprNodeSafeRegistry registry) public {
        setNodeSafeRegistry(registry);
    }
}

contract MulitSigTest is Test, AccountsFixtureTest {
    HoprNodeSafeRegistry safeRegistry;
    MultiSigContract msContract;

    function setUp() public {
        msContract = new MultiSigContract();
        safeRegistry = new HoprNodeSafeRegistry();
    }

    function testRevert_initializeTwice() public {
        msContract.mySetNodeSafeRegistry(safeRegistry);

        vm.expectRevert(HoprMultiSig.AlreadyInitialized.selector);
        msContract.mySetNodeSafeRegistry(safeRegistry);
    }

    function test_emptySafeAddress() public {
        (bool success, bytes memory result) = address(msContract).staticcall(
            abi.encodeWithSelector(MultiSigContract.mySetNodeSafeRegistry.selector, address(0))
        );

        assertFalse(success);
        assertEq(bytes32(result), HoprMultiSig.InvalidSafeAddress.selector);
    }

    function testRevert_uninitialized(address caller) public {
        vm.expectRevert(HoprMultiSig.MultiSigUninitialized.selector);

        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.prank(caller);
        msContract.modifierNoSafeSet();

        vm.clearMockedCalls();
    }

    function test_noSafeSet(address caller) public {
        msContract.mySetNodeSafeRegistry(safeRegistry);
        vm.prank(caller);

        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        msContract.modifierNoSafeSet();

        vm.clearMockedCalls();
    }

    function test_noSafeSetButSafeSet(address caller, address safeAddress) public {
        vm.assume(safeAddress != address(0));

        msContract.mySetNodeSafeRegistry(safeRegistry);
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(1))
        );

        vm.prank(caller);
        vm.expectRevert(HoprMultiSig.ContractNotResponsible.selector);
        msContract.modifierNoSafeSet();

        vm.clearMockedCalls();
    }

    function test_onlySafe(address safeAddr, address caller) public {
        vm.assume(safeAddr != address(0));

        msContract.mySetNodeSafeRegistry(safeRegistry);

        vm.mockCall(address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(safeAddr));

        vm.prank(safeAddr);
        msContract.modifierOnlySafe(caller);
    }

    function testRevert_onlySafe(address safeAddr, address caller, address setSafeAddr) public {
        vm.assume(safeAddr != address(0));
        vm.assume(safeAddr != setSafeAddr);
        vm.assume(setSafeAddr != address(0));

        msContract.mySetNodeSafeRegistry(safeRegistry);

        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(setSafeAddr)
        );

        vm.expectRevert(HoprMultiSig.ContractNotResponsible.selector);
        vm.prank(safeAddr);
        msContract.modifierOnlySafe(caller);
    }
}
