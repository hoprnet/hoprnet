// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import './utils/Accounts.sol';

import '../src/MultiSig.sol';
import '../src/node-stake/NodeSafeRegistry.sol';

// We need this dummy contract to correctly set msg.sender
// when testing modifiers of abstract contracts
contract MultiSigContract is HoprMultiSig {
  function modifierOnlySafe(address self) onlySafe(self) public {}

  function modifierNoSafeSet() noSafeSet public {}

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

    vm.expectRevert(AlreadyInitialized.selector);
    msContract.mySetNodeSafeRegistry(safeRegistry);
  }

  function testRevert_emptySafeAddress() public {
    vm.expectRevert(InvalidSafeAddress.selector);
    (bool success, bytes memory result) = address(msContract).staticcall(abi.encodeWithSelector(MultiSigContract.mySetNodeSafeRegistry.selector, address(0)));

    assertFalse(success);
    assertEq(bytes32(result), bytes32(0x0000000000000000000000000000000000000000000000000000000000000020)); // error code
  }

  function testRevert_uninitialized(address caller) public {
    vm.expectRevert(MultiSigUninitialized.selector);

    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(address(0))
    );

    vm.prank(caller);
    msContract.modifierNoSafeSet();

    vm.clearMockedCalls();
  }

  function test_noSafeSet(address caller) public {
    msContract.mySetNodeSafeRegistry(safeRegistry);
    vm.prank(caller);

    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(address(0))
    );

    msContract.modifierNoSafeSet();

    vm.clearMockedCalls();
  }

  function test_noSafeSetButSafeSet(address caller, address safeAddress) public {
    vm.assume(safeAddress != address(0));

    msContract.mySetNodeSafeRegistry(safeRegistry);
    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(address(1))
    );
    
    vm.prank(caller);
    vm.expectRevert(ContractNotResponsible.selector);
    msContract.modifierNoSafeSet();

    vm.clearMockedCalls();
  }

  function test_onlySafe(address safeAddr, address caller) public {
    vm.assume(safeAddr != address(0));

    msContract.mySetNodeSafeRegistry(safeRegistry);

    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(safeAddr)
    );

    vm.prank(safeAddr);
    msContract.modifierOnlySafe(caller);
  }

  function testRevert_onlySafe(address safeAddr, address caller, address setSafeAddr) public {
    vm.assume(safeAddr != address(0));
    vm.assume(safeAddr != setSafeAddr);
    vm.assume(setSafeAddr != address(0));

    msContract.mySetNodeSafeRegistry(safeRegistry);

    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(setSafeAddr)
    );

    vm.expectRevert(ContractNotResponsible.selector);
    vm.prank(safeAddr);
    msContract.modifierOnlySafe(caller);
  }
}