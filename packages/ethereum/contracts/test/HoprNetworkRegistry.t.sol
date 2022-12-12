// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import '../src/HoprNetworkRegistry.sol';
import 'forge-std/Test.sol';

contract HoprNetworkRegistryTest is Test {
  HoprNetworkRegistry public hoprNetworkRegistry;
  address public proxy;
  address public owner;

  /**
   * Manually import the errors and events
   */
  error InvalidPeerId(string peerId);

  event EnabledNetworkRegistry(bool indexed isEnabled); // Global toggle of the network registry
  event RequirementUpdated(address indexed requirementImplementation); // Emit when the network registry proxy is updated
  event Registered(address indexed account, string hoprPeerId); // Emit when an account register a node peer id for itself
  event Deregistered(address indexed account, string hoprPeerId); // Emit when an account deregister a node peer id for itself
  event RegisteredByOwner(address indexed account, string hoprPeerId); // Emit when the contract owner register a node peer id for an account
  event DeregisteredByOwner(address indexed account, string hoprPeerId); // Emit when the contract owner deregister a node peer id for an account
  event EligibilityUpdated(address indexed account, bool indexed eligibility); // Emit when the eligibility of an account is updated

  function setUp() public {
    proxy = vm.addr(100); // make vm.addr(100) requirementImplementation
    owner = vm.addr(101); // make address(101) new owner
    hoprNetworkRegistry = new HoprNetworkRegistry(proxy, owner);
  }

  function testFuzz_MockProxyReturn() public {
    _helperMockProxyReturns();
    (bool successRead0, bytes memory returndataAllowance0) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(1))
    );
    (bool successRead1, bytes memory returndataAllowance1) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(2))
    );
    (bool successRead2, bytes memory returndataAllowance2) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(3))
    );
    (bool successRead3, bytes memory returndataAllowance3) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(4))
    );
    (bool successRead4, bytes memory returndataAllowance4) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(5))
    );
    (bool successRead5, bytes memory returndataAllowance5) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(6))
    );
    uint256 allowance0 = abi.decode(returndataAllowance0, (uint256));
    uint256 allowance1 = abi.decode(returndataAllowance1, (uint256));
    uint256 allowance2 = abi.decode(returndataAllowance2, (uint256));
    uint256 allowance3 = abi.decode(returndataAllowance3, (uint256));
    uint256 allowance4 = abi.decode(returndataAllowance4, (uint256));
    uint256 allowance5 = abi.decode(returndataAllowance5, (uint256));
    assertEq(allowance0, type(uint256).max);
    assertEq(allowance1, type(uint256).max);
    assertEq(allowance2, 1);
    assertEq(allowance3, 1);
    assertEq(allowance4, 0);
    assertEq(allowance5, 0);
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it is enabled globally
   */
  function test_IsEnabled() public {
    _helperMockProxyReturns();
    assertTrue(hoprNetworkRegistry.enabled());
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it allows owner to update the registry
   */
  function testFuzz_OwnerUpdateRegistry(address newImp) public {
    _helperMockProxyReturns();
    vm.prank(owner);
    vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
    emit RequirementUpdated(newImp);
    hoprNetworkRegistry.updateRequirementImplementation(newImp);
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it fails to update the registry by a non-owner account
   */
  function testRevert_WhenNotOwnerUpdateRegistry(address caller) public {
    vm.assume(caller != owner);
    _helperMockProxyReturns();
    vm.prank(caller);
    vm.expectRevert('Ownable: caller is not the owner');
    hoprNetworkRegistry.updateRequirementImplementation(address(0));
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it fails to enable the registry by a non-owner account
   */
  function testRevert_WhenNotOwnerEnableRegistry(address caller) public {
    vm.assume(caller != owner);
    _helperMockProxyReturns();
    vm.prank(caller);
    vm.expectRevert('Ownable: caller is not the owner');
    hoprNetworkRegistry.enableRegistry();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it fails to enable an enabled registry by its owner
   */
  function testRevert_WhenEnablingAnEnabledRegistry() public {
    _helperMockProxyReturns();
    vm.prank(owner);
    vm.expectRevert('HoprNetworkRegistry: Registry is enabled');
    hoprNetworkRegistry.enableRegistry();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it allows owner to disable the registry
   */
  function test_OwnerDisableRegistry() public {
    _helperMockProxyReturns();
    vm.prank(owner);
    vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
    emit EnabledNetworkRegistry(false);
    hoprNetworkRegistry.disableRegistry();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it fails to enable an enabled registry by its owner
   */
  function testRevert_WhenDisablingADisnabledRegistry() public {
    _helperMockProxyReturns();
    vm.startPrank(owner);
    hoprNetworkRegistry.disableRegistry();

    vm.expectRevert('HoprNetworkRegistry: Registry is disabled');
    hoprNetworkRegistry.disableRegistry();

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it allows owner to enable an enabled registry
   */
  function test_OwnerEnsableRegistry() public {
    _helperMockProxyReturns();
    vm.startPrank(owner);
    hoprNetworkRegistry.disableRegistry();

    vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
    emit EnabledNetworkRegistry(true);
    hoprNetworkRegistry.enableRegistry();

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   *@dev Helper function to mock returns of `maxAllowedRegistrations` function on proxy contract
   */
  function _helperMockProxyReturns() internal {
    // account vm.addr(1) and vm.addr(2) have max allowance
    vm.mockCall(
      proxy,
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(1)),
      abi.encode(type(uint256).max)
    );
    vm.mockCall(
      proxy,
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(2)),
      abi.encode(type(uint256).max)
    );
    // account vm.addr(3) and vm.addr(4) have 1 allowance
    vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(3)), abi.encode(1));
    vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(4)), abi.encode(1));
    // account vm.addr(5) and vm.addr(6) have 0 allowance
    vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(5)), abi.encode(0));
    vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(6)), abi.encode(0));
  }
}
